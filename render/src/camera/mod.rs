use cgmath::{
    point3, vec2, vec3, vec4, Angle, Deg, EuclideanSpace, InnerSpace, Matrix4, Point3, Quaternion,
    Rad, Rotation3, SquareMatrix, Vector2, Vector3, Vector4,
};

#[derive(Debug, Clone, Copy)]
pub enum CameraAngle {
    // Faces
    Front,
    Right,
    Back,
    Left,
    Top,
    Bottom,

    // Edges
    FrontRight,
    BackRight,
    BackLeft,
    FrontLeft,
    FrontTop,
    BackTop,
    FrontBottom,
    BackBottom,
    RightTop,
    LeftTop,
    RightBottom,
    LeftBottom,

    // Corners
    FrontRightTop,
    BackRightTop,
    BackLeftTop,
    FrontLeftTop,
    FrontRightBottom,
    BackRightBottom,
    BackLeftBottom,
    FrontLeftBottom,
}
impl CameraAngle {
    pub fn from_name(name: &str) -> Option<Self> {
        match &*name {
            "Front" => Some(Self::Front),
            "Right" => Some(Self::Right),
            "Back" => Some(Self::Back),
            "Left" => Some(Self::Left),
            "Top" => Some(Self::Top),
            "Bottom" => Some(Self::Bottom),
            "FrontRight" => Some(Self::FrontRight),
            "BackRight" => Some(Self::BackRight),
            "BackLeft" => Some(Self::BackLeft),
            "FrontLeft" => Some(Self::FrontLeft),
            "FrontTop" => Some(Self::FrontTop),
            "BackTop" => Some(Self::BackTop),
            "FrontBottom" => Some(Self::FrontBottom),
            "BackBottom" => Some(Self::BackBottom),
            "RightTop" => Some(Self::RightTop),
            "LeftTop" => Some(Self::LeftTop),
            "RightBottom" => Some(Self::RightBottom),
            "LeftBottom" => Some(Self::LeftBottom),
            "FrontRightTop" => Some(Self::FrontRightTop),
            "BackRightTop" => Some(Self::BackRightTop),
            "BackLeftTop" => Some(Self::BackLeftTop),
            "FrontLeftTop" => Some(Self::FrontLeftTop),
            "FrontRightBottom" => Some(Self::FrontRightBottom),
            "BackRightBottom" => Some(Self::BackRightBottom),
            "BackLeftBottom" => Some(Self::BackLeftBottom),
            "FrontLeftBottom" => Some(Self::FrontLeftBottom),
            _ => None,
        }
    }

    pub fn get_rotation(&self) -> Quaternion<f32> {
        let x: i32 = match self {
            // Faces
            CameraAngle::Front => 0,
            CameraAngle::Right => 0,
            CameraAngle::Back => 0,
            CameraAngle::Left => 0,
            CameraAngle::Top => 270,
            CameraAngle::Bottom => 90,

            // Edges
            CameraAngle::FrontRight => 0,
            CameraAngle::BackRight => 0,
            CameraAngle::BackLeft => 0,
            CameraAngle::FrontLeft => 0,
            CameraAngle::FrontTop => -45,
            CameraAngle::BackTop => -45,
            CameraAngle::FrontBottom => 45,
            CameraAngle::BackBottom => 45,
            CameraAngle::RightTop => -45,
            CameraAngle::LeftTop => -45,
            CameraAngle::RightBottom => 45,
            CameraAngle::LeftBottom => 45,

            // Corners
            CameraAngle::FrontRightTop => -45,
            CameraAngle::BackRightTop => -45,
            CameraAngle::BackLeftTop => -45,
            CameraAngle::FrontLeftTop => -45,
            CameraAngle::FrontRightBottom => 45,
            CameraAngle::BackRightBottom => 45,
            CameraAngle::BackLeftBottom => 45,
            CameraAngle::FrontLeftBottom => 45,
        };

        let y: i32 = match self {
            // Faces
            CameraAngle::Front => 0,
            CameraAngle::Right => -90,
            CameraAngle::Back => 180,
            CameraAngle::Left => 90,
            CameraAngle::Top => 0,
            CameraAngle::Bottom => 0,

            // Edges
            CameraAngle::FrontRight => -45,
            CameraAngle::BackRight => -135,
            CameraAngle::BackLeft => 135,
            CameraAngle::FrontLeft => 45,
            CameraAngle::FrontTop => 0,
            CameraAngle::BackTop => 180,
            CameraAngle::FrontBottom => 0,
            CameraAngle::BackBottom => 180,
            CameraAngle::RightTop => -90,
            CameraAngle::LeftTop => 90,
            CameraAngle::RightBottom => -90,
            CameraAngle::LeftBottom => 90,

            // Corners
            CameraAngle::FrontRightTop => -45,
            CameraAngle::BackRightTop => -135,
            CameraAngle::BackLeftTop => 135,
            CameraAngle::FrontLeftTop => 45,
            CameraAngle::FrontRightBottom => -45,
            CameraAngle::BackRightBottom => -135,
            CameraAngle::BackLeftBottom => 135,
            CameraAngle::FrontLeftBottom => 45,
        };

        Quaternion::from_angle_x(Deg(x as f32)) * Quaternion::from_angle_y(Deg(y as f32))
    }
}

#[derive(Debug, Clone)]
pub enum ProjectionType {
    Orthographic { height: f32 },
    Perspective { fov_y: Rad<f32> },
}

#[derive(Clone, Debug)]
struct Frustum {
    near: Vector4<f32>,
    far: Vector4<f32>,
    right: Vector4<f32>,
    left: Vector4<f32>,
    top: Vector4<f32>,
    bottom: Vector4<f32>,
}
impl Frustum {
    pub fn zero() -> Self {
        Frustum {
            near: vec4(0.0, 0.0, 0.0, 0.0),
            far: vec4(0.0, 0.0, 0.0, 0.0),
            right: vec4(0.0, 0.0, 0.0, 0.0),
            left: vec4(0.0, 0.0, 0.0, 0.0),
            top: vec4(0.0, 0.0, 0.0, 0.0),
            bottom: vec4(0.0, 0.0, 0.0, 0.0),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Camera {
    viewport_in_pixels: [u32; 2],
    projection_type: ProjectionType,
    near_dist: f32,
    far_dist: f32,
    position: Point3<f32>,
    direction: Vector3<f32>,
    up: Vector3<f32>,
    view_matrix: Matrix4<f32>,
    perspective_matrix: Matrix4<f32>,
    screen_to_ray_matrix: Matrix4<f32>,
    frustum: Frustum,
}
impl Camera {
    pub fn create_orthographic(
        viewport_in_pixels: [u32; 2],
        position: Point3<f32>,
        direction: Vector3<f32>,
        up: Vector3<f32>,
        height: f32,
        near_dist: f32,
        far_dist: f32,
    ) -> Self {
        let mut camera = Camera::create(
            viewport_in_pixels.clone(),
            near_dist,
            far_dist,
            ProjectionType::Orthographic { height },
        );
        camera.orient(position, direction, up);
        camera.update_projection();
        camera
    }

    pub fn projection_matrix(&self) -> Matrix4<f32> {
        self.perspective_matrix() * self.view_matrix()
    }

    pub fn create_perspective(
        viewport_in_pixels: [u32; 2],
        position: Point3<f32>,
        direction: Vector3<f32>,
        up: Vector3<f32>,
        fov_y: Rad<f32>,
        near_dist: f32,
        far_dist: f32,
    ) -> Self {
        let mut camera = Camera::create(
            viewport_in_pixels,
            near_dist,
            far_dist,
            ProjectionType::Perspective { fov_y },
        );
        camera.orient(position, direction, up);
        camera.update_projection();
        camera
    }

    pub fn set_viewport_in_pixels(&mut self, viewport_in_pixels: [u32; 2]) {
        self.viewport_in_pixels = viewport_in_pixels;
        self.update_projection();
    }

    fn update_projection(&mut self) {
        match self.projection_type {
            ProjectionType::Orthographic { height } => {
                let width = height * self.aspect();
                self.perspective_matrix = Self::make_ortho_matrix(
                    -0.5 * width,
                    0.5 * width,
                    -0.5 * height,
                    0.5 * height,
                    self.near_dist,
                    self.far_dist,
                );
            }
            ProjectionType::Perspective { fov_y } => {
                self.perspective_matrix = Self::make_perspective_matrix(
                    fov_y,
                    self.aspect(),
                    self.near_dist,
                    self.far_dist,
                );
            }
        };
        self.update_screen_to_ray_matrix();
        self.update_frustum();
    }

    pub fn orient(&mut self, position: Point3<f32>, direction: Vector3<f32>, up: Vector3<f32>) {
        self.position = position;
        self.direction = direction;
        self.up = up;
        self.view_matrix = Self::make_view_matrix(self.position, self.direction, self.up);
        self.update_screen_to_ray_matrix();
        self.update_frustum();
    }

    fn make_ortho_matrix(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) -> Matrix4<f32> {
        let c0r0 = 2.0 / (right - left);
        let c0r1 = 0.0;
        let c0r2 = 0.0;
        let c0r3 = 0.0;

        let c1r0 = 0.0;
        let c1r1 = 2.0 / (top - bottom);
        let c1r2 = 0.0;
        let c1r3 = 0.0;

        let c2r0 = 0.0;
        let c2r1 = 0.0;
        let c2r2 = 1.0 / (far - near);
        let c2r3 = 0.0;

        let c3r0 = (right + left) / (right - left);
        let c3r1 = (top + bottom) / (top - bottom);
        let c3r2 = -near / (far - near);
        let c3r3 = 1.0;

        #[cfg_attr(rustfmt, rustfmt_skip)]
        Matrix4::new(
            c0r0, c0r1, c0r2, c0r3,
            c1r0, c1r1, c1r2, c1r3,
            c2r0, c2r1, c2r2, c2r3,
            c3r0, c3r1, c3r2, c3r3,
        )
    }

    fn make_perspective_matrix(fov_y: Rad<f32>, aspect: f32, near: f32, far: f32) -> Matrix4<f32> {
        let f = Rad::cot(fov_y / 2.0);

        let c0r0 = f / aspect;
        let c0r1 = 0.0;
        let c0r2 = 0.0;
        let c0r3 = 0.0;

        let c1r0 = 0.0;
        let c1r1 = f;
        let c1r2 = 0.0;
        let c1r3 = 0.0;

        let c2r0 = 0.0;
        let c2r1 = 0.0;
        let c2r2 = 1.0 / (far - near);
        let c2r3 = 1.0;

        let c3r0 = 0.0;
        let c3r1 = 0.0;
        let c3r2 = -near / (far - near);
        let c3r3 = 0.0;

        #[cfg_attr(rustfmt, rustfmt_skip)]
        Matrix4::new(
            c0r0, c0r1, c0r2, c0r3,
            c1r0, c1r1, c1r2, c1r3,
            c2r0, c2r1, c2r2, c2r3,
            c3r0, c3r1, c3r2, c3r3,
        )
    }

    fn make_view_matrix(pos: Point3<f32>, dir: Vector3<f32>, up: Vector3<f32>) -> Matrix4<f32> {
        let f = dir.normalize();
        let s = f.cross(up).normalize();
        let u = f.cross(s);

        #[cfg_attr(rustfmt, rustfmt_skip)]
        Matrix4::new(
            s.x.clone(), u.x.clone(), f.x.clone(), 0.0,
            s.y.clone(), u.y.clone(), f.y.clone(), 0.0,
            s.z.clone(), u.z.clone(), f.z.clone(), 0.0,
            -pos.dot(s), -pos.dot(u), -pos.dot(f), 1.0,
        )
    }

    pub fn vec_to(&self, location: Point3<f32>) -> Vector3<f32> {
        location - self.position
    }

    pub fn viewport_size_at_dist(&self, dist: f32) -> Vector2<f32> {
        match self.projection_type {
            ProjectionType::Orthographic { height } => vec2(height * self.aspect(), height),
            ProjectionType::Perspective { fov_y } => {
                let z_tan_fov = (fov_y / 2.0).tan() * dist * 2.0;
                vec2(z_tan_fov * self.aspect(), z_tan_fov)
            }
        }
    }

    pub fn near_dist(&self) -> f32 {
        self.near_dist
    }

    pub fn far_dist(&self) -> f32 {
        self.far_dist
    }

    pub fn aspect(&self) -> f32 {
        self.viewport_in_pixels[0] as f32 / self.viewport_in_pixels[1] as f32
    }

    pub fn view_matrix(&self) -> &Matrix4<f32> {
        &self.view_matrix
    }

    pub fn perspective_matrix(&self) -> &Matrix4<f32> {
        &self.perspective_matrix
    }

    fn update_screen_to_ray_matrix(&mut self) {
        let mut view_matrix = self.view_matrix;
        view_matrix[3] = vec4(0.0, 0.0, 0.0, 1.0);
        self.screen_to_ray_matrix = (self.perspective_matrix * view_matrix).invert().unwrap();
    }

    fn update_frustum(&mut self) {
        let m = self.perspective_matrix * self.view_matrix;
        self.frustum = Frustum {
            near: vec4(m.x.w + m.x.x, m.y.w + m.y.x, m.z.w + m.z.x, m.w.w + m.w.x),
            far: vec4(m.x.w - m.x.x, m.y.w - m.y.x, m.z.w - m.z.x, m.w.w - m.w.x),
            right: vec4(m.x.w + m.x.y, m.y.w + m.y.y, m.z.w + m.z.y, m.w.w + m.w.y),
            left: vec4(m.x.w - m.x.y, m.y.w - m.y.y, m.z.w - m.z.y, m.w.w - m.w.y),
            top: vec4(m.x.w + m.x.z, m.y.w + m.y.z, m.z.w + m.z.z, m.w.w + m.w.z),
            bottom: vec4(m.x.w - m.x.z, m.y.w - m.y.z, m.z.w - m.z.z, m.w.w - m.w.z),
        };
    }

    fn create(
        viewport_in_pixels: [u32; 2],
        near_dist: f32,
        far_dist: f32,
        projection_type: ProjectionType,
    ) -> Self {
        Camera {
            viewport_in_pixels,
            projection_type,
            near_dist,
            far_dist,
            position: point3(0.0, 0.0, 0.0),
            direction: vec3(0.0, 0.0, 0.0),
            up: vec3(0.0, 0.0, 0.0),
            view_matrix: Matrix4::identity(),
            perspective_matrix: Matrix4::identity(),
            screen_to_ray_matrix: Matrix4::identity(),
            frustum: Frustum::zero(),
        }
    }
}
