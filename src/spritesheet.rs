// KITTY_SPRITESHEET

pub const KITTY_SPRITESHEET_DRAW_COLORS: u16 = 0x3420;

pub const KITTY_SPRITESHEET_PAGE_WIDTH: u8 = 32;

pub const KITTY_SPRITESHEET_STRIDE: u8 = KITTY_SPRITESHEET_PAGE_WIDTH * 6;

pub const KITTY_SPRITESHEET_PALETTES: [[u32; 4]; 9] = [
    [
        // Teal & Gold Palette
        0x57dab2, // pig (Cool Teal)

        0xe4f288, // background (Cool Teal)
        0xdff6633, // lizard (Warm Goldenrod)

        0xf8ffd2, // foreground (Warm Wheat)
        // Teal & Gold Palette
        // 0x8adab2, // pig (Cool Teal)

        // 0xd0f288, // background (Cool Teal)
        // 0xdf826c, // lizard (Warm Goldenrod)

        // 0xf8ffd2, // foreground (Warm Wheat)
    ],
    [
        // Pinky
        0x4B0082, // Indigo
        0xFF69B4, // Hot Pink
        0xFFA500, // Orange
        0x90EE90, // Light Green
    ],
    [
        // Midnight Serenade
        0x050540, // Midnight Blue
        0xFFFFFF, // White
        0xFF3311, // Red
        0xf6f666, // Gold
    ],
    [
        // red and golden
        0x440000, // Dark Red
        0xFFF7D9, // Cream
        0xFF6347, // Tomato
        0x99E64E, // Light Green
    ],
    [
        // Pine & Rose Palette
        0x0b4221, // background (Dark Pine Green)
        0xffaac8, // foreground (Light Rose)
        0xff69b4, // pig (Cool Pink)
        0x00ff88, // lizard
    ],
    [
        // Maroon & Turquoise Palette
        0x800000, // background (Warm Maroon)
        0x40e0d0, // foreground (Cool Turquoise)
        0xff4500, // pig (Warm Red-Orange)
        0x00f080, // lizard (Cool Teal)
    ],
    [
        // Slate & Peach Palette
        0x3a4c52, // background (Dark Slate Blue)
        0xffdab9, // foreground (Light Peach)
        0xff69b4, // pig (Cool Pink)
        0x00ffaa, // lizard
    ],
    [
        // Spring Bloom Palette
        0x56b88c, // Bright Blue-Green (Ball/Overlay/Smoke)
        0xceedc7, // Light Orange (Background)
        0xffd4b2, // Medium Orange-Pink (Alternate Text)
        0xfff6bd, // Bright Blue-Green (Paddles)
    ],
    [
        // very cool: Black, Orange Red, Gold, Dark Turquoise
        0x000000, // bg: black
        0xFF4500, // fg: red
        0xFFD700, // pig: yellow
        0x00CED1, // lizard: teal
    ],
    // [
    //     // RGB
    //     0x000000, // Black
    //     0xFFFFFF, // White
    //     0xFF0000, // R
    //     0x00FF00, // G
    // ],
];

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
    pub frames: Vec<SpriteFrame>,
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
    Left,
    Top,
    Right,
    Bottom,
    KittyCard,
    PiggyCard,
    LizardCard,
    BirdCard,
    Cloud,
    Clock,
    CatHead,
}

static mut SPRITES: Option<Vec<Sprite>> = None;

impl Sprite {
    pub fn init_all_sprites() {
        unsafe {
            match &mut SPRITES {
                Some(_) => {
                    unreachable!();
                }
                None => {
                    SPRITES = Some(Vec::with_capacity(30));
                }
            }
        };

        let the_sprites: &mut Vec<Sprite>;

        unsafe {
            match &mut SPRITES {
                Some(s) => {
                    the_sprites = s;
                }
                None => {
                    unreachable!()
                }
            }
        }

        //  ---------- TILESET ------------

        // 0: main kitty
        the_sprites.push(Sprite::from_page_x_y_w_h(vec![
            [0, 16, 55, 14, 9],
            [1, 16, 55, 13, 9],
            [2, 16, 55, 14, 9],
            [3, 16, 50, 11, 14],
            [4, 16, 52, 6, 12],
            [5, 16, 54, 12, 10],
        ]));

        // 1: lil kitty 1
        the_sprites.push(Sprite::from_page_x_y_w_h(vec![
            [0, 0, 43, 9, 5],
            [1, 0, 42, 10, 6],
            [2, 0, 41, 9, 7],
            [3, 0, 38, 9, 10],
            [4, 0, 39, 6, 9],
            [5, 0, 41, 7, 7],
        ]));

        // 2: lil kitty 2
        the_sprites.push(Sprite::from_page_x_y_w_h(vec![
            [0, 0, 52, 9, 4],
            [1, 0, 51, 9, 5],
            [2, 0, 51, 9, 5],
            [3, 0, 48, 10, 8],
            [4, 0, 48, 5, 8],
            [5, 0, 50, 8, 7],
        ]));

        // 3: lil kitty 3
        the_sprites.push(Sprite::from_page_x_y_w_h(vec![
            [0, 0, 60, 10, 4],
            [1, 0, 59, 10, 5],
            [2, 0, 59, 10, 5],
            [3, 0, 57, 10, 7],
            [4, 0, 56, 5, 8],
            [5, 0, 57, 8, 7],
        ]));

        // 4: lil kitty 4
        the_sprites.push(Sprite::from_page_x_y_w_h(vec![
            [0, 16, 44, 10, 4],
            [1, 16, 43, 10, 5],
            [2, 16, 43, 10, 5],
            [3, 16, 40, 10, 8],
            [4, 16, 40, 5, 8],
            [5, 16, 41, 8, 7],
        ]));

        // 5: pig
        the_sprites.push(Sprite::from_page_x_y_w_h(vec![
            [0, 16, 35, 8, 5],
            [1, 16, 35, 8, 5],
            [0, 16, 35, 8, 5],
            [1, 16, 35, 8, 5],
            [0, 16, 35, 8, 5],
            [1, 16, 35, 8, 5],
        ]));

        // 6: lizard
        the_sprites.push(Sprite::from_page_x_y_w_h(vec![
            [0, 16, 23, 10, 9],
            [1, 16, 24, 10, 8],
            [0, 16, 23, 10, 9],
            [1, 16, 24, 10, 8],
            [0, 16, 23, 10, 9],
            [1, 16, 24, 10, 8],
        ]));

        // 7: bird
        the_sprites.push(Sprite::from_page_x_y_w_h(vec![
            [0, 16, 9, 10, 7],
            [1, 16, 8, 10, 8],
            [2, 16, 10, 10, 6],
            [1, 16, 8, 10, 8],
            [2, 16, 10, 10, 6],
            [1, 16, 8, 10, 8],
        ]));

        // -------- tiles ---------

        // --- filled tiles ---

        // 8: top left corner
        the_sprites.push(Sprite::from_page_x_y_w_h(vec![[0, 0, 0, 5, 5]]));

        // 9: solid block
        the_sprites.push(Sprite::from_page_x_y_w_h(vec![[0, 5, 0, 5, 5]]));

        // 10: top right corner
        the_sprites.push(Sprite::from_page_x_y_w_h(vec![[0, 10, 0, 5, 5]]));

        // 11: bottom left corner
        the_sprites.push(Sprite::from_page_x_y_w_h(vec![[0, 0, 5, 5, 5]]));

        // 12: see through bubble block
        the_sprites.push(Sprite::from_page_x_y_w_h(vec![[0, 5, 5, 5, 5]]));

        // 13: bottom right corner
        the_sprites.push(Sprite::from_page_x_y_w_h(vec![[0, 10, 5, 5, 5]]));

        // --- Columns ---

        // 14: column top
        the_sprites.push(Sprite::from_page_x_y_w_h(vec![[0, 0, 10, 5, 5]]));

        // 15: column middle
        the_sprites.push(Sprite::from_page_x_y_w_h(vec![[0, 0, 15, 5, 5]]));

        // 16: column bottom
        the_sprites.push(Sprite::from_page_x_y_w_h(vec![[0, 0, 20, 5, 5]]));

        // --- The regular Line wall sprites -----

        // 17: left
        the_sprites.push(Sprite::from_page_x_y_w_h(vec![[0, 5, 10, 5, 5]]));

        // 18: bottom
        the_sprites.push(Sprite::from_page_x_y_w_h(vec![[0, 10, 10, 5, 5]]));

        // 19: top
        the_sprites.push(Sprite::from_page_x_y_w_h(vec![[0, 5, 15, 5, 5]]));

        // 20: right
        the_sprites.push(Sprite::from_page_x_y_w_h(vec![[0, 10, 15, 5, 5]]));

        // 21: kitty card
        the_sprites.push(Sprite::from_page_x_y_w_h(vec![[3, 0, 0, 12, 12]]));

        // 22: pig card
        the_sprites.push(Sprite::from_page_x_y_w_h(vec![[3, 12, 0, 12, 12]]));

        // 23: lizard card
        the_sprites.push(Sprite::from_page_x_y_w_h(vec![[3, 0, 12, 12, 12]]));

        // 24: bird card
        the_sprites.push(Sprite::from_page_x_y_w_h(vec![[3, 12, 12, 12, 12]]));

        // ------- SOLID TILESET ----------

        // 25: solid tileset top left
        the_sprites.push(Sprite::from_page_x_y_w_h(vec![[0, 0, 25, 5, 5]]));

        // 26: solid block
        the_sprites.push(Sprite::from_page_x_y_w_h(vec![[0, 5, 25, 5, 5]]));

        // 27: solid top right
        the_sprites.push(Sprite::from_page_x_y_w_h(vec![[0, 10, 25, 5, 5]]));

        // 28: bottom left
        the_sprites.push(Sprite::from_page_x_y_w_h(vec![[0, 0, 30, 5, 5]]));

        // 29: bottom right
        the_sprites.push(Sprite::from_page_x_y_w_h(vec![[0, 10, 30, 5, 5]]));

        // 30: complete solid
        the_sprites.push(Sprite::from_page_x_y_w_h(vec![[0, 0, 35, 5, 5]]));

        // 31: scraggle
        the_sprites.push(Sprite::from_page_x_y_w_h(vec![[0, 5, 35, 5, 5]]));

        // 32: checker
        the_sprites.push(Sprite::from_page_x_y_w_h(vec![[0, 10, 35, 5, 5]]));

        // 33: cloud
        the_sprites.push(Sprite::from_page_x_y_w_h(vec![[1, 0, 0, 6, 3]]));

        // 34: clock
        the_sprites.push(Sprite::from_page_x_y_w_h(vec![[0, 17, 0, 7, 8]]));

        // 35: cat head
        the_sprites.push(Sprite::from_page_x_y_w_h(vec![[3, 3, 3, 6, 6]]));
    }

    pub fn get_spritesheet() -> &'static Vec<Sprite> {
        unsafe {
            match &SPRITES {
                Some(s) => s,
                None => {
                    unreachable!()
                }
            }
        }
    }

    pub fn from_page_x_y_w_h(spriteframe_indecies: Vec<[u8; 5]>) -> Sprite {
        Sprite {
            frames: spriteframe_indecies
                .iter()
                .map(|pos| SpriteFrame {
                    width: pos[3],
                    height: pos[4],
                    start_x: pos[0] * KITTY_SPRITESHEET_PAGE_WIDTH + pos[1],
                    start_y: pos[2],
                })
                .collect::<Vec<_>>(),
        }
    }

    // pub fn from_idx(idx: usize) -> &'static Sprite {
    //     let sprites_vec;
    //     unsafe {
    //         match &mut SPRITES {
    //             Some(s) => {
    //                 sprites_vec = s;
    //             },
    //             None => unreachable!()
    //         }
    //     }
    //     &sprites_vec[idx]
    // }

    pub fn from_preset(preset_sprite: &PresetSprites) -> &'static Sprite {
        let sprites_vec;
        unsafe {
            match &mut SPRITES {
                Some(s) => {
                    sprites_vec = s;
                }
                None => unreachable!(),
            }
        }
        match preset_sprite {
            PresetSprites::MainCat => &sprites_vec[0],
            PresetSprites::Kitty1 => &sprites_vec[1],
            PresetSprites::Kitty2 => &sprites_vec[2],
            PresetSprites::Kitty3 => &sprites_vec[3],
            PresetSprites::Kitty4 => &sprites_vec[4],
            PresetSprites::Pig => &sprites_vec[5],
            PresetSprites::Lizard => &sprites_vec[6],
            PresetSprites::BirdIsntReal => &sprites_vec[7],
            PresetSprites::TopleftSolidCorner => &sprites_vec[8],
            PresetSprites::SolidWhite => &sprites_vec[9],
            PresetSprites::ToprightSolidCorner => &sprites_vec[10],
            PresetSprites::BottomleftSolidCorner => &sprites_vec[11],
            PresetSprites::SeethroughWhite => &sprites_vec[12],
            PresetSprites::BottomrightSolidCorner => &sprites_vec[13],
            PresetSprites::ColumnTop => &sprites_vec[14],
            PresetSprites::ColumnMiddle => &sprites_vec[15],
            PresetSprites::ColumnBottom => &sprites_vec[16],
            PresetSprites::Left => &sprites_vec[17],
            PresetSprites::Top => &sprites_vec[18],
            PresetSprites::Right => &sprites_vec[19],
            PresetSprites::Bottom => &sprites_vec[20],
            PresetSprites::KittyCard => &sprites_vec[21],
            PresetSprites::PiggyCard => &sprites_vec[22],
            PresetSprites::LizardCard => &sprites_vec[23],
            PresetSprites::BirdCard => &sprites_vec[24],
            PresetSprites::Cloud => &sprites_vec[33],
            PresetSprites::Clock => &sprites_vec[34],
            PresetSprites::CatHead => &sprites_vec[35],
        }
    }
}
