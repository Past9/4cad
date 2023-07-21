use graphics::*;
use winit::{
    dpi::PhysicalSize,
    event::{Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

fn main() {
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::<u32>::from((1600, 900)))
        .with_title("brim-win")
        .build(&event_loop)
        .unwrap();

    let mut program = build_program(&window);
    Vulkan::execute(&mut program);

    println!("program {:#?}", program);

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            window_id: _,
            event:
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                },
        } => {
            *control_flow = ControlFlow::Exit;
        }
        Event::LoopDestroyed => {
            *control_flow = ControlFlow::Exit;
        }
        _ => {}
    });
}

fn build_program(window: &Window) -> Program {
    let mut program = Program::on_window(window);

    program.render_pass(|render_pass| {
        render_pass.msaa_samples(MsaaSamples::Samples1);

        let depth_attachment = render_pass.attachment(
            Attachment::depth()
                .format(Format::absolute(AbsoluteFormat::D16_UNORM))
                .load_op(LoadOp::Clear)
                .initial_layout(Layout::DepthStencil)
                .final_layout(Layout::DepthStencil),
        );

        let color_attachment = render_pass.attachment(
            Attachment::color()
                .format(Format::reference(FormatRef::Surface))
                .load_op(LoadOp::Clear)
                .store_op(StoreOp::Store)
                .final_layout(Layout::PresentationSource)
                .output(),
        );

        render_pass.subpass(|subpass| {
            subpass
                .color_attachment(color_attachment)
                .depth_attachment(depth_attachment)
                .pipeline(|pipeline| {
                    pipeline
                        .vertex_shader("shaders/triangle.vert")
                        .fragment_shader("shaders/triangle.frag");

                    // TODO: Add shaders and stuff
                });
        });
    });

    program
}
