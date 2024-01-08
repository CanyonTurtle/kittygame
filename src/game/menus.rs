use core::cell::RefCell;

use super::collision::AbsoluteBoundingBox;

pub enum MenuTypes {
    StartGameMessage,
    StartLevel,
    // Options,
    WonLevel,
    Done,
}

pub struct Modal {
    pub timer: RefCell<u32>,
    pub target_position: RefCell<AbsoluteBoundingBox<i32, u32>>,
    pub actual_position: RefCell<AbsoluteBoundingBox<f32, f32>>,
    pub menu_type: MenuTypes
}

impl Modal {
    pub fn new(target_position: AbsoluteBoundingBox<i32, u32>, menu_type: MenuTypes) -> Modal {
        Modal {
            timer: RefCell::new(0),
            target_position: RefCell::new(target_position),
            actual_position: RefCell::new(AbsoluteBoundingBox{
                x: 0.0, y: 0.0, width: 1.0, height: 1.0
            }),
            menu_type
        }
    }
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


