use crate::spritesheet::{self, PresetSprites};

use super::ability_cards::AbilityCardStack;

pub struct Player {
    pub character: Character,
    pub card_stack: AbilityCardStack,
}

pub enum OptionallyEnabledPlayer {
    Enabled(Player),
    Disabled
}

pub enum MovingEntity<'a> {
    OptionalPlayer(&'a mut OptionallyEnabledPlayer),
    NPC(&'a mut Character)
}
 
// If a player can warp, they need to hold the button long enough.
type WarpInputTimer = u8;

#[derive(PartialEq, Eq, Hash)]
pub enum WarpState {
    Charging(WarpInputTimer),
    Ready,
}

#[derive(PartialEq, Eq, Hash)]
pub enum WarpAbility {
    CannotWarp,
    CanWarp(WarpState)
}

pub struct Character {
    pub x_pos: f32,
    pub y_pos: f32,
    pub x_vel: f32,
    pub y_vel: f32,
    pub x_vel_cap: f32,
    pub y_vel_cap: f32,
    pub count: i32,
    pub is_facing_right: bool,
    pub state: KittyStates,
    pub current_sprite_i: i32,
    pub sprite: &'static spritesheet::Sprite,
    pub following_i: Option<u8>,
    pub can_fly: bool,
    pub sprite_type: PresetSprites,
    pub warp_ability: WarpAbility,
}

#[derive(PartialEq, Eq, Hash)]


pub enum KittyStates {
    Sleeping,
    Walking(u8),
    JumpingUp(u8),
    HuggingWall(bool),
    OnCeiling(u8)
}



impl Character {
    pub fn new(sprite_type: PresetSprites) -> Character {
        Character {
            x_pos: 10 as f32,
            y_pos: 10.0,
            x_vel: 0.0,
            y_vel: 0.0,
            x_vel_cap: 2.0,
            y_vel_cap: 7.0,
            count: 0,
            is_facing_right: true,
            state: KittyStates::JumpingUp(200),
            current_sprite_i: 0,
            sprite: &spritesheet::Sprite::from_preset(&sprite_type),
            following_i: None,
            can_fly: false,
            sprite_type,
            warp_ability: WarpAbility::CannotWarp
        }
    }
}