use core::cell::RefCell;

use super::collision::AbsoluteBoundingBox;

pub enum MenuTypes {
    StartGameMessage,
    Options,
    WonLevel,
    Done,
}

pub struct Modal {
    pub n_options: u8,
    pub timer: RefCell<u32>,
    pub current_selection: RefCell<u8>,
    pub target_position: RefCell<AbsoluteBoundingBox<i32, u32>>,
    pub actual_position: RefCell<AbsoluteBoundingBox<f32, f32>>,
    pub menu_type: MenuTypes
}

pub enum NormalPlayModes {
    MainGameplay,
    // hover modal is a text, 
    HoverModal(Modal)
}

pub enum GameMode {
    StartScreen,
    NormalPlay(NormalPlayModes),
}

