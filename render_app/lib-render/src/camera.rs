use glam::{Mat4, Vec2, Vec3};
use winit::dpi::PhysicalSize;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Camera2DUniform {
    pub(crate) view_proj: [[f32; 4]; 4],
}

#[derive(Debug)]
pub(crate) struct Camera2D {
    position: Vec2,
    screen_size: PhysicalSize<u32>,
}

impl Camera2D {
    pub(crate) fn new(screen_width: u32, screen_height: u32) -> Self {
        Self { 
            position: Vec2::new(0.0, 0.0), 
            screen_size: PhysicalSize::new(screen_width, screen_height),
        }
    }

    fn build_projection_matrix(&self) -> Mat4 {
        let width = self.screen_size.width as f32;
        let height = self.screen_size.height as f32;

        Mat4::orthographic_rh_gl(
            0.0,
            width,
            height,
            0.0,
            -1.0,
            1.0,
        )
    }

    fn build_view_matrix(&self) -> Mat4 {
        Mat4::from_translation(Vec3::new(-self.position.x, -self.position.y, 0.0))
    }

    pub(crate) fn build_view_projection_matrix(&self) -> Mat4 {
        let view_proj = self.build_projection_matrix() * self.build_view_matrix();
        view_proj
    }

    pub(crate) fn update_screen_size(&mut self, new_size: PhysicalSize<u32>) {
        self.screen_size = new_size;
    }
}

// 3D camera uniform, the same as the 2D one
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Camera3DUniform {
    pub(crate) view_proj: [[f32; 4]; 4],
}

pub(crate) struct Camera3D {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov_y: f32,
    pub aspect: f32,
    pub z_near: f32,
    pub z_far: f32,
}

impl Camera3D {
    pub fn new(screen_size: PhysicalSize<u32>) -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 5.0),
            target: Vec3::new(0.0, 0.0, 0.0),
            up: Vec3::Y,
            fov_y: 45.0,
            aspect: screen_size.width as f32 / screen_size.height as f32,
            z_near: 0.1,
            z_far: 100.0,
        }
    }

    pub fn build_view_projection_matrix(&self) -> Mat4 {
        let view = Mat4::look_at_rh(self.position, self.target, self.up);
        let proj = Mat4::perspective_rh_gl(self.fov_y, self.aspect, self.z_near, self.z_far);
        proj * view
    }

    pub fn update_screen_size(&mut self, new_size: PhysicalSize<u32>) {
        self.aspect = new_size.width as f32 / new_size.height as f32;
    }
}