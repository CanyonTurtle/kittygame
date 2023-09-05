use crate::spritesheet;

pub enum OptionallyEnabledPlayer {
    Enabled(Character),
    Disabled
}

pub enum MovingEntity<'a> {
    OptionalPlayer(&'a mut OptionallyEnabledPlayer),
    NPC(&'a mut Character)
}

pub struct Character {
    pub x_pos: f32,
    pub y_pos: f32,
    pub x_vel: f32,
    pub y_vel: f32,
    pub x_vel_cap: f32,
    pub y_vel_cap: f32,
    pub count: i32,
    pub facing_right: bool,
    pub state: KittyStates,
    pub current_sprite_i: i32,
    pub sprite: spritesheet::Sprite,
}

#[derive(PartialEq, Eq, Hash)]



pub enum KittyStates {
    Sleeping,
    Walking(u8),
    JumpingUp(u8),
    HuggingWall(bool),
}



impl Character {
    pub fn new(sprite_type: spritesheet::PresetSprites) -> Character {
        Character {
            x_pos: 10 as f32,
            y_pos: 10.0,
            x_vel: 0.0,
            y_vel: 0.0,
            x_vel_cap: 2.0,
            y_vel_cap: 7.0,
            count: 0,
            facing_right: true,
            state: KittyStates::JumpingUp(200),
            current_sprite_i: 0,
            sprite: spritesheet::Sprite::from_preset(sprite_type),
        }
    }
}