
pub enum PopupIcon {
    None,
    Clock,
    CatHead,
    DownArrow
}

pub struct PopupText {
    pub x_pos: f32,
    pub y_pos: f32,
    pub target_x_pos: f32,
    pub target_y_pos: f32,
    pub duration_timer: u32,
    pub text: String,
    pub icon: PopupIcon
}

pub struct PopTextRingbuffer {
    pub texts: [Option<PopupText>; 10],
    pub next_avail_idx: u8
}

impl PopTextRingbuffer {
    pub fn add_new_popup(self: &mut Self, x: f32, y: f32, s: String, icon: PopupIcon) {
        const POPUP_RISE_DIST: f32 = 15.0;
        const POPUP_Y_OFFSET: f32 = -8.0;
        self.texts[self.next_avail_idx as usize] = Some(PopupText {
            x_pos: x,
            y_pos: y + POPUP_Y_OFFSET,
            target_x_pos: x,
            target_y_pos: y + POPUP_Y_OFFSET - POPUP_RISE_DIST,
            duration_timer: 0,
            text: s,
            icon
        });
        self.next_avail_idx += 1;
        self.next_avail_idx %= self.texts.len() as u8;
    }

    pub fn update_popup_positions(self: &mut Self) {
        for popup in self.texts.iter_mut() {
            match popup {
                Some(p) => {
                    const POPUP_TEXT_DURATION: u32 = 100;
                    p.update_position();
                    if p.duration_timer > POPUP_TEXT_DURATION {
                        *popup = None;
                    }
                },
                None => {}
            }
        }
    }
    
}

impl PopupText {
    pub fn update_position(self: &mut Self) {
        const PID_P: f32 = 0.1;

        self.x_pos += PID_P * (self.target_x_pos - self.x_pos);
        self.y_pos += PID_P * (self.target_y_pos - self.y_pos);

        self.duration_timer += 1;
    }
}