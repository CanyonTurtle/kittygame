use super::collision::AbsoluteBoundingBox;

pub enum MenuTypes {
    StartGameMessage,
    StartLevel,
    // Options,
    WonLevel,
    Done,
    WonGame,
    Setup,
}

pub struct Modal {
    pub timer: u32,
    pub target_position: AbsoluteBoundingBox<i32, u32>,
    pub actual_position: AbsoluteBoundingBox<f32, f32>,
    pub menu_type: MenuTypes,
    pub should_oscillate: bool,
    pub interactive_delay: u8,
}

impl Modal {
    pub fn new(
        target_position: AbsoluteBoundingBox<i32, u32>,
        menu_type: MenuTypes,
        should_oscillate: bool,
        interactive_delay: Option<u8>,
    ) -> Modal {
        Modal {
            timer: 0,
            target_position,
            actual_position: AbsoluteBoundingBox {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
            menu_type,
            should_oscillate,
            interactive_delay: interactive_delay.unwrap_or(60),
        }
    }

    pub fn ready_to_show_text(&self) -> bool {
        const TOL: f32 = 10.0;
        (self.actual_position.width - self.target_position.width as f32).abs() < TOL
    }

    pub fn text_timer(&self) -> u32 {
        if self.ready_to_show_text() && self.timer >= self.interactive_delay as u32 {
            return self.timer;
        }
        0
    }

    pub fn options_ready_to_select(&self) -> bool {
        if self.ready_to_show_text() && self.timer >= self.interactive_delay as u32 {
            return true;
        }
        false
    }

    fn update_position(&mut self, phase: f32) {
        const SPEED: f32 = 0.15;

        let real_tpy = self.target_position.y
            + if self.should_oscillate {
                (4f32 * num::Float::sin(phase)) as i32
            } else {
                0
            };

        self.actual_position.x += (self.target_position.x as f32 - self.actual_position.x) * SPEED;
        self.actual_position.y += (real_tpy as f32 - self.actual_position.y) * SPEED;
        self.actual_position.width +=
            (self.target_position.width as f32 - self.actual_position.width) * SPEED;
        self.actual_position.height +=
            (self.target_position.height as f32 - self.actual_position.height) * SPEED;
    }

    pub fn update(&mut self, phase: f32) {
        self.update_position(phase);
        self.timer += 1;
    }
}
