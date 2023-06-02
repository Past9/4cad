use camera::Camera;
use cgmath::{Angle, Deg, InnerSpace, Rad};
use error::RenderResult;
use model::model::{BufferedModel, Model};
use renderer::{MsaaSamples, Renderer};
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

pub mod camera;
pub mod error;
pub mod model;
pub mod renderer;

pub type Vec3 = cgmath::Vector3<f32>;
pub type Point3 = cgmath::Point3<f32>;
pub type Quat = cgmath::Quaternion<f32>;
pub type Mat4 = cgmath::Matrix4<f32>;

const ROTATION_SENSITIVITY: f32 = 0.007;
const ZOOM_SENSITIVITY: f32 = 0.02;

pub trait Vec3Utils {
    fn to_f32_array(&self) -> [f32; 3];
}

impl Vec3Utils for Vec3 {
    fn to_f32_array(&self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }
}

pub fn render_model(model: Model, dist: f32) -> RenderResult<()> {
    let event_loop = EventLoop::new();

    let mut renderer = Renderer::new(
        &event_loop,
        MsaaSamples::Samples8,
        //[0.0, 0.0, 0.0],
        //[1.0, 1.0, 1.0],
        [0.15, 0.2, 0.25],
        true,
        true,
        true,
    )?;

    let model = BufferedModel::from_mesh(renderer.device(), model);

    let mut camera = Camera::default();
    camera.set_position(Point3::new(0.0, 0.0, dist));
    camera.set_look_at(Point3::new(0.0, 0.0, 0.0));
    camera.set_fov(Deg(60.0));
    camera.set_near_clip(1.0);
    camera.set_far_clip(16000.0);

    let mut last_fps_report = std::time::Instant::now();
    let fps_report_interval = std::time::Duration::from_millis(100);

    let mut is_rmb_pressed = false;
    let mut is_mmb_pressed = false;
    let mut is_shift_pressed = false;

    let mut last_mouse_pos = PhysicalPosition { x: 0f64, y: 0f64 };

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *control_flow = ControlFlow::Exit;
        }
        Event::WindowEvent {
            event: WindowEvent::Resized(size),
            ..
        } => {
            camera.set_viewport_dimensions(size.width, size.height);
            renderer.recreate_swapchain();
        }
        Event::WindowEvent {
            event:
                WindowEvent::MouseInput {
                    state,
                    button: MouseButton::Right,
                    ..
                },
            ..
        } => {
            is_rmb_pressed = match state {
                ElementState::Pressed => true,
                ElementState::Released => false,
            }
        }
        Event::WindowEvent {
            event:
                WindowEvent::MouseInput {
                    state,
                    button: MouseButton::Middle,
                    ..
                },
            ..
        } => {
            is_mmb_pressed = match state {
                ElementState::Pressed => true,
                ElementState::Released => false,
            }
        }
        Event::WindowEvent {
            event: WindowEvent::CursorMoved { position, .. },
            ..
        } => {
            // Rotate on RMB and no shift
            if is_rmb_pressed && !is_shift_pressed {
                let dx = -(position.x - last_mouse_pos.x) as f32;
                let dy = -(position.y - last_mouse_pos.y) as f32;

                camera.rotate(
                    Vec3::new(dy, dx, 0.0 as f32),
                    Rad(Vec3::new(dx, dy, 0.0).magnitude() * ROTATION_SENSITIVITY),
                );
            }

            // Pan on MMB or RMB + Shift
            if is_mmb_pressed || (is_rmb_pressed && is_shift_pressed) {
                let (w, h) = renderer.dimensions();
                let w = w;
                let h = h;

                let fov = camera.get_fov();
                let ar = camera.get_aspect_ratio();
                let cam_z = camera.get_position().z;

                let last_mouse_x = (last_mouse_pos.x as f32 - w / 2.0) / w * 2.0;
                let current_mouse_x = (position.x as f32 - w / 2.0) / w * 2.0;
                let last_x_angle = Rad(((fov / 2.0).0.tan() * last_mouse_x * ar).atan());
                let current_x_angle = Rad(((fov / 2.0).0.tan() * current_mouse_x * ar).atan());
                let x_move = (current_x_angle - last_x_angle).tan() * cam_z;

                let last_mouse_y = (last_mouse_pos.y as f32 - h / 2.0) / h * 2.0;
                let current_mouse_y = (position.y as f32 - h / 2.0) / h * 2.0;
                let last_y_angle = Rad(((fov / 2.0).0.tan() * last_mouse_y).atan());
                let current_y_angle = Rad(((fov / 2.0).0.tan() * current_mouse_y).atan());
                let y_move = (current_y_angle - last_y_angle).tan() * cam_z;

                camera.translate(Vec3::new(x_move, y_move, 0.0));
            }

            last_mouse_pos = position;
        }
        Event::WindowEvent {
            event:
                WindowEvent::MouseWheel {
                    delta: MouseScrollDelta::LineDelta(_x, y),
                    ..
                },
            ..
        } => {
            let y = y as f32;

            // Zoom on mousewheel
            let (w, h) = renderer.dimensions();
            let w = w as f32;
            let h = h as f32;

            let mouse_x = (last_mouse_pos.x as f32 - w / 2.0) / w * 2.0;
            let mouse_y = (last_mouse_pos.y as f32 - h / 2.0) / h * 2.0;

            let fov = camera.get_fov();
            let ar = camera.get_aspect_ratio();

            let x_angle = Rad(((fov / 2.0).0.tan() * mouse_x * ar).atan());
            let y_angle = Rad(((fov / 2.0).0.tan() * mouse_y).atan());

            let z = -camera.get_position().z * (y * ZOOM_SENSITIVITY);

            let x_move = z * x_angle.tan();
            let y_move = z * y_angle.tan();

            camera.translate(Vec3::new(x_move, y_move, z));
        }
        Event::RedrawEventsCleared => {
            match renderer.render(&model, &camera) {
                Ok(()) => {}
                Err(err) => panic!("{:#?}", err),
            };

            let now = std::time::Instant::now();
            if now - last_fps_report >= fps_report_interval {
                println!("{:?} FPS", renderer.fps());
                last_fps_report = now;
            }
        }
        Event::WindowEvent {
            event: WindowEvent::ModifiersChanged(modifiers),
            ..
        } => {
            is_shift_pressed = modifiers.shift();
        }
        _ => {}
    });

    Ok(())
}
