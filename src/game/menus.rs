use super::collision::AbsoluteBoundingBox;

pub enum MenuTypes {
    StartGameMessage,
    StartLevel,
    // Options,
    WonLevel,
    Done,
    WonGame
}

pub struct Modal {
    pub timer: u32,
    pub target_position: AbsoluteBoundingBox<i32, u32>,
    pub actual_position: AbsoluteBoundingBox<f32, f32>,
    pub menu_type: MenuTypes
}

impl Modal {
    pub fn new(target_position: AbsoluteBoundingBox<i32, u32>, menu_type: MenuTypes) -> Modal {
        Modal {
            timer: 0,
            target_position: target_position,
            actual_position: AbsoluteBoundingBox{
                x: 0.0, y: 0.0, width: 1.0, height: 1.0
            },
            menu_type
        }
    }
}

pub enum NormalPlayModes {
    MainGameplay,
    // hover modal is a text, 
    HoverModal(Modal)
}

pub enum SelectMenuFocuses {
    // Difficulty,
    RunType,
    // CharacterSelect,
    // StartGameBtn
}

pub struct SelectSetup{
    pub current_selection: SelectMenuFocuses
}

pub enum GameMode {
    StartScreen,
    NormalPlay(NormalPlayModes),
    SelectScreen(SelectSetup)
}


