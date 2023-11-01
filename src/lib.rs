//! Kitty game!
//!
//! [`kittygame`]: https://canyonturtle.github.io/kittygame/

// ideas
//
// custom tilemap code? or hand write? custom tilemap code is preferrable.

#[cfg(feature = "buddy-alloc")]
mod alloc;
mod kitty_ss;
mod spritesheet;

mod wasm4;
use std::borrow::BorrowMut;

use game::{
    camera::Camera,
    collision::{check_entity_collisions, update_pos},
    entities::{Character, MovingEntity, KittyStates},
    game_constants::{
        MAX_N_NPCS, TILE_HEIGHT_PX, TILE_WIDTH_PX, X_LEFT_BOUND, X_RIGHT_BOUND, Y_LOWER_BOUND,
        Y_UPPER_BOUND,
    },
    game_state::GameState,
    menus::GameMode,
    music::{play_bgm, SONGS}, game_map::MAP_TILESETS,
};
use title_ss::{OUTPUT_ONLINEPNGTOOLS_WIDTH, OUTPUT_ONLINEPNGTOOLS_HEIGHT, OUTPUT_ONLINEPNGTOOLS_FLAGS};
mod game;
use core::cell::RefCell;
use wasm4::*;
mod title_ss;

use crate::{
    game::{
        collision::{get_bound_of_character, AbsoluteBoundingBox},
        entities::OptionallyEnabledPlayer,
        menus::{Modal, NormalPlayModes, MenuTypes}, game_constants::{COUNTDOWN_TIMER_START, START_DIFFICULTY_LEVEL, MAJOR_VERSION, MINOR_VERSION, INCR_VERSION}, popup_text::PopTextRingbuffer,
    },
    title_ss::OUTPUT_ONLINEPNGTOOLS
};

// const MIN_BUILDING_DIM: i32 = 4;

fn drawmap(game_state: &GameState) {
    let map = &game_state.map;
    let camera = &game_state.camera;

    let tileset = &MAP_TILESETS[*game_state.tileset_idx.borrow()];

    for chunk in &map.chunks {
        for row in 0..chunk.bound.height {
            for col in 0..chunk.bound.width {
                let map_tile_i = chunk.get_tile(col as usize, row as usize);
                match map_tile_i {
                    0 => {}
                    tile_idx => {
                        let tile_i: usize = tileset[tile_idx as usize] as usize; // *tile_idx as usize;
                        if tile_i == 0 {
                            continue
                        }                                   // trace(format!("Tile {tile_i}"));
                        let chunk_x_offset: i32 = (TILE_WIDTH_PX) as i32 * chunk.bound.x;
                        let chunk_y_offset: i32 = (TILE_HEIGHT_PX) as i32 * chunk.bound.y;
                        let x_loc = (chunk_x_offset + col as i32 * TILE_HEIGHT_PX as i32)
                            - camera.borrow().current_viewing_x_offset as i32;
                        let y_loc = (chunk_y_offset + row as i32 * TILE_WIDTH_PX as i32)
                            - camera.borrow().current_viewing_y_offset as i32;

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
                    }
                }
            }
        }
    }
}

static mut GAME_STATE_HOLDER: Option<GameState<'static>> = None;

fn drawcharacter(
    spritesheet: &[u8],
    spritesheet_stride: &usize,
    camera: &Camera,
    character: MovingEntity,
) {
    let the_char: &mut Character;

    match character {
        MovingEntity::OptionalPlayer(optionally_enabled_player) => {
            match optionally_enabled_player {
                OptionallyEnabledPlayer::Enabled(p) => {
                    the_char = &mut p.character;
                }
                OptionallyEnabledPlayer::Disabled => return,
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
        spritesheet::KITTY_SPRITESHEET_FLAGS
            | if the_char.is_facing_right {
                0
            } else {
                BLIT_FLIP_X
            }
            | match the_char.state {
                KittyStates::OnCeiling(_) => {BLIT_FLIP_Y},
                _ => 0
            }
    );
}

static mut NPC_INPUTS: [u8; MAX_N_NPCS] = [0; MAX_N_NPCS];

static mut PREVIOUS_GAMEPAD: [u8; 4] = [0, 0, 0, 0];

fn get_inputs_this_frame() -> [[u8; 4]; 2] {
    let gamepads: [u8; 4] = unsafe { [*GAMEPAD1, *GAMEPAD2, *GAMEPAD3, *GAMEPAD4] };
    let mut btns_pressed_this_frame: [u8; 4] = [0; 4];

    for i in 0..gamepads.len() {
        let gamepad = gamepads[i];
        let previous = unsafe { PREVIOUS_GAMEPAD[i] };
        let pressed_this_frame = gamepad & (gamepad ^ previous);
        btns_pressed_this_frame[i] = pressed_this_frame;
    }
    unsafe { PREVIOUS_GAMEPAD.copy_from_slice(&gamepads[0..4]) };
    [btns_pressed_this_frame, gamepads]
}

// DRAW BLURRED BACKGROUND BEHIND SCORE AND TIME TEXTS IN-GAME
const TOP_UI_TEXT_Y: i32 = 2;
const BOTTOM_UI_TEXT_Y: i32 = 160 - 8; // 160 - 8 - 2;

fn draw_modal_bg(pf: &AbsoluteBoundingBox<f32, f32>, style: u8) {
    unsafe { *DRAW_COLORS = 0x0001 }
    let p: AbsoluteBoundingBox<i32, u32> = AbsoluteBoundingBox {
        x: pf.x as i32,
        y: pf.y as i32,
        width: pf.width as u32,
        height: pf.height as u32,
    };

    unsafe {
        *DRAW_COLORS = 0x0001;
    }

    match style {
        1 => {
            rect(p.x, p.y, p.width as u32, p.height as u32);
        }
        _ => {}
    }

    unsafe {
        *DRAW_COLORS = match style {
            0 => 0x0001,
            _ => 0x0002,
        }
    };

    // fill
    for i in p.x..=p.x + p.width as i32 {
        for j in p.y..=p.y + p.height as i32 {
            let cond = match style {
                1 => false,
                _ => (i + j) % 3 != 0,
            };
            if cond {
                line(i, j, i, j)
            }
        }
    }

    unsafe { *DRAW_COLORS = 0x0002 }
    // borders

    match style {
        1 => {
            line(p.x, p.y, p.x + p.width as i32, p.y);
            line(p.x, p.y, p.x, p.y + p.height as i32);
            line(
                p.x,
                p.y + p.height as i32,
                p.x + p.width as i32,
                p.y + p.height as i32,
            );
            line(
                p.x + p.width as i32,
                p.y,
                p.x + p.width as i32,
                p.y + p.height as i32,
            );
        }
        _ => {}
    }
    
}

#[no_mangle]
fn update() {
    let game_state: &mut GameState;

    // -------- INITIALIZE GAME STATE IF NEEDED ----------
    unsafe {
        match &mut GAME_STATE_HOLDER {
            None => {
                spritesheet::Sprite::init_all_sprites();
                let mut new_game_state = GameState::new();
                for _ in 0..20 {
                    new_game_state.rng.borrow_mut().next();
                }
                
                new_game_state.regenerate_map();
                GAME_STATE_HOLDER = Some(new_game_state);
                // trace("finished reset");
            }
            Some(_) => {}
        }
        match &mut GAME_STATE_HOLDER {
            Some(game_state_holder) => {
                game_state = game_state_holder;
            }
            None => unreachable!(),
        }
    }

    // ----------- UPDATE TIMER AND PLAY BGM -----------


    
    

    game_state.song_timer += 1;
    play_bgm(game_state.song_timer, &SONGS[game_state.song_idx]);


    let mut player_idx: u8 = 0b0;

    // SET CAMERA POSITION
    match &mut game_state.players.borrow_mut()[player_idx as usize] {
        OptionallyEnabledPlayer::Disabled => {}
        OptionallyEnabledPlayer::Enabled(player) => {
            game_state.camera.borrow_mut().current_viewing_x_target = num::clamp(
                player.character.x_pos - 80.0,
                X_LEFT_BOUND as f32,
                X_RIGHT_BOUND as f32,
            );
            game_state.camera.borrow_mut().current_viewing_y_target = num::clamp(
                player.character.y_pos - 80.0,
                Y_LOWER_BOUND as f32,
                Y_UPPER_BOUND as f32,
            );
        }
    }

    game_state.camera.borrow_mut().slew();

    // ------------- POLL INPUT ---------------

    let [btns_pressed_this_frame, gamepads] = get_inputs_this_frame();

    // ON TITLE SCREEN, MOVE PLAYER 1 BASED ON TIME
    
    // CHECK IF CHARACTERS / CATS ARE COLLIDING
    check_entity_collisions(&game_state);



    // PREPARE TO RENDER THE MAP & ENTITIES
    unsafe {
        *PALETTE = spritesheet::KITTY_SPRITESHEET_PALETTES[game_state.pallette_idx];
    }
    unsafe { *DRAW_COLORS = spritesheet::KITTY_SPRITESHEET_DRAW_COLORS }

    // MOVE AND RENDER THE PLAYERS 
    {
        let mut optional_players = game_state.players.borrow_mut();

        for (i, optional_player) in &mut optional_players.iter_mut().enumerate() {


            let mut input = match false { // showing_modal {
                false => gamepads[i],
                true => 0,
            };
            if i == 0 {
                match game_state.game_mode {
                    GameMode::StartScreen => {
                        let mut move_n = (((game_state.song_timer / 10) * 31) % 29) as u8;
                        move_n &= !(BUTTON_LEFT | BUTTON_RIGHT);
                        input = move_n;
                        match move_n {
                            0..=2 => {
                                input |= BUTTON_LEFT;
                            },
                            3..=6=> {
                                input |= BUTTON_RIGHT;
                            }
                            _ => {}
                        }
                    },
                    GameMode::NormalPlay(_) => {},
                }
            }
            update_pos(
                &game_state.map,
                MovingEntity::OptionalPlayer(optional_player),
                input,
                game_state.godmode,
            );
            drawcharacter(
                &game_state.spritesheet,
                &game_state.spritesheet_stride,
                &game_state.camera.borrow(),
                MovingEntity::OptionalPlayer(optional_player),
            );
        }
    }

    // CREATE INPUTS FOR NPCS
    let inputs: &mut [u8; MAX_N_NPCS] = unsafe { NPC_INPUTS.borrow_mut() };
    let l;
    {
        l = game_state.npcs.borrow().len();
    }
    for i in 0..l {
        let rngg = &mut game_state.rng.borrow_mut();
        let rand_val = (rngg.next() % 255) as u8;
        let current_npc = &mut game_state.npcs.borrow_mut()[i];
        let mut use_rng_input = false;
        match current_npc.following_i {
            None => {
                use_rng_input = true;
            }
            Some(p_i) => {
                let the_opt_player = &game_state.players.borrow()[p_i as usize];
                if let OptionallyEnabledPlayer::Enabled(p) = the_opt_player {
                    let p_bound = get_bound_of_character(&p.character);
                    let npc_bound: AbsoluteBoundingBox<i32, u32> =
                        get_bound_of_character(&current_npc);
                    let needs_teleport;
                    {
                        // teleportAyh-shon if needed
                        const TELEPORT_AXIS_MIN_DIST: u32 = 160;
                        if p_bound.x.abs_diff(npc_bound.x) > TELEPORT_AXIS_MIN_DIST
                            || p_bound.y.abs_diff(npc_bound.y) > TELEPORT_AXIS_MIN_DIST
                        {
                            needs_teleport = true
                        } else {
                            needs_teleport = false
                        }
                    }

                    if needs_teleport {
                        current_npc.x_pos = p_bound.x as f32;
                        current_npc.y_pos = p_bound.y as f32;
                        current_npc.x_vel = 0.0;
                        current_npc.y_vel = 0.0;
                    } else {
                        if rngg.next() % 10 > 1 {
                            inputs[i] = 0;

                            // if current_npc.x_pos + (npc_bound.width as f32) < p.x_pos {
                            // else if current_npc.x_pos > p.x_pos + p_bound.width as f32 {

                            // make NPCs tryhard when they're not in the same Y to get to exact x position to help with climbing
                            let mut tryhard_get_to_0: bool = true;
                            let ch = &p.character;
                            // fall by doing nothing
                            if current_npc.y_pos + (npc_bound.height as f32) < ch.y_pos {
                            } else if current_npc.y_pos > ch.y_pos + p_bound.height as f32 {
                                inputs[i] |= BUTTON_1;
                            } else {
                                tryhard_get_to_0 = false;
                            }

                            if tryhard_get_to_0 {
                                if current_npc.x_pos < ch.x_pos {
                                    inputs[i] |= BUTTON_RIGHT;
                                } else if current_npc.x_pos > ch.x_pos {
                                    inputs[i] |= BUTTON_LEFT;
                                }
                            } else {
                                if current_npc.x_pos + (npc_bound.width as f32) < ch.x_pos {
                                    inputs[i] |= BUTTON_RIGHT;
                                } else if current_npc.x_pos > ch.x_pos + p_bound.width as f32
                                {
                                    inputs[i] |= BUTTON_LEFT;
                                }
                            }
                        } else {
                            use_rng_input = true;
                        }
                    }
                } else {
                    use_rng_input = false;
                }
            }
        }

        if use_rng_input {
            if rand_val < 20 {
                inputs[i] = 0x10;
            } else if rand_val < 40 {
                inputs[i] = 0x20;
            } else if rand_val < 42 {
                inputs[i] = BUTTON_1;
            } else {
                inputs[i] = 0x0;
            }
        }
    }

    // MOVE AND DRAW NPCS
    for (i, npc) in game_state.npcs.borrow_mut().iter_mut().enumerate() {
        update_pos(
            &game_state.map,
            MovingEntity::NPC(npc),
            inputs[i],
            game_state.godmode,
        );
        drawcharacter(
            &game_state.spritesheet,
            &game_state.spritesheet_stride,
            &game_state.camera.borrow(),
            MovingEntity::NPC(npc),
        );
    }

    // RENDER THE MAP
    drawmap(&game_state);

    match &game_state.game_mode {
        GameMode::NormalPlay(play_mode) => {
            let mut showing_modal = false;

            match play_mode {
                NormalPlayModes::MainGameplay => {
                    // handle player inputs here
                    game_state.countdown_paused = false;
                }
                NormalPlayModes::HoverModal(_) => {
                    showing_modal = true;
                    game_state.countdown_paused = true;
                }
            }
            
            // UPDATE WHICH PLAYER WE'RE PLAYING IN NETPLAY
            unsafe {
                // If netplay is active
                if *NETPLAY & 0b100 != 0 {
                    player_idx = *NETPLAY & 0b011;
                // Render the game from player_idx's perspective
                } else {
                }
            }

            


            
            

            

            draw_modal_bg(
                &AbsoluteBoundingBox {
                    x: -1.0,
                    y: 0.0,
                    width: 162.0,
                    height: 10.0,
                },
                0,
            );

            draw_modal_bg(
                &AbsoluteBoundingBox {
                    x: -1.0,
                    y: 150.0,
                    width: 162.0,
                    height: 10.0,
                },
                0,
            );

            // WRITE SCORE / TIME OF GAME
            fn layertext(t: &str, x: i32, y: i32) {
                unsafe { *DRAW_COLORS = 0x0001 }
                text(t, x + 1, y);
                text(t, x, y + 1);
                text(t, x + 1, y + 1);
                unsafe { *DRAW_COLORS = 0x0002 }

                text(t, x, y);
            }

            layertext(&format!["Lv. {}", game_state.difficulty_level], 0, BOTTOM_UI_TEXT_Y);
            layertext(&format!["Sc. {}", *game_state.score.borrow()], 80, BOTTOM_UI_TEXT_Y);

            // COUNT THE NUMBER OF NPCS THAT ARE FOLLOWING PLAYERS
            let mut current_found_npcs = 0;
            for npc in game_state.npcs.borrow().iter() {
                current_found_npcs += match npc.following_i {
                    None => 0,
                    Some(_) => 1,
                }
            }

            // RENDER NUMBER OF KITTIES TO FIND
            if game_state.total_npcs_to_find == current_found_npcs {
                // layertext("You found them!! :D", 0, TOP_UI_TEXT_Y);
                // game_state.difficulty_level += 1;
            } else {
                // layertext("find the kitties...", 0, BOTTOM_UI_TEXT_Y);
                layertext(
                    &format![
                        "{}/{} {:.2}",
                        current_found_npcs, game_state.total_npcs_to_find, *game_state.countdown_timer_msec.borrow() as u32 / 60
                    ],
                    0,
                    TOP_UI_TEXT_Y,
                );
            }

            // UPDATE POPUPS
            {
                let popup_texts_rb: &mut PopTextRingbuffer = &mut game_state.popup_text_ringbuffer.borrow_mut();
                popup_texts_rb.update_popup_positions();
                let camera = game_state.camera.borrow();
                for popup in popup_texts_rb.texts.iter() {
                    match popup {
                        Some(p) => {
                            const T_BEFORE_BLINK: u32 = 60;
                            if p.duration_timer < T_BEFORE_BLINK || p.duration_timer % 6 < 3 {
                                layertext(&p.text, (p.x_pos - camera.current_viewing_x_offset) as i32, (p.y_pos - camera.current_viewing_y_offset) as i32);
                            }
                        },
                        None => {

                        }
                    }
                }
            }

            // USE ABILITY CARDS
            for (p_i, pr) in game_state.players.borrow_mut().iter_mut().enumerate() {
                // if btns_pressed_this_frame[p_i] != 0 {
                //     trace(format!["check p{}: {}", p_i, btns_pressed_this_frame[p_i]]);
                // }
                
                
                match pr {
                    OptionallyEnabledPlayer::Enabled(p) => {
                        if !showing_modal && btns_pressed_this_frame[p_i] & BUTTON_2 != 0 {
                            let res = p.card_stack.try_use_cards();
                            // trace("tried");
                            let added_t;
                            let popup_t: Option<String>;
                            match res {
                                game::ability_cards::AbilityCardUsageResult::NothingHappened => {
                                    added_t = 0;
                                    popup_t = None;
                                },
                                game::ability_cards::AbilityCardUsageResult::GainedTime(t) => {
                                    added_t = t;
                                    popup_t = Some(format!["+{}", t]);
                                },
                                game::ability_cards::AbilityCardUsageResult::EnabledFlyAndTime(t) => {
                                    if p.character.can_fly {
                                        added_t = t;
                                        popup_t = Some(format!["+{}", t]);
                                    } else {
                                        p.character.can_fly = true;
                                        added_t = t - 10;
                                        popup_t = Some("+Fly!".to_string());
                                    }
                                    
                                }
                            }
                            match popup_t {
                                Some(pt) => {
                                    game_state.popup_text_ringbuffer.borrow_mut().add_new_popup(p.character.x_pos, p.character.y_pos, pt);
                                }
                                _ => {}
                            }
                            *game_state.countdown_timer_msec.borrow_mut() += added_t * 60;
                            *game_state.score.borrow_mut() += added_t * 60;
                        }
                    },
                    OptionallyEnabledPlayer::Disabled => {},
                }
            }
            

            // MOVE ABILITY CARD POSITIONS
            match &mut (*game_state.players.borrow_mut())[player_idx as usize] {
                OptionallyEnabledPlayer::Enabled(p) => {
                    for (i, card) in p.card_stack.cards.iter_mut().enumerate() {
                        match card {
                            Some(c) => {
                                c.target_x = (80 + 15 * i) as f32;
                                c.target_y = 1.0;
                            },
                            None => {}
                        }
                    }
                    p.card_stack.move_cards();
                },
                OptionallyEnabledPlayer::Disabled => {}
            }
            
            // DRAW ABILITY CARDS
            unsafe { *DRAW_COLORS = spritesheet::KITTY_SPRITESHEET_DRAW_COLORS }
            match &(*game_state.players.borrow())[player_idx as usize] {
                OptionallyEnabledPlayer::Enabled(p) => {
                    for card in p.card_stack.cards.iter() {
                        match &card {
                            
                            Some(c) => {
                                // trace(&format!["{}", i]);
                                blit_sub(
                                    &game_state.spritesheet,
                                    c.x_pos as i32,
                                    c.y_pos as i32,
                                    c.sprite.frames[0].width as u32,
                                    c.sprite.frames[0].height as u32,
                                    c.sprite.frames[0].start_x as u32,
                                    c.sprite.frames[0].start_y as u32,
                                    (game_state.spritesheet_stride) as u32,
                                    spritesheet::KITTY_SPRITESHEET_FLAGS,
                                );
                            },
                            None => {},
                        }
                        
                    }
                },
                OptionallyEnabledPlayer::Disabled => {},
            }

            // SHOW MODAL DIALOGS
            if showing_modal {
                match play_mode {
                    NormalPlayModes::MainGameplay => {
                        unreachable!()
                    }
                    NormalPlayModes::HoverModal(m) => {
                        let mut options_ready_to_select: bool = false;
                        
                        let ready_to_show_text;
                        {
                            let actual_position: &mut AbsoluteBoundingBox<f32, f32> = &mut m.actual_position.borrow_mut();
                            let target_position: &mut AbsoluteBoundingBox<i32, u32> = &mut m.target_position.borrow_mut();

                            const SPEED: f32 = 0.15;
                            const TOL: f32 = 10.0;

                            let real_tpy = target_position.y + (4f32 * num::Float::sin(game_state.song_timer as f32 * 0.05f32)) as i32;

                            actual_position.x += (target_position.x as f32 - actual_position.x) * SPEED;
                            actual_position.y += (real_tpy as f32 - actual_position.y) * SPEED;
                            actual_position.width += (target_position.width as f32 - actual_position.width) * SPEED;
                            actual_position.height += (target_position.height as f32 - actual_position.height) * SPEED;

                            

                            ready_to_show_text = (actual_position.width - target_position.width as f32).abs() < TOL;

                            draw_modal_bg(&actual_position, 1);
                        
                        }

                        let mut text_timer = 0;
                        {
                            *m.timer.borrow_mut() += 1;
                        }
                        const INTERACTIVE_DELAY: u32 = 60;
                        if ready_to_show_text && *m.timer.borrow() >= INTERACTIVE_DELAY {
                            {
                                // let timer: &mut u32 = &mut m.timer.borrow_mut();
                                // *timer += 1;
                                options_ready_to_select = true;
                                text_timer = *m.timer.borrow();

                            }
                        }
                        
                        let modal_text = |st: &str, x, y| {
                            text(st, m.actual_position.borrow().x as i32 + x, m.actual_position.borrow().y as i32 + y);
                        };
                        
                        

                        if ready_to_show_text {
                            // let cursor_opt: u8;
                            let mut option_selected: u8 = 10;
                            if options_ready_to_select {
                                let option: &mut u8 = &mut m.current_selection.borrow_mut();
                                // cursor_opt = *option;
                                if btns_pressed_this_frame[0] & BUTTON_DOWN != 0 {
                                    *option += 1;
                                    *option %= m.n_options;
                                } else if btns_pressed_this_frame[0] & BUTTON_UP != 0 {
                                    *option -= 1;
                                    *option %= m.n_options;
                                } else if btns_pressed_this_frame[0] & (BUTTON_1 | BUTTON_2) != 0 {
                                    option_selected = *option;
                                }
                            }
                            match m.menu_type {
                                MenuTypes::WonLevel => {
                                    const BLINK_START: u32 = 50;
                                    const BLINK_TITLE_PERIOD: u32 = 17;
                                    if text_timer < BLINK_START || (text_timer / BLINK_TITLE_PERIOD) % 2 == 0 {
                                        modal_text("Found!!", 12, 15);
                                    }

                                    match option_selected {
                                        0 => {
                                            game_state.difficulty_level += 1;
                                            game_state.game_mode =
                                                GameMode::NormalPlay(NormalPlayModes::MainGameplay);
                                            game_state.regenerate_map();
                                        }
                                        10 => {}
                                        _ => {
                                            unreachable!()
                                        }
                                    }
                                },
                                MenuTypes::Done => {
                                    const BLINK_START: u32 = 50;
                                    const BLINK_TITLE_PERIOD: u32 = 17;
                                    if text_timer < BLINK_START || (text_timer / BLINK_TITLE_PERIOD) % 2 == 0 {
                                        modal_text("Time's Up!", 34, 40);
                                    }

                                    modal_text(&format!["Score: {} pts", *game_state.score.borrow()], 10, 70);

                                    match option_selected {
                                        0 => {
                                            game_state.difficulty_level = START_DIFFICULTY_LEVEL;
                                            game_state.game_mode = GameMode::StartScreen;
                                        }
                                        10 => {}
                                        _ => {
                                            unreachable!()
                                        }
                                    }
                                },
                                MenuTypes::StartGameMessage => {
                                    modal_text("-- GOAL --", 30, 20);
                                    modal_text("Find all the", 20, 40);
                                    modal_text("kitties in time!", 10, 55);
                                    modal_text("-- CONTROLS --", 14, 100);
                                    modal_text("< > to move,", 24, 114); 
                                    modal_text("x=jump, z=card", 16, 126);
                                    match option_selected {
                                        0 => {
                                            game_state.game_mode = GameMode::NormalPlay(NormalPlayModes::MainGameplay);
                                        }
                                        10 => {}
                                        _ => {
                                            unreachable!()
                                        }
                                    }
                                }
                            }     
                        }
                    }
                }
            } else {

                // HELP TEXT AT START OF GAME
                if game_state.difficulty_level == 1 && *game_state.countdown_timer_msec.borrow() == COUNTDOWN_TIMER_START - 2 * 60 && *game_state.tutorial_text_counter.borrow() == 0 {
                    *game_state.tutorial_text_counter.borrow_mut() += 1;
                    game_state.game_mode = GameMode::NormalPlay(NormalPlayModes::HoverModal(Modal {
                        n_options: 1,
                        timer: RefCell::new(0),
                        current_selection: RefCell::new(0),
                        target_position: RefCell::new(AbsoluteBoundingBox {
                            x: 10,
                            y: 10,
                            width: 140,
                            height: 140,
                        }),
                        actual_position: RefCell::new(AbsoluteBoundingBox {
                            x: 80.0,
                            y: 80.0,
                            width: 1.0,
                            height: 1.0,
                        }),
                        menu_type: MenuTypes::StartGameMessage
                    }));
                }
                

                // ------- LEVEL WIN CONDITION -----------
                if game_state.total_npcs_to_find == current_found_npcs {
                    game_state.game_mode =
                        GameMode::NormalPlay(NormalPlayModes::HoverModal(Modal {
                            n_options: 1,
                            timer: RefCell::new(0),
                            current_selection: RefCell::new(0),
                            target_position: RefCell::new(AbsoluteBoundingBox {
                                x: 40,
                                y: 40,
                                width: 80,
                                height: 40,
                            }),
                            actual_position: RefCell::new(AbsoluteBoundingBox {
                                x: 0.0,
                                y: 0.0,
                                width: 1.0,
                                height: 1.0,
                            }),
                            menu_type: MenuTypes::WonLevel
                        }));
                    game_state.song_idx = 0;
                    game_state.song_timer = 0;
                }

                // PROGRESS TIME, CHECK FOR GAME END
                if !game_state.countdown_paused {
                    *game_state.countdown_timer_msec.borrow_mut() -= 1;
            
                    // ---- LOSE CONDITION ----
                    if *game_state.countdown_timer_msec.borrow() <= 0 {
            
                        game_state.song_idx = 0;
            
                        game_state.game_mode = GameMode::NormalPlay(NormalPlayModes::HoverModal(Modal {
                            n_options: 1,
                            timer: RefCell::new(0),
                            current_selection: RefCell::new(0),
                            target_position: RefCell::new(AbsoluteBoundingBox {
                                x: 10,
                                y: 10,
                                width: 140,
                                height: 140,
                            }),
                            actual_position: RefCell::new(AbsoluteBoundingBox {
                                x: 80.0,
                                y: 80.0,
                                width: 1.0,
                                height: 1.0,
                            }),
                            menu_type: MenuTypes::Done
                        }));
                    }
                }
            }
        }
        GameMode::StartScreen => {
            
            
            // SETUP TITLE MUSIC AND COLORS
            game_state.song_idx = 1;
            unsafe { *DRAW_COLORS = 0x0002 }

            // SHOW TITLE-SCREEN SUBTEXT
            const TIMER_INTERACTIVE_START: u32 = 100;
            if game_state.song_timer >= TIMER_INTERACTIVE_START {
                draw_modal_bg(
                    &AbsoluteBoundingBox {
                        x: 15.0,
                        y: 105.0,
                        width: 130.0,
                        height: 40.0,
                    },
                    0,
                );
                if game_state.song_timer % 30 >= 15 {
                    text("Any key: start", 20, 110);
                }
                
                text("by CanyonTurtle", 20, 125);
                text(" & BurntSugar  ", 20, 135);
                text(format!["ver. {}.{}.{}", MAJOR_VERSION, MINOR_VERSION, INCR_VERSION], 40, 150);
                if btns_pressed_this_frame[0] != 0 {
                    game_state.game_mode = GameMode::NormalPlay(NormalPlayModes::MainGameplay);
                    
                    // drop(game_state.map.chunks);
                    game_state.new_game();
                    game_state.regenerate_map();
                }
            }
            
            // RENDER THE TITLE
            unsafe { *DRAW_COLORS = 0x0034 }
            const TITLE_Y: i32 = 15;
            const TITLE_X: i32 = 5;
            let title_y_osc = match game_state.song_timer {
                0..=TIMER_INTERACTIVE_START => {
                    0
                }
                _ => {
                    (5f32 * num::Float::sin((game_state.song_timer - TIMER_INTERACTIVE_START) as f32 * 0.05f32)) as i32
                }
            };
            for row in 0..OUTPUT_ONLINEPNGTOOLS_HEIGHT as i32 {
                blit_sub(&OUTPUT_ONLINEPNGTOOLS, TITLE_X + (3000000f32 * (1f32 / (1f32 + num::Float::powf(game_state.song_timer as f32, 3f32))) * num::Float::sin((game_state.song_timer as f32 + row as f32 * 4f32) * 0.1f32)) as i32, TITLE_Y + title_y_osc + row, OUTPUT_ONLINEPNGTOOLS_WIDTH, 1, 0, row as u32, OUTPUT_ONLINEPNGTOOLS_WIDTH, OUTPUT_ONLINEPNGTOOLS_FLAGS)
            }
            unsafe {
                *PALETTE = spritesheet::KITTY_SPRITESHEET_PALETTES[game_state.pallette_idx];
            }
            unsafe { *DRAW_COLORS = spritesheet::KITTY_SPRITESHEET_DRAW_COLORS }
            game_state.rng.borrow_mut().next();
            
            // trace("updated positions");
            unsafe { *DRAW_COLORS = 0x1112 }
            
        }
    }
}
