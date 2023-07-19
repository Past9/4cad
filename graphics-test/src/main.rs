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

    let program = build_program(&window);

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

        let color_attachment = render_pass.attachment(
            Attachment::color()
                .format(Format::reference(FormatRef::Surface))
                .load_op(LoadOp::Clear)
                .store_op(StoreOp::Store)
                .final_layout(Layout::PresentationSource),
        );

        let depth_attachment = render_pass.attachment(
            Attachment::depth()
                .format(Format::absolute(AbsoluteFormat::D16_UNORM))
                .load_op(LoadOp::Clear)
                .initial_layout(Layout::DepthStencil)
                .final_layout(Layout::DepthStencil),
        );

        render_pass.subpass(|subpass| {
            subpass
                .color_attachment(color_attachment)
                .depth_attachment(depth_attachment)
                .pipeline(|pipeline| {
                    pipeline;

                    // TODO: Add shaders and stuff
                });
        });
    });

    program
}

mod triangle_vs {
    graphics::shader! {{
            ty: "vertex",
            src: "
#version 400
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (location = 0) in vec4 pos;
layout (location = 1) in vec4 color;


layout (location = 0) out vec4 o_color;
void main() {
    o_color = color;
    gl_Position = pos;
}
        "
    }}
}

/*
mod triangle_vs {
    graphics::shader! {
        ty: "vertex",
        src: "
#version 400
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (location = 0) in vec4 pos;
layout (location = 1) in vec4 color;


layout (location = 0) out vec4 o_color;
void main() {
    o_color = color;
    gl_Position = pos;
}
        "
    }
}

 */
