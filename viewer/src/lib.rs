mod ui;

use egui_winit_vulkano::{Gui, GuiConfig};
use render::scene::Scene;
use ui::{Ui, Window};
use vulkano::{
    device::{DeviceExtensions, Features},
    format::Format,
    image::SampleCount,
};
use vulkano_util::{
    context::{VulkanoConfig, VulkanoContext},
    window::{VulkanoWindows, WindowDescriptor},
};
use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop},
};

pub const IMAGE_FORMAT: Format = Format::B8G8R8A8_SRGB;

pub fn run_viewer(scene: Scene) -> ! {
    let event_loop = EventLoop::new();
    let context = VulkanoContext::new(VulkanoConfig {
        device_features: Features {
            dynamic_rendering: false,
            sample_rate_shading: true,
            wide_lines: true,
            rectangular_lines: true,
            independent_blend: true,
            fill_mode_non_solid: true,
            ..Default::default()
        },
        device_extensions: DeviceExtensions {
            khr_push_descriptor: true,
            khr_swapchain: true,
            ext_line_rasterization: true,
            ext_blend_operation_advanced: false,
            ..Default::default()
        },
        ..Default::default()
    });

    let mut windows = VulkanoWindows::default();
    windows.create_window(
        &event_loop,
        &context,
        &WindowDescriptor {
            position: Some([320.0, 50.0]),
            width: 1600.0,
            height: 900.0,
            ..Default::default()
        },
        |ci| ci.image_format = Some(IMAGE_FORMAT),
    );

    let mut window = Ui::new(scene);

    let mut gui = {
        let renderer = windows.get_primary_renderer_mut().unwrap();

        Gui::new(
            &event_loop,
            renderer.surface(),
            renderer.graphics_queue(),
            GuiConfig {
                preferred_format: Some(IMAGE_FORMAT),
                is_overlay: false,
                samples: SampleCount::Sample1,
            },
        )
    };

    event_loop.run(move |event, _, control_flow| {
        let renderer = windows.get_primary_renderer_mut().unwrap();

        match &event {
            Event::WindowEvent { window_id, event } => {
                if *window_id != renderer.window().id() {
                    return;
                }

                gui.update(&event);

                match event {
                    winit::event::WindowEvent::Resized(_)
                    | winit::event::WindowEvent::ScaleFactorChanged { .. } => {
                        renderer.resize();
                    }
                    winit::event::WindowEvent::CloseRequested => {
                        if window.on_close() {
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                    _ => (),
                }
            }
            Event::RedrawRequested(window_id) => {
                if *window_id != renderer.window().id() {
                    return;
                }

                window.draw(&mut gui);

                let before_future = renderer.acquire().unwrap();
                let after_future =
                    gui.draw_on_image(before_future, renderer.swapchain_image_view());
                renderer.present(after_future, true);
            }
            Event::MainEventsCleared => {
                renderer.window().request_redraw();
            }
            _ => {}
        };

        window.on_event(&event, control_flow, renderer, &mut gui);
    });
}
