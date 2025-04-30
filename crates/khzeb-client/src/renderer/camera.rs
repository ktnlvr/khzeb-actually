use glam::{Mat4, Vec2};

pub struct Camera {
    pub position: Vec2,
    pub aspect_ratio: f32,
    pub size: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            aspect_ratio: 16. / 9.,
            size: 7.,
            position: Vec2 { x: 0., y: 0. },
        }
    }
}

impl Camera {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bake(&self) -> Mat4 {
        let half_width = 0.5 * self.size;
        let half_height = half_width / self.aspect_ratio;

        let left = self.position.x - half_width;
        let right = self.position.x + half_width;
        let bottom = self.position.y - half_height;
        let top = self.position.y + half_height;

        let ortho = Mat4::orthographic_rh_gl(left, right, bottom, top, -1.0, 1.0);
        ortho
    }
}
