


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


pub const KITTY_TILE_SIZE: usize = 8;

pub const KITTY_SPRITESHEET_STRIDE: usize = 320;

pub const KITTY_SPRITESHEET_PAGE_WIDTH: usize = 64;
// KITTY_SPRITESHEET


pub const KITTY_SPRITESHEET_FLAGS: u32 = 1; // BLIT_2BPP










#[derive(Clone, Copy)]


pub struct SpriteFrameInternalPositioning {
    pub width: u8,
    pub height: u8,
    pub start_x: usize,
    pub start_y: usize,
}

pub struct SpriteFrame {
    pub positioning: SpriteFrameInternalPositioning,
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
    pub fn new(spriteframe_indecies: Vec<[usize; 4]>) -> Sprite {
        Sprite {
            frames: spriteframe_indecies.iter().map(|&x| SpriteFrame {
                positioning: SpriteFrameInternalPositioning { width: x[0] as u8, height: x[1] as u8, start_x: x[2], start_y: x[3] }
            }).collect::<Vec<_>>()
        }
    }

    pub fn from_page_tilei_tilej(spriteframe_indecies: Vec<[u8; 7]>) -> Sprite {
        Sprite::new(spriteframe_indecies.iter().map(|&x| [x[3] as usize, x[4] as usize, KITTY_SPRITESHEET_PAGE_WIDTH * x[0] as usize + KITTY_TILE_SIZE * x[1] as usize + x[5] as usize, KITTY_TILE_SIZE * x[2] as usize + x[6] as usize]).collect())
    }

    pub fn from_preset(preset_sprite: PresetSprites) -> Sprite {
        match preset_sprite {
            PresetSprites::MainCat => Sprite::from_page_tilei_tilej(vec![
                [1, 0, 6, 14, 7, 1, 4],
                [2, 0, 6, 13, 9, 2, 2],
                [3, 0, 6, 14, 9, 1, 2],
                [4, 0, 6, 13, 13, 2, 0],
                [4, 2, 6, 5, 13, 11, 1]
            ]),
            PresetSprites::Kitty1 => Sprite::from_page_tilei_tilej(vec![
                // page, tile_i, tile_j, width, height, x_sub_tile, y_sub_tile
                [1, 0, 0, 10, 5, 2, 2],
                [2, 0, 0, 10, 5, 2, 2],
                [3, 0, 0, 10, 5, 2, 2],
                [4, 0, 0, 10, 5, 2, 2],
                [4, 0, 0, 10, 5, 2, 2],
            ]),
            PresetSprites::Kitty2 => Sprite::from_page_tilei_tilej(vec![
                [1, 2, 0, 10, 5, 1, 2],
                [2, 2, 0, 10, 5, 1, 2],
                [3, 2, 0, 10, 5, 1, 2],
                [4, 2, 0, 10, 7, 1, 1],
                [4, 2, 0, 10, 7, 1, 1],
            ]),
            PresetSprites::Kitty3 => Sprite::from_page_tilei_tilej(vec![
                [1, 0, 2, 10, 5, 4, 4],
                [2, 0, 2, 10, 5, 4, 4],
                [3, 0, 2, 10, 5, 4, 4],
                [4, 0, 2, 10, 7, 4, 2],
                [4, 0, 2, 10, 7, 4, 2],
            ]),
            PresetSprites::Kitty4 => Sprite::from_page_tilei_tilej(vec![
                [1, 2, 2, 10, 6, 2, 3],
                [2, 2, 2, 10, 6, 2, 3],
                [3, 2, 2, 10, 6, 2, 3],
                [4, 2, 2, 10, 10, 2, 0],
                [4, 2, 2, 10, 10, 2, 0],
            ]),
            PresetSprites::Pig => Sprite::from_page_tilei_tilej(vec![
                [1, 0, 4, 8, 5, 4, 4],
                [2, 0, 4, 8, 5, 4, 4],
                [3, 0, 4, 8, 5, 4, 4],
                [2, 0, 4, 8, 5, 4, 4],
                [2, 0, 4, 8, 5, 4, 4],
            ]),
            PresetSprites::Lizard => Sprite::from_page_tilei_tilej(vec![
                [3, 2, 4, 10, 9, 3, 2],
                [2, 2, 4, 10, 9, 3, 2],
                [2, 2, 4, 10, 9, 3, 2],
                [2, 2, 4, 10, 9, 3, 2],
                [2, 2, 4, 10, 9, 3, 2],
            ]),
            PresetSprites::BirdIsntReal => Sprite::from_page_tilei_tilej(vec![
                [2, 2, 6, 10, 7, 1, 2],
                [3, 2, 6, 10, 8, 1, 2],
                [3, 2, 6, 10, 6, 1, 3],
                [3, 2, 6, 10, 8, 1, 2],
                [3, 2, 6, 10, 8, 1, 2],
            ]),
            PresetSprites::TopleftSolidCorner => Sprite::from_page_tilei_tilej(vec![[0, 5, 0, 5, 5, 0, 0]]),
            PresetSprites::SolidWhite => Sprite::from_page_tilei_tilej(vec![[0, 6, 0, 5, 5, 0, 0]]),
            PresetSprites::ToprightSolidCorner => Sprite::from_page_tilei_tilej(vec![[0, 7, 0, 5, 5, 0, 0]]),
            PresetSprites::BottomleftSolidCorner => Sprite::from_page_tilei_tilej(vec![[0, 5, 1, 5, 5, 0, 0]]),
            PresetSprites::SeethroughWhite => Sprite::from_page_tilei_tilej(vec![[0, 6, 1, 5, 5, 0, 0]]),
            PresetSprites::BottomrightSolidCorner => Sprite::from_page_tilei_tilej(vec![[0, 7, 1, 5, 5, 0, 0]]),
            PresetSprites::ColumnTop => Sprite::from_page_tilei_tilej(vec![[0, 5, 2, 5, 5, 0, 0]]),
            PresetSprites::ColumnMiddle => Sprite::from_page_tilei_tilej(vec![[0, 5, 1, 5, 5, 0, 0]]),
            PresetSprites::ColumnBottom => Sprite::from_page_tilei_tilej(vec![[0, 5, 2, 5, 5, 0, 0]]),
            PresetSprites::LineLeft => Sprite::from_page_tilei_tilej(vec![[0, 6, 2, 5, 5, 0, 0]]),
            PresetSprites::LineTop => Sprite::from_page_tilei_tilej(vec![[0, 7, 2, 5, 5, 0, 0]]),
            PresetSprites::LineRight => Sprite::from_page_tilei_tilej(vec![[0, 6, 3, 5, 5, 0, 0]]),
            PresetSprites::LineBottom => Sprite::from_page_tilei_tilej(vec![[0, 7, 3, 5, 5, 0, 0]]),
        }
    }
}
