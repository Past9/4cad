use cgmath::{Deg, InnerSpace, Matrix4, Point3, Quaternion, Rad, Rotation3, Vector3, Vector4};

#[derive(Clone)]
pub struct Camera {
    position: Point3<f32>,
    direction: Vector3<f32>,
    up: Vector3<f32>,
    fov: Rad<f32>,
    near_clip: f32,
    far_clip: f32,
    rotation: Quaternion<f32>,
    viewport_width: f32,
    viewport_height: f32,
}
impl Camera {
    pub fn get_fov(&self) -> Rad<f32> {
        self.fov
    }

    pub fn get_aspect_ratio(&self) -> f32 {
        self.viewport_width / self.viewport_height
    }

    pub fn get_position(&self) -> Point3<f32> {
        self.position
    }

    pub fn set_up(&mut self, up: Vector3<f32>) {
        self.up = up;
    }

    pub fn set_position(&mut self, position: Point3<f32>) {
        self.position = position;
    }

    pub fn set_look_at(&mut self, target: Point3<f32>) {
        self.direction = target - self.position;
    }

    pub fn set_viewport_dimensions(&mut self, width: u32, height: u32) {
        self.viewport_width = width as f32;
        self.viewport_height = height as f32;
    }

    pub fn set_fov<A: Into<Rad<f32>>>(&mut self, angle: A) {
        self.fov = angle.into();
    }

    pub fn set_near_clip(&mut self, z: f32) {
        self.near_clip = z;
    }

    pub fn set_far_clip(&mut self, z: f32) {
        self.far_clip = z;
    }

    pub fn rotate(&mut self, axis: Vector3<f32>, angle: Rad<f32>) -> &mut Self {
        self.rotation = Quaternion::<f32>::from_axis_angle(axis.normalize(), angle) * self.rotation;
        self
    }

    pub fn translate(&mut self, translation: Vector3<f32>) -> &mut Self {
        self.position += translation;
        self
    }

    pub fn get_perspective_matrix(&self) -> Matrix4<f32> {
        let f = self.far_clip;
        let n = self.near_clip;
        let aspect_ratio = self.get_aspect_ratio();
        let fov = self.get_fov().0;
        let focal_length = 1.0 / (fov / 2.0).tan();
        let x = focal_length / aspect_ratio;
        let y = -focal_length;
        let a = n / (f - n);
        let b = f * a;

        Matrix4 {
            x: Vector4::new(x, 0.0, 0.0, 0.0),
            y: Vector4::new(0.0, y, 0.0, 0.0),
            z: Vector4::new(0.0, 0.0, a, -1.0),
            w: Vector4::new(0.0, 0.0, b, 0.0),
        }
    }

    pub fn get_view_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_to_rh(
            self.position.clone(),
            self.direction.clone(),
            self.up.clone(),
        ) * Matrix4::<f32>::from(self.rotation)
    }
}
impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Point3::new(0.0, 0.0, 0.0),
            direction: Vector3::new(0.0, 0.0, 1.0),
            up: Vector3::new(0.0, -1.0, 0.0),
            fov: Deg(60.0).into(),
            near_clip: 0.1,
            far_clip: 1024.0,
            rotation: Quaternion::<f32>::from_axis_angle((0.0, 1.0, 0.0).into(), Rad(0.0)),
            viewport_width: 1.0,
            viewport_height: 1.0,
        }
    }
}
