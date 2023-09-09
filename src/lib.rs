//! Kitty game!
//!
//! [`kittygame`]: https://canyonturtle.github.io/kittygame/


// ideas
//
// custom tilemap code? or hand write? custom tilemap code is preferrable.

#[cfg(feature = "buddy-alloc")]
mod alloc;
mod spritesheet;
mod wasm4;
use std::borrow::BorrowMut;

use game::{game_constants::{TILE_WIDTH_PX, TILE_HEIGHT_PX, X_LEFT_BOUND, X_RIGHT_BOUND, Y_LOWER_BOUND, Y_UPPER_BOUND, GameMode, N_NPCS}, game_state::GameState, entities::{MovingEntity, Character}, camera::Camera, collision::update_pos};
use num;
mod game;
use wasm4::*;

use crate::game::entities::OptionallyEnabledPlayer;



// const MIN_BUILDING_DIM: i32 = 4;






fn drawmap(game_state: &GameState) {
    let map = &game_state.map;
    let camera = &game_state.camera;

    for chunk in &map.chunks {
        for row in 0..chunk.bound.height {
            for col in 0..chunk.bound.width {
                let map_tile_i = chunk.get_tile(col as usize, row as usize);
                match map_tile_i {
                    0 => {},
                    tile_idx => {
                        let tile_i: usize = tile_idx as usize - 1; // *tile_idx as usize;
                        // trace(format!("Tile {tile_i}"));
                        let chunk_x_offset: i32 = (TILE_WIDTH_PX) as i32 * chunk.bound.x;
                        let chunk_y_offset: i32 = (TILE_HEIGHT_PX) as i32 * chunk.bound.y;
                        let x_loc = (chunk_x_offset + col as i32 * TILE_HEIGHT_PX as i32) - camera.borrow().current_viewing_x_offset as i32;
                        let y_loc = (chunk_y_offset + row as i32 * TILE_WIDTH_PX as i32) - camera.borrow().current_viewing_y_offset as i32;

                        if x_loc >= 0 && x_loc < 160 && y_loc > 0 && y_loc < 160 {
                            blit_sub(
                                &game_state.spritesheet,
                                x_loc,
                                y_loc,
                                game_state.background_tiles[tile_i].frames[0].positioning.width as u32,
                                game_state.background_tiles[tile_i].frames[0].positioning.height as u32,
                                game_state.background_tiles[tile_i].frames[0].positioning.start_x as u32,
                                game_state.background_tiles[tile_i].frames[0].positioning.start_y as u32,
                                game_state.spritesheet_stride as u32,
                                spritesheet::KITTY_SS_FLAGS,
                            );
                        }
                    },
                }
                
            }
        }
    }
}














static mut GAME_STATE_HOLDER: Option<GameState<'static>> = None;


fn drawcharacter(spritesheet: &[u8], spritesheet_stride: &usize, camera: &Camera, character: MovingEntity) {

    let the_char: &mut Character;

    match character {
        MovingEntity::OptionalPlayer(optionally_enabled_player) => {
            match optionally_enabled_player {
                OptionallyEnabledPlayer::Enabled(character) => {
                    the_char = character;
                }
                OptionallyEnabledPlayer::Disabled => {
                    return
                }
            }
        }
        MovingEntity::NPC(npc) => {
            the_char = npc;
        }
    }

    let i = the_char.current_sprite_i as usize;
    blit_sub(
        &spritesheet,
        (the_char.x_pos - camera.current_viewing_x_offset) as i32,
        (the_char.y_pos - camera.current_viewing_y_offset) as i32,
        the_char.sprite.frames[i].positioning.width as u32,
        the_char.sprite.frames[i].positioning.height as u32,
        the_char.sprite.frames[i].positioning.start_x as u32,
        the_char.sprite.frames[i].positioning.start_y as u32,
        *spritesheet_stride as u32,
        spritesheet::KITTY_SS_FLAGS | if the_char.is_facing_right { 0 } else { BLIT_FLIP_X },
    );
}

static mut NPC_INPUTS: [u8; N_NPCS] = [0; N_NPCS];

static mut PREVIOUS_GAMEPAD: [u8; 4] = [0, 0, 0, 0];
#[no_mangle]
fn update() {


    let game_state: &mut GameState;

    unsafe {
        match &mut GAME_STATE_HOLDER {
            None => {
                let new_game_state = GameState::new();
                GAME_STATE_HOLDER = Some(new_game_state);
                
            },
            Some(_) => {

            }
        }
        match &mut GAME_STATE_HOLDER {
            Some(game_state_holder) => {
                game_state = game_state_holder;
            },
            None => unreachable!()
        }
    }
    



    let gamepads: [u8; 4] = unsafe { [*GAMEPAD1, *GAMEPAD2, *GAMEPAD3, *GAMEPAD4] };
    let mut btns_pressed_this_frame: [u8; 4] = [0; 4];


    for i in 0..gamepads.len() {
        let gamepad = gamepads[i];
        let previous = unsafe {PREVIOUS_GAMEPAD[i]};
        let pressed_this_frame = gamepad & (gamepad ^ previous);
        btns_pressed_this_frame[i] = pressed_this_frame;
        unsafe {PREVIOUS_GAMEPAD.copy_from_slice(&gamepads[..])};
    }
    

    
    
    match game_state.game_mode {
        GameMode::NormalPlay => {
            
            
            unsafe {
                *PALETTE = spritesheet::KITTY_SS_PALLETE;
            }
            unsafe { *DRAW_COLORS = spritesheet::KITTY_SS_DRAW_COLORS }
            
            

            let mut player_idx: u8 = 0b0;

            unsafe {
                // If netplay is active
                if *NETPLAY & 0b100 != 0 {
                    player_idx = *NETPLAY & 0b011;
                    // Render the game from player_idx's perspective
                    
                    }
                else {
    
                }
            }
            match &mut game_state.players.borrow_mut()[player_idx as usize]{
                OptionallyEnabledPlayer::Disabled => {

                },
                OptionallyEnabledPlayer::Enabled(player) => {
                    game_state.camera.borrow_mut().current_viewing_x_target = num::clamp(player.x_pos - 80.0, X_LEFT_BOUND as f32, X_RIGHT_BOUND as f32);
                    game_state.camera.borrow_mut().current_viewing_y_target = num::clamp(player.y_pos - 80.0, Y_LOWER_BOUND as f32, Y_UPPER_BOUND as f32);
                    
                }
            }

            game_state.camera.borrow_mut().slew();

            {
                let mut optional_players = game_state.players.borrow_mut();

                for (i, optional_player) in &mut optional_players.iter_mut().enumerate() {
                    update_pos(&game_state.map, MovingEntity::OptionalPlayer(optional_player), gamepads[i]);
                    drawcharacter(&game_state.spritesheet, &game_state.spritesheet_stride, &game_state.camera.borrow(), MovingEntity::OptionalPlayer(optional_player));
                } 
            }
        

            
            // unsafe { *DRAW_COLORS = 0x1112 }
            // text("WELCOME TO KITTY GAME.          :D       xD                           WHAT IS POPPIN ITS YOUR BOY, THE KITTY GAME", 200 - game_state.camera.current_viewing_x_offset as i32, 130);
            
            // unsafe { *DRAW_COLORS = spritesheet::KITTY_SS_DRAW_COLORS }
            let inputs: &mut [u8; N_NPCS] = unsafe {NPC_INPUTS.borrow_mut()};
    
            for i in 0..game_state.npcs.borrow().len() {
                let rngg = &mut game_state.rng.borrow_mut();
                let rand_val = (rngg.next() % 255) as u8;
                if rand_val < 20 {
                    inputs[i] = 0x10;
                }
                else if rand_val < 40 {
                    inputs[i] = 0x20;
                }
                else if rand_val < 42 {
                    inputs[i] = BUTTON_1;
                }
                else {
                    inputs[i] = 0x0;
                }
                
            }
    
            for (i, npc) in game_state.npcs.borrow_mut().iter_mut().enumerate() {
                update_pos(&game_state.map, MovingEntity::NPC(npc), inputs[i]);
                drawcharacter(&game_state.spritesheet, &game_state.spritesheet_stride, &game_state.camera.borrow(), MovingEntity::NPC(npc));
            }

            
            drawmap(&game_state);
            
            if btns_pressed_this_frame[0] & BUTTON_2 != 0 {
                game_state.regenerate_map();
            }
            
            // blit_sub(
            //     &game_state.spritesheet,
            //     0 as i32,
            //     150 as i32,
            //     game_state.background_tiles[0].frames[0].positioning.width as u32,
            //     game_state.background_tiles[0].frames[0].positioning.height as u32,
            //     game_state.background_tiles[0].frames[0].positioning.start_x as u32,
            //     game_state.background_tiles[0].frames[0].positioning.start_y as u32,
            //     game_state.spritesheet_stride as u32,
            //     spritesheet::KITTY_SS_FLAGS | if bob.facing_right { 0 } else { BLIT_FLIP_X },
            // );

            unsafe { *DRAW_COLORS = 0x1112 }
            text("< > to move, x=jump", 0, 0);
            text("z=reset", 104, 8);

        },
        GameMode::StartScreen => {
            unsafe { *DRAW_COLORS = 0x1112 }
            text("Any key: start", 20, 20);
            unsafe {
                *PALETTE = spritesheet::KITTY_SS_PALLETE;
            }
            unsafe { *DRAW_COLORS = spritesheet::KITTY_SS_DRAW_COLORS }
            game_state.rng.borrow_mut().next();
            if gamepads[0] != 0 {
                game_state.game_mode = GameMode::NormalPlay;
                // drop(game_state.map.chunks);
                text("Spawning map...", 20, 50);
                game_state.regenerate_map();
            }
        }
    }

}
