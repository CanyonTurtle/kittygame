//! Kitty game!
//!
//! [`kittygame`]: https://canyonturtle.github.io/kittygame/


// ideas
//
// custom tilemap code? or hand write? custom tilemap code is preferrable.

#[cfg(feature = "buddy-alloc")]
mod alloc;
mod spritesheet;
mod kitty_ss;

mod wasm4;
use std::borrow::BorrowMut;

use game::{game_constants::{TILE_WIDTH_PX, TILE_HEIGHT_PX, X_LEFT_BOUND, X_RIGHT_BOUND, Y_LOWER_BOUND, Y_UPPER_BOUND, GameMode, N_NPCS}, game_state::GameState, entities::{MovingEntity, Character}, camera::Camera, collision::{update_pos, check_entity_collisions}};
use num;
mod game;
use wasm4::*;

use crate::{game::{entities::OptionallyEnabledPlayer, collision::{get_bound_of_character, AbsoluteBoundingBox}, game_constants::OptionsState}, spritesheet::KITTY_SPRITESHEET_PALLETES};



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
                                game_state.background_tiles[tile_i].frames[0].width as u32,
                                game_state.background_tiles[tile_i].frames[0].height as u32,
                                game_state.background_tiles[tile_i].frames[0].start_x as u32,
                                game_state.background_tiles[tile_i].frames[0].start_y as u32,
                                (game_state.spritesheet_stride) as u32,
                                spritesheet::KITTY_SPRITESHEET_FLAGS,
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
        the_char.sprite.frames[i].width as u32,
        the_char.sprite.frames[i].height as u32,
        the_char.sprite.frames[i].start_x as u32,
        the_char.sprite.frames[i].start_y as u32,
        *spritesheet_stride as u32,
        spritesheet::KITTY_SPRITESHEET_FLAGS | if the_char.is_facing_right { 0 } else { BLIT_FLIP_X },
    );
}

static mut NPC_INPUTS: [u8; N_NPCS] = [0; N_NPCS];



static mut PREVIOUS_GAMEPAD: [u8; 4] = [0, 0, 0, 0];

fn get_inputs_this_frame() -> [[u8; 4]; 2] {
    let gamepads: [u8; 4] = unsafe { [*GAMEPAD1, *GAMEPAD2, *GAMEPAD3, *GAMEPAD4] };
    let mut btns_pressed_this_frame: [u8; 4] = [0; 4];

    for i in 0..gamepads.len() {
        let gamepad = gamepads[i];
        let previous = unsafe {PREVIOUS_GAMEPAD[i]};
        let pressed_this_frame = gamepad & (gamepad ^ previous);
        btns_pressed_this_frame[i] = pressed_this_frame;
        unsafe {PREVIOUS_GAMEPAD.copy_from_slice(&gamepads[..])};
    }
    [btns_pressed_this_frame, gamepads]
}

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
    



    let [btns_pressed_this_frame, gamepads] = get_inputs_this_frame();

    

    
    
    match &mut game_state.game_mode {
        GameMode::NormalPlay => {
            
            
            unsafe {
                *PALETTE = spritesheet::KITTY_SPRITESHEET_PALLETES[game_state.pallette_idx];
            }
            unsafe { *DRAW_COLORS = spritesheet::KITTY_SPRITESHEET_DRAW_COLORS }
            
            

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
                    update_pos(&game_state.map, MovingEntity::OptionalPlayer(optional_player), gamepads[i], game_state.godmode);
                    drawcharacter(&game_state.spritesheet, &game_state.spritesheet_stride, &game_state.camera.borrow(), MovingEntity::OptionalPlayer(optional_player));
                } 
            }

            check_entity_collisions(&game_state);
        

            
            // unsafe { *DRAW_COLORS = 0x1112 }
            // text("WELCOME TO KITTY GAME.          :D       xD                           WHAT IS POPPIN ITS YOUR BOY, THE KITTY GAME", 200 - game_state.camera.current_viewing_x_offset as i32, 130);
            
            // unsafe { *DRAW_COLORS = spritesheet::KITTY_SPRITESHEET_DRAW_COLORS }
            let inputs: &mut [u8; N_NPCS] = unsafe {NPC_INPUTS.borrow_mut()};
    
            for i in 0..game_state.npcs.borrow().len() {
                let rngg = &mut game_state.rng.borrow_mut();
                let rand_val = (rngg.next() % 255) as u8;
                let current_npc = &game_state.npcs.borrow()[i];
                let mut use_rng_input = false;
                match current_npc.following_i {
                    None => {
                        use_rng_input = true;
                    },
                    Some(p_i) => {
                        let the_opt_player = &game_state.players.borrow()[p_i as usize];
                        if let OptionallyEnabledPlayer::Enabled(p) = the_opt_player {
                            let p_bound = get_bound_of_character(&p);
                            if rngg.next() % 10 > 1 {
                                inputs[i] = 0;
                                let npc_bound: AbsoluteBoundingBox = get_bound_of_character(&current_npc);
                                
                                // if current_npc.x_pos + (npc_bound.width as f32) < p.x_pos {
                                // else if current_npc.x_pos > p.x_pos + p_bound.width as f32 {
                                
                                // make NPCs tryhard when they're not in the same Y to get to exact x position to help with climbing
                                let mut tryhard_get_to_0: bool = true;
                                
                                // fall by doing nothing
                                if current_npc.y_pos + (npc_bound.height as f32) < p.y_pos {
    
                                }
                                else if current_npc.y_pos > p.y_pos + p_bound.height as f32 {
                                    inputs[i] |= BUTTON_1;
                                } else {
                                    tryhard_get_to_0 = false;
                                }

                                if tryhard_get_to_0 {
                                    if current_npc.x_pos < p.x_pos {
                                        inputs[i] |= BUTTON_RIGHT;
                                    }
                                    else if current_npc.x_pos > p.x_pos {
                                        inputs[i] |= BUTTON_LEFT;
                                    }
                                }
                                else {
                                    if current_npc.x_pos + (npc_bound.width as f32) < p.x_pos {
                                        inputs[i] |= BUTTON_RIGHT;
                                    }
                                    else if current_npc.x_pos > p.x_pos + p_bound.width as f32 {
                                        inputs[i] |= BUTTON_LEFT;
                                    }
                                }
                            }
                            else {
                                use_rng_input = true;
                            }
                        }
                        else {
                            use_rng_input = false;
                        }
                    }
                    
                }
                
                if use_rng_input {
                        
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
            }
    
            for (i, npc) in game_state.npcs.borrow_mut().iter_mut().enumerate() {
                update_pos(&game_state.map, MovingEntity::NPC(npc), inputs[i], game_state.godmode);
                drawcharacter(&game_state.spritesheet, &game_state.spritesheet_stride, &game_state.camera.borrow(), MovingEntity::NPC(npc));
            }

            
            drawmap(&game_state);
            
            if btns_pressed_this_frame[0] & BUTTON_2 != 0 {
                game_state.game_mode = GameMode::Options(OptionsState{ current_selection: 0 });
                return;
                // game_state.regenerate_map();
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
            //     spritesheet::KITTY_SPRITESHEET_FLAGS | if bob.facing_right { 0 } else { BLIT_FLIP_X },
            // );

            const TOP_UI_TEXT_Y: i32 = 2;

            const BOTTOM_UI_TEXT_Y: i32 = 12; // 160 - 8 - 2;
                
            unsafe { *DRAW_COLORS = 0x0001}

            for i in 0..160 {
                for j in 0..20 {
                    if (i + j) % 3 != 0 {
                        line(i, j, i, j)
                    }
                }
            }

            unsafe { *DRAW_COLORS = 0x0002 }

            fn layertext (t: &str, x: i32, y: i32) {

                unsafe {*DRAW_COLORS = 0x0001}
                text(t, x + 1, y);
                text(t, x, y + 1);
                text(t, x + 1, y + 1);
                unsafe {*DRAW_COLORS = 0x0002}

                text(t, x, y);
            
            }
            layertext("< >=move,x=jmp,z=new", 0, TOP_UI_TEXT_Y);
            //layertext("z=reset", 104, 8);
            
            let total_npcs_to_find: u8 = N_NPCS as u8;
            let mut current_found_npcs = 0;
            for npc in game_state.npcs.borrow().iter() {
                current_found_npcs += match npc.following_i {
                    None => 0,
                    Some(_) => 1
                }
            }
            // layertext(&format!["{}/{} found", current_found_npcs, total_npcs_to_find], 0, BOTTOM_UI_TEXT_Y);

            // keep going till the timer hits
            if total_npcs_to_find == current_found_npcs {
                layertext("You found them!! :D", 0, BOTTOM_UI_TEXT_Y)    
            } else {
                layertext("find the kitties...", 0, BOTTOM_UI_TEXT_Y);
                game_state.timer += 1;
            }
            
            
            // let time_sec = game_state.timer as f32 / 60.0;
            // layertext(&format!["{:.1} sec", time_sec], 90, BOTTOM_UI_TEXT_Y);

            // draw UI lines
            // line(0, TOP_UI_TEXT_Y + 8 + 1, 160, TOP_UI_TEXT_Y + 8 + 1);
            line(0, TOP_UI_TEXT_Y - 2, 160, TOP_UI_TEXT_Y - 2);

            line(0, BOTTOM_UI_TEXT_Y + 8 + 1, 160, BOTTOM_UI_TEXT_Y + 8 + 1);
            // line(0, BOTTOM_UI_TEXT_Y - 2, 160, BOTTOM_UI_TEXT_Y - 2);
        },
        GameMode::StartScreen => {
            unsafe { *DRAW_COLORS = 0x1112 }
            text("Find the kitties!", 20, 20);
            text("Any key: start", 20, 40);
            unsafe {
                *PALETTE = spritesheet::KITTY_SPRITESHEET_PALLETES[game_state.pallette_idx];
            }
            unsafe { *DRAW_COLORS = spritesheet::KITTY_SPRITESHEET_DRAW_COLORS }
            game_state.rng.borrow_mut().next();
            if gamepads[0] != 0 {
                game_state.game_mode = GameMode::NormalPlay;
                // drop(game_state.map.chunks);
                text("Spawning map...", 20, 50);
                game_state.regenerate_map();
            }
        },
        GameMode::Options(option_state) => {
            const MENU_X: i32 = 55;
            const MENU_TOP_Y: i32 = 50;
            const MENU_SPACING: i32 = 15;
            const N_OPTIONS: u8 = 4;
            unsafe { *DRAW_COLORS = 0x0002 }
            text("-- OPTIONS --", 25, 20);
            text("back", MENU_X, MENU_TOP_Y + MENU_SPACING * 0);
            text("pallette", MENU_X, MENU_TOP_Y + MENU_SPACING * 1);

            for (i, c) in (0x0002..=0x0004).enumerate() {
                unsafe { *DRAW_COLORS = c }
                text("x", MENU_X + 70 + 8 * i as i32, MENU_TOP_Y + MENU_SPACING * 1)
            }
            unsafe { *DRAW_COLORS = 0x0002 }

            text("fly", MENU_X, MENU_TOP_Y + MENU_SPACING * 2);
            text("reset", MENU_X, MENU_TOP_Y + MENU_SPACING * 3);

            let cursor_x = MENU_X -25;
            let cursor_y: i32 = MENU_TOP_Y + MENU_SPACING * option_state.current_selection as i32;

            text(">>", cursor_x, cursor_y);

            if btns_pressed_this_frame[0] & BUTTON_DOWN != 0{
                option_state.current_selection += 1;
                option_state.current_selection %= N_OPTIONS;
            }
            else if btns_pressed_this_frame[0] & BUTTON_UP != 0{
                option_state.current_selection -= 1;
                option_state.current_selection %= N_OPTIONS;
            } else if btns_pressed_this_frame[0] & (BUTTON_1 | BUTTON_2) != 0 {
                match option_state.current_selection {
                    0 => {
                        game_state.game_mode = GameMode::NormalPlay
                    },
                    1 => {
                        game_state.pallette_idx += 1;
                        game_state.pallette_idx %= KITTY_SPRITESHEET_PALLETES.len();
                        unsafe {
                            *PALETTE = spritesheet::KITTY_SPRITESHEET_PALLETES[game_state.pallette_idx];
                        }
                        // unsafe { *DRAW_COLORS = spritesheet::KITTY_SPRITESHEET_DRAW_COLORS }
                    },
                    2 => {
                        game_state.godmode = !game_state.godmode;
                        game_state.game_mode = GameMode::NormalPlay;
                    },
                    3 => {
                        game_state.regenerate_map();
                        game_state.game_mode = GameMode::NormalPlay;
                    }
                    _ => {
                        unreachable!()
                    }
                }
            }
        }
    }

}
