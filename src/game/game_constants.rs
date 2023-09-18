// BUILDING PROFILE #1: baseline
// const BUILDING_SUGGESTED_MIN_WIDTH: usize = 8; // 3;
// const BUILDING_SUGGESTED_MAX_WIDTH: usize = 11; // 14;
// const BUILDING_SUGGESTED_MIN_HEIGHT: usize = 2; // 3;
// const BUILDING_SUGGESTED_MAX_HEIGHT: usize = 6; // 12;

// const N_BUILDINGS_PER_CHUNK: usize = 30;
// const USING_DOORS: bool = true;

use core::cell::RefCell;

pub const MAP_CHUNK_MIN_SIDE_LEN: usize = 6;
pub const MAP_CHUNK_MAX_SIDE_LEN: usize = 50;

pub const MAP_CHUNK_MAX_N_TILES: usize = 400;

pub const TOTAL_TILES_IN_MAP: usize = 200 * 1024;

pub const MAX_N_NPCS: usize = 20;
  
pub const TILE_WIDTH_PX: usize = 5;
pub const TILE_HEIGHT_PX: usize = 5;

pub const X_LEFT_BOUND: i32 = -5000;
pub const X_RIGHT_BOUND: i32 = 5000;
pub const Y_LOWER_BOUND: i32 = -5000;
pub const Y_UPPER_BOUND: i32 = 5000;

pub struct OptionsState {
    pub current_selection: u8,
}

pub struct Modal<'a> {
    pub message: &'a str,
    pub options: Vec<&'a str>,
    pub timer: RefCell<u32>,
    pub current_selection: RefCell<u8>
}

pub enum NormalPlayModes<'a> {
    MainGameplay,
    // hover modal is a text, 
    HoverModal(Modal<'a>)
}

pub enum GameMode<'a> {
    StartScreen,
    NormalPlay(NormalPlayModes<'a>),
    Options(OptionsState),
}