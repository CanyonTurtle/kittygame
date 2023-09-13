


// KITTY_SPRITESHEET
pub const KITTY_SPRITESHEET_PALLETE: [u32; 4] = [
    // 0x222034,
    // 0x000000,
    // 0x202020,
    0x000000,
    0xffffff,
    0xff67d3,
    0x99e550,
];

pub const KITTY_SPRITESHEET_DRAW_COLORS: u16 = 0x3420;




pub const KITTY_SPRITESHEET_PAGE_WIDTH: u8 = 32;

pub const KITTY_SPRITESHEET_STRIDE: u8 = KITTY_SPRITESHEET_PAGE_WIDTH * 6;
// KITTY_SPRITESHEET

// kitty_ss
// const KITTY_SS_WIDTH: u32 = 192;
// const KITTY_SS_HEIGHT: u32 = 64;
// const KITTY_SS_FLAGS: u32 = 1; // BLIT_2BPP

pub const KITTY_SPRITESHEET_FLAGS: u32 = 1; // BLIT_2BPP










#[derive(Clone, Copy)]


pub struct SpriteFrame {
    pub width: u8,
    pub height: u8,
    pub start_x: u8,
    pub start_y: u8,
}

pub struct Sprite {
    pub frames: Vec<SpriteFrame>
}

#[allow(dead_code)]
pub enum PresetSprites {
    MainCat,
    Kitty1,
    Kitty2,
    Kitty3,
    Kitty4,
    Pig,
    Lizard,
    BirdIsntReal,
    TopleftSolidCorner,
    ToprightSolidCorner,
    SolidWhite,
    BottomleftSolidCorner,
    BottomrightSolidCorner,
    SeethroughWhite,
    ColumnTop,
    ColumnMiddle,
    ColumnBottom,
    LineLeft,
    LineTop,
    LineRight,
    LineBottom,
}

impl Sprite {
    pub fn from_page_x_y_w_h(spriteframe_indecies: Vec<[u8; 5]>) -> Sprite {
        Sprite {
            frames: spriteframe_indecies.iter().map(|pos| SpriteFrame {
                width: pos[3],
                height: pos[4],
                start_x: pos[0] * KITTY_SPRITESHEET_PAGE_WIDTH + pos[1],
                start_y: pos[2]
            
            }).collect::<Vec<_>>()
        }
    }

    pub fn from_preset(preset_sprite: PresetSprites) -> Sprite {
        match preset_sprite {
            PresetSprites::MainCat => Sprite::from_page_x_y_w_h(vec![
                [0, 16, 57, 14, 7],
                [1, 16, 55, 13, 9],
                [2, 16, 55, 14, 9],
                [3, 16, 50, 11, 14],
                [4, 16, 52, 6, 12],
                [5, 16, 54, 12, 10]
            ]),
            PresetSprites::Kitty1 => Sprite::from_page_x_y_w_h(vec![
                [0, 0, 43, 9, 5],
                [1, 0, 42, 10, 6],
                [2, 0, 41, 9, 7],
                [3, 0, 38, 9, 10],
                [4, 0, 39, 6, 9],
                [5, 0, 41, 7, 7],
            ]),
            PresetSprites::Kitty2 => Sprite::from_page_x_y_w_h(vec![
                [0, 0, 52, 9, 4],
                [1, 0, 51, 9, 5],
                [2, 0, 51, 9, 5],
                [3, 0, 48, 10, 8],
                [4, 0, 48, 5, 8],
                [5, 0, 50, 8, 7]
            ]),
            PresetSprites::Kitty3 => Sprite::from_page_x_y_w_h(vec![
                [0, 0, 60, 10, 4],
                [1, 0, 59, 10, 5],
                [2, 0, 59, 10, 5],
                [3, 0, 57, 10, 8],
                [4, 0, 56, 5, 8],
                [5, 0, 57, 8, 7]
            ]),
            PresetSprites::Kitty4 => Sprite::from_page_x_y_w_h(vec![
                [0, 16, 44, 10, 4],
                [1, 16, 43, 10, 5],
                [2, 16, 43, 10, 5],
                [3, 16, 40, 10, 8],
                [4, 16, 40, 5, 8],
                [5, 16, 41, 8, 7]
            ]),
            PresetSprites::Pig => Sprite::from_page_x_y_w_h(vec![
                [0, 16, 35, 8, 5],
                [1, 16, 35, 8, 5],
                [0, 16, 35, 8, 5],
                [1, 16, 35, 8, 5],
                [0, 16, 35, 8, 5],
                [1, 16, 35, 8, 5],
            ]),
            PresetSprites::Lizard => Sprite::from_page_x_y_w_h(vec![
                [0, 16, 23, 10, 9],
                [1, 16, 24, 10, 8],
                [0, 16, 23, 10, 9],
                [1, 16, 24, 10, 8],
                [0, 16, 23, 10, 9],
                [1, 16, 24, 10, 8]
            ]),
            PresetSprites::BirdIsntReal => Sprite::from_page_x_y_w_h(vec![
                [0, 16, 9, 10, 7],
                [1, 16, 8, 10, 8],
                [2, 16, 10, 10, 6],
                [1, 16, 8, 10, 8],
                [2, 16, 10, 10, 6],
                [1, 16, 8, 10, 8],
            ]),
            PresetSprites::TopleftSolidCorner => Sprite::from_page_x_y_w_h(vec![[0, 0, 0, 5, 5]]),
            PresetSprites::SolidWhite => Sprite::from_page_x_y_w_h(vec![[0, 5, 0, 5, 5]]),
            PresetSprites::ToprightSolidCorner => Sprite::from_page_x_y_w_h(vec![[0, 10, 0, 5, 5]]),
            PresetSprites::BottomleftSolidCorner => Sprite::from_page_x_y_w_h(vec![[0, 5, 0, 5, 5]]),
            PresetSprites::SeethroughWhite => Sprite::from_page_x_y_w_h(vec![[0, 5, 5, 5, 5]]),
            PresetSprites::BottomrightSolidCorner => Sprite::from_page_x_y_w_h(vec![[0, 10, 5, 5, 5]]),
            PresetSprites::ColumnTop => Sprite::from_page_x_y_w_h(vec![[0, 0, 10, 5, 5]]),
            PresetSprites::ColumnMiddle => Sprite::from_page_x_y_w_h(vec![[0, 0, 15, 5, 5]]),
            PresetSprites::ColumnBottom => Sprite::from_page_x_y_w_h(vec![[0, 0, 20, 5, 5]]),
            PresetSprites::LineLeft => Sprite::from_page_x_y_w_h(vec![[0, 5, 10, 5, 5]]),
            PresetSprites::LineTop => Sprite::from_page_x_y_w_h(vec![[0, 10, 10, 5, 5]]),
            PresetSprites::LineRight => Sprite::from_page_x_y_w_h(vec![[0, 5, 15, 5, 5]]),
            PresetSprites::LineBottom => Sprite::from_page_x_y_w_h(vec![[0, 10, 15, 5, 5]]),
        }
    }
}
