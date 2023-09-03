use std::cell::RefCell;

use crate::{spritesheet, create_map};

use super::{entities::{OptionallyEnabledPlayer, Character}, game_map::GameMap, camera::Camera, rng::Rng, game_constants::{GameMode, N_NPCS}};

pub struct GameState<'a> {
    pub players: RefCell<[OptionallyEnabledPlayer; 4]>,
    pub npcs: RefCell<Vec<Character>>,
    pub spritesheet: &'a [u8],
    pub spritesheet_stride: usize,
    pub background_tiles: Vec<spritesheet::Sprite>,
    pub map: GameMap,
    pub camera: RefCell<Camera>,
    pub rng: RefCell<Rng>,
    pub game_mode: GameMode,
}


impl GameState<'static> {
    pub fn new() -> GameState<'static> {

        let characters = [
            OptionallyEnabledPlayer::Enabled(Character::new(spritesheet::PresetSprites::MainCat)),
            OptionallyEnabledPlayer::Enabled(Character::new(spritesheet::PresetSprites::MainCat)),
            OptionallyEnabledPlayer::Enabled(Character::new(spritesheet::PresetSprites::MainCat)),
            OptionallyEnabledPlayer::Enabled(Character::new(spritesheet::PresetSprites::MainCat)),
        ];

        let rng = Rng::new();
        GameState {
            players: RefCell::new(characters),
            npcs: RefCell::new((0..N_NPCS).map(|mut x| {
                x %= 7;
                let preset = match x {
                    0 => spritesheet::PresetSprites::Kitty1,
                    1 => spritesheet::PresetSprites::Kitty2,
                    2 => spritesheet::PresetSprites::Kitty3,
                    3 => spritesheet::PresetSprites::Kitty4,
                    4 => spritesheet::PresetSprites::Lizard,
                    5 => spritesheet::PresetSprites::Pig,
                    6 => spritesheet::PresetSprites::BirdIsntReal,
                    _ => spritesheet::PresetSprites::Pig
                };
                Character::new(preset)
            }).collect::<Vec<Character>>()),
            // npcs: vec![
            //     Character::new(500, spritesheet::PresetSprites::Kitty2),
            //     Character::new(400, spritesheet::PresetSprites::Kitty3),
            //     Character::new(300, spritesheet::PresetSprites::Kitty4),
            //     Character::new(200, spritesheet::PresetSprites::Pig),
            //     Character::new(100, spritesheet::PresetSprites::Lizard),
            // ],
            spritesheet: &spritesheet::KITTY_SS,
            spritesheet_stride: spritesheet::KITTY_SS_STRIDE,
            background_tiles: vec![
                spritesheet::Sprite::from_preset(spritesheet::PresetSprites::LineTop),
                spritesheet::Sprite::from_preset(spritesheet::PresetSprites::LineLeft),
                spritesheet::Sprite::from_preset(spritesheet::PresetSprites::LineRight),
                spritesheet::Sprite::from_preset(spritesheet::PresetSprites::LineBottom),
                spritesheet::Sprite::from_preset(spritesheet::PresetSprites::SolidWhite),
                spritesheet::Sprite::from_preset(spritesheet::PresetSprites::SeethroughWhite),
                spritesheet::Sprite::from_preset(spritesheet::PresetSprites::TopleftSolidCorner),
                spritesheet::Sprite::from_preset(spritesheet::PresetSprites::ToprightSolidCorner),
                spritesheet::Sprite::from_preset(spritesheet::PresetSprites::BottomleftSolidCorner),
                spritesheet::Sprite::from_preset(spritesheet::PresetSprites::BottomrightSolidCorner),
                spritesheet::Sprite::from_preset(spritesheet::PresetSprites::ColumnTop),
                spritesheet::Sprite::from_preset(spritesheet::PresetSprites::ColumnMiddle),
                spritesheet::Sprite::from_preset(spritesheet::PresetSprites::ColumnBottom),
            ],
            map: create_map(),
            camera: RefCell::new(Camera { current_viewing_x_offset: 0.0, current_viewing_y_offset: 0.0 }),
            rng: RefCell::new(rng),
            game_mode: GameMode::StartScreen
        }
    }
}
