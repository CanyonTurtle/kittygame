//! Kitty game!
//!
//! [`kittygame`]: https://canyonturtle.github.io/kittygame/

/// This is essentially the entrypoint of the game, providing the update() loop.
/// This has all the drawing code and lots of update logic.
mod alloc;
mod draw;
mod kitty_ss;
mod spritesheet;

mod wasm4;

use game::{
    cloud::Cloud,
    collision::{check_entity_collisions, update_pos},
    entities::{WarpAbility, WarpState},
    game_constants::{MAX_N_NPCS, SCREEN_HEIGHT_PX},
    menus::GameMode,
    music::{play_bgm, SONGS},
};

mod game;
mod title_ss;

use wasm4::*;

use crate::{
    alloc::init_heap,
    draw::draw_game,
    game::{
        collision::{get_bound_of_character, AbsoluteBoundingBox},
        entities::{MovingEntityMut, OptionallyEnabledPlayer},
        game_constants::{COUNTDOWN_TIMER_START, FINAL_LEVEL, START_DIFFICULTY_LEVEL},
        game_state::{GameState, RunType},
        menus::{MenuTypes, Modal, NormalPlayModes, SelectMenuFocuses, SelectSetup},
        popup_text::PopupIcon,
        rng::{GameRng, Rng},
    },
};

static mut GAME_STATE_HOLDER: Option<GameState<'static>> = None;

static mut NPC_INPUTS: [u8; MAX_N_NPCS] = [0; MAX_N_NPCS];

static mut PREVIOUS_GAMEPAD: [u8; 4] = [0, 0, 0, 0];

/// get joystick inputs from this and last frame.
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

/// Main loop that runs every frame. Progress the game state and render.
#[no_mangle]
fn update() {
    let game_state: &mut GameState;

    // -------- INITIALIZE GAME STATE IF NEEDED ----------
    unsafe {
        if !GAME_STATE_HOLDER.is_some() {
            init_heap();
            spritesheet::Sprite::init_all_sprites();
            let mut new_game_state = GameState::new();
            for _ in 0..20 {
                new_game_state.rng.next_for_worldgen();
            }

            new_game_state.regenerate_map();
            GAME_STATE_HOLDER = Some(new_game_state);
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

    // UPDATE WHICH PLAYER WE'RE PLAYING IN NETPLAY
    unsafe {
        // If netplay is active
        if *NETPLAY & 0b100 != 0 {
            player_idx = *NETPLAY & 0b011;
            // Render the game from player_idx's perspective
        }
    }

    // SET CAMERA POSITION
    match &mut game_state.players[player_idx as usize] {
        OptionallyEnabledPlayer::Disabled => {}
        OptionallyEnabledPlayer::Enabled(player) => {
            game_state.camera.current_viewing_x_target = player.character.x_pos - 80.0;
            game_state.camera.current_viewing_y_target = player.character.y_pos - 80.0;
        }
    }

    game_state.camera.slew();

    // ------------- POLL INPUT ---------------

    let [btns_pressed_this_frame, gamepads] = get_inputs_this_frame();

    // CHECK IF WE NEED TO FREEZE CHARACTERS / GAMEPLAY ON SCREEN
    let mut showing_modal = false;
    if let GameMode::NormalPlay(play_mode) = &game_state.game_mode {
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
    }
    // ON TITLE SCREEN, MOVE PLAYER 1 BASED ON TIME

    // CHECK IF CHARACTERS / CATS ARE COLLIDING
    if !showing_modal {
        check_entity_collisions(game_state);
    }

    // MOVE AND RENDER THE PLAYERS
    {
        let optional_players: &mut [OptionallyEnabledPlayer; 4] = &mut game_state.players;

        for (i, optional_player) in &mut optional_players.iter_mut().enumerate() {
            let mut input = match false {
                // showing_modal {
                false => gamepads[i],
                true => 0,
            };
            if i == 0 {
                if let GameMode::StartScreen = game_state.game_mode {
                    let mut move_n = (((game_state.song_timer / 10) * 31) % 29) as u8;
                    move_n &= !(BUTTON_LEFT | BUTTON_RIGHT);
                    input = move_n;
                    match move_n {
                        0..=2 => {
                            input |= BUTTON_LEFT;
                        }
                        3..=6 => {
                            input |= BUTTON_RIGHT;
                        }
                        _ => {}
                    }
                }
            }

            update_pos(
                &game_state.map,
                MovingEntityMut::OptionalPlayer(optional_player),
                input,
                game_state.godmode,
                &mut game_state.clouds,
            );
        }
    }

    // CREATE INPUTS FOR NPCS
    let inputs: &mut [u8; MAX_N_NPCS] = unsafe { &mut NPC_INPUTS };
    let l;
    {
        l = game_state.npcs.len();
    }
    (0..l).for_each(|i| {
        let rng = &mut game_state.rng;
        let rand_val = (rng.next_for_input() % 255) as u8;
        let current_npc = &mut game_state.npcs[i];
        let mut use_rng_input = false;
        match current_npc.following_i {
            None => {
                use_rng_input = true;
            }
            Some(p_i) => {
                let the_opt_player = &game_state.players[p_i as usize];
                if let OptionallyEnabledPlayer::Enabled(p) = the_opt_player {
                    let p_bound = get_bound_of_character(&p.character);
                    let npc_bound: AbsoluteBoundingBox<i32, u32> =
                        get_bound_of_character(current_npc);
                    let needs_teleport;
                    {
                        // teleportAyh-shon if needed
                        const TELEPORT_AXIS_MIN_DIST: u32 = SCREEN_HEIGHT_PX as u32;
                        needs_teleport = p_bound.x.abs_diff(npc_bound.x) > TELEPORT_AXIS_MIN_DIST
                            || p_bound.y.abs_diff(npc_bound.y) > TELEPORT_AXIS_MIN_DIST;
                    }

                    if needs_teleport {
                        current_npc.x_pos = p_bound.x as f32;
                        current_npc.y_pos = p_bound.y as f32;
                        current_npc.x_vel = 0.0;
                        current_npc.y_vel = 0.0;
                    } else {
                        if rng.next_for_input() % 10 > 1 {
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
                                } else if current_npc.x_pos > ch.x_pos + p_bound.width as f32 {
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
    });

    // MOVE NPCS
    {
        for (i, npc) in game_state.npcs.iter_mut().enumerate() {
            update_pos(
                &game_state.map,
                MovingEntityMut::Npc(npc),
                inputs[i],
                game_state.godmode,
                &mut game_state.clouds,
            );
        }
    }

    // UPDATE CLOUDS
    Cloud::update_clouds(&mut game_state.clouds);

    // Depending on what gamemode we're in, we do different update steps.
    {
        match &mut game_state.game_mode {
            GameMode::NormalPlay(play_mode) => {
                // COUNT THE NUMBER OF NPCS THAT ARE FOLLOWING PLAYERS
                let current_found_npcs: u32 = game_state.npcs.iter().fold(0, |acc, e| {
                    acc + match e.following_i {
                        None => 0,
                        Some(_) => 1,
                    }
                });

                // UPDATE & DRAW POPUPS
                game_state.popup_text_ringbuffer.update_popup_positions();

                // USE ABILITY CARDS
                if !showing_modal {
                    for (p_i, pr) in game_state.players.iter_mut().enumerate() {
                        match pr {
                            OptionallyEnabledPlayer::Enabled(p) => {
                                if !showing_modal && btns_pressed_this_frame[p_i] & BUTTON_2 != 0 {
                                    let res = p.card_stack.try_use_cards();
                                    let added_t;
                                    let popup_t: Option<String>;
                                    let popup_icon: PopupIcon;
                                    match res {
                                    game::ability_cards::AbilityCardUsageResult::NothingHappened => {
                                        added_t = 0;
                                        popup_t = None;
                                        popup_icon = PopupIcon::None;
                                    },
                                    game::ability_cards::AbilityCardUsageResult::GainedTime(t) => {
                                        added_t = t;
                                        popup_t = Some(format![" +{}", t]);
                                        popup_icon = PopupIcon::Clock;
                                    },
                                    game::ability_cards::AbilityCardUsageResult::EnabledFlyAndTime(t) => {
                                        if p.character.can_fly {
                                            added_t = t;
                                            popup_t = Some(format![" +{}", t]);
                                            popup_icon = PopupIcon::Clock;
                                        } else {
                                            p.character.can_fly = true;
                                            added_t = t;
                                            popup_t = Some("fly!".to_string());
                                            popup_icon = PopupIcon::None;
                                        }

                                    },
                                    game::ability_cards::AbilityCardUsageResult::EnabledWarpAndTime(t) => {
                                        if p.character.warp_ability == WarpAbility::CannotWarp {
                                            p.character.warp_ability = WarpAbility::CanWarp(WarpState::Charging(0));
                                            added_t = t - 10;
                                            popup_t = Some("hold   : warp".to_string());
                                            popup_icon = PopupIcon::DownArrow;
                                        } else {
                                            added_t = 10;
                                            popup_t = Some(format![" +{}", t]);
                                            popup_icon = PopupIcon::Clock;
                                        }
                                    }
                                }
                                    if let Some(pt) = popup_t {
                                        // spawn some clouds
                                        for dir in [
                                            (1.0, 0.0),
                                            (0.5, 0.86),
                                            (-0.5, 0.86),
                                            (-1.0, 0.0),
                                            (-0.5, -0.86),
                                            (0.5, -0.86),
                                        ] {
                                            const CARD_CLOUD_SPEED: f32 = 4.0;

                                            let vx = CARD_CLOUD_SPEED * dir.0;
                                            let vy = CARD_CLOUD_SPEED * dir.1;
                                            Cloud::try_push_cloud(
                                                &mut game_state.clouds,
                                                p.character.x_pos + 2.0,
                                                p.character.y_pos + 3.0,
                                                vx,
                                                vy,
                                            );
                                        }
                                        game_state.popup_text_ringbuffer.add_new_popup(
                                            p.character.x_pos - 14.0,
                                            p.character.y_pos,
                                            pt,
                                            popup_icon,
                                        );
                                    }
                                    game_state.countdown_timer_msec += added_t * 60;
                                    game_state.countdown_timer_msec =
                                        game_state.countdown_timer_msec.min(100 * 60 - 1);
                                    game_state.score += added_t;
                                }
                            }
                            OptionallyEnabledPlayer::Disabled => {}
                        }
                    }
                }

                // MOVE ABILITY CARD POSITIONS
                match &mut game_state.players[player_idx as usize] {
                    OptionallyEnabledPlayer::Enabled(p) => {
                        for (i, card) in p.card_stack.cards.iter_mut().enumerate() {
                            if let Some(c) = card {
                                c.target_x = (80 + 15 * i) as f32;
                                c.target_y = 1.0;
                            }
                        }
                        p.card_stack.move_cards();
                    }
                    OptionallyEnabledPlayer::Disabled => {}
                }

                // ADVANCE MODAL DIALOGS
                if showing_modal {
                    match play_mode {
                        NormalPlayModes::MainGameplay => {
                            unreachable!()
                        }
                        NormalPlayModes::HoverModal(m) => {
                            m.update(game_state.song_timer as f32 * 0.05f32);

                            if m.options_ready_to_select() {
                                // let cursor_opt: u8;
                                let mut btn_pressed: bool = false;
                                // cursor_opt = *option;
                                if btns_pressed_this_frame[0] & (BUTTON_1 | BUTTON_2) != 0 {
                                    btn_pressed = true
                                }
                                match m.menu_type {
                                    MenuTypes::WonLevel => {
                                        if btn_pressed {
                                            game_state.difficulty_level += 1;
                                            // game_state.game_mode =
                                            //     GameMode::NormalPlay(NormalPlayModes::MainGameplay);
                                            game_state.game_mode = GameMode::NormalPlay(
                                                NormalPlayModes::HoverModal(Modal::new(
                                                    AbsoluteBoundingBox {
                                                        x: 45,
                                                        y: 40,
                                                        width: 70,
                                                        height: 50,
                                                    },
                                                    MenuTypes::StartLevel,
                                                )),
                                            );
                                            game_state.regenerate_map();
                                        }
                                    }
                                    MenuTypes::StartLevel => {
                                        let mut start_normal_play = false;
                                        if m.text_timer() > 100 {
                                            start_normal_play = true;
                                        }

                                        if btn_pressed {
                                            start_normal_play = true;
                                        }
                                        if start_normal_play {
                                            game_state.game_mode =
                                                GameMode::NormalPlay(NormalPlayModes::MainGameplay);
                                        }
                                    }
                                    MenuTypes::Done => {
                                        if btn_pressed {
                                            game_state.difficulty_level = START_DIFFICULTY_LEVEL;
                                            game_state.game_mode = GameMode::StartScreen;
                                        }
                                    }
                                    MenuTypes::WonGame => {
                                        if btn_pressed {
                                            game_state.difficulty_level = START_DIFFICULTY_LEVEL;
                                            game_state.game_mode = GameMode::StartScreen;
                                        }
                                    }
                                    MenuTypes::StartGameMessage => {
                                        if btn_pressed {
                                            game_state.game_mode =
                                                GameMode::NormalPlay(NormalPlayModes::MainGameplay);
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // HELP TEXT AT START OF GAME
                    if game_state.difficulty_level == 1
                        && game_state.countdown_timer_msec == COUNTDOWN_TIMER_START - 1
                        && game_state.tutorial_text_counter == 0
                    {
                        game_state.tutorial_text_counter += 1;
                        game_state.game_mode =
                            GameMode::NormalPlay(NormalPlayModes::HoverModal(Modal::new(
                                AbsoluteBoundingBox {
                                    x: 10,
                                    y: 10,
                                    width: 140,
                                    height: 140,
                                },
                                MenuTypes::StartGameMessage,
                            )));
                    }

                    // ------- LEVEL WIN CONDITION -----------
                    if game_state.total_npcs_to_find == current_found_npcs {
                        if game_state.difficulty_level == FINAL_LEVEL {
                            game_state.game_mode =
                                GameMode::NormalPlay(NormalPlayModes::HoverModal(Modal::new(
                                    AbsoluteBoundingBox {
                                        x: 25,
                                        y: 35,
                                        width: 110,
                                        height: 65,
                                    },
                                    MenuTypes::WonGame,
                                )));
                            game_state.song_idx = 0;
                        } else {
                            game_state.game_mode =
                                GameMode::NormalPlay(NormalPlayModes::HoverModal(Modal::new(
                                    AbsoluteBoundingBox {
                                        x: 40,
                                        y: 40,
                                        width: 80,
                                        height: 40,
                                    },
                                    MenuTypes::WonLevel,
                                )));
                            game_state.song_idx = 0;
                        }

                        game_state.song_timer = 0;
                    }

                    // PROGRESS TIME, CHECK FOR GAME END
                    if !game_state.countdown_paused {
                        game_state.speedrun_timer_msec += 1;
                        game_state.countdown_timer_msec -= 1;

                        // ---- LOSE CONDITION ----
                        if let RunType::TimedMode = game_state.settings.run_type {
                            if game_state.countdown_timer_msec == 0 {
                                game_state.song_idx = 0;

                                game_state.game_mode =
                                    GameMode::NormalPlay(NormalPlayModes::HoverModal(Modal::new(
                                        AbsoluteBoundingBox {
                                            x: 15,
                                            y: 50,
                                            width: 130,
                                            height: 60,
                                        },
                                        MenuTypes::Done,
                                    )));
                            }
                        }
                    }
                }
            }
            GameMode::StartScreen => {
                // SETUP TITLE MUSIC AND COLORS
                game_state.song_idx = 1;

                game_state.rng.next_for_input();
                if btns_pressed_this_frame[0] != 0 {
                    // game_state.game_mode = GameMode::NormalPlay(NormalPlayModes::MainGameplay);
                    game_state.game_mode = GameMode::SelectScreen(SelectSetup {
                        current_selection: SelectMenuFocuses::RunType,
                    });
                    // game_state.regenerate_map();
                }
            }
            GameMode::SelectScreen(select_setup) => match select_setup.current_selection {
                SelectMenuFocuses::RunType => {
                    if btns_pressed_this_frame[0] & (BUTTON_UP) != 0 {
                        game_state.menu_idx -= 1;
                    }
                    if btns_pressed_this_frame[0] & (BUTTON_DOWN) != 0 {
                        game_state.menu_idx += 1;
                    }
                    game_state.menu_idx %= 4;

                    if btns_pressed_this_frame[0] & (BUTTON_RIGHT | BUTTON_LEFT) != 0 {
                        game_state.settings.run_type = match game_state.menu_idx {
                            0 => RunType::Casual,
                            1 => RunType::TimedMode,
                            2 => RunType::Speedrun(0),
                            _ => RunType::Casual,
                        }
                    }

                    if btns_pressed_this_frame[0] & (BUTTON_2) != 0 {
                        if let game::game_state::RunType::Speedrun(n) = game_state.settings.run_type
                        {
                            game_state.settings.run_type =
                                game::game_state::RunType::Speedrun(n + 1);
                        }
                    }

                    if btns_pressed_this_frame[0] & BUTTON_1 != 0 {
                        game_state.game_mode = GameMode::NormalPlay(NormalPlayModes::MainGameplay);
                        if let game::game_state::RunType::Speedrun(n) = game_state.settings.run_type
                        {
                            game_state.rng =
                                GameRng::FixedSeed(Rng::new_from_seed(n), Rng::new_from_seed(n));
                        }
                        game_state.regenerate_map();
                    }
                }
            },
        }
    }
    draw_game(game_state, player_idx);
}
