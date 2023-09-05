
#[derive(Clone, Copy)]

pub struct Camera {
    pub current_viewing_x_offset: f32,
    pub current_viewing_y_offset: f32,

    pub current_viewing_x_target: f32,
    pub current_viewing_y_target: f32,
}

impl Camera {
    pub fn slew(self: &mut Self) {
        let x_err = self.current_viewing_x_target - self.current_viewing_x_offset;
        let y_err = self.current_viewing_y_target - self.current_viewing_y_offset;

        const KP: f32 = 0.3;
        self.current_viewing_x_offset += KP * x_err;
        self.current_viewing_y_offset += KP * y_err;
    }
}