use crate::game::camera::Camera;
use crate::game::collision::AbsoluteBoundingBox;
use crate::game::entities::{Character, KittyStates, MovingEntity, OptionallyEnabledPlayer};
use crate::game::game_constants::{
    INCR_VERSION, MAJOR_VERSION, MINOR_VERSION, SCREEN_HEIGHT_PX, SCREEN_WIDTH_PX, TILE_HEIGHT_PX,
    TILE_WIDTH_PX,
};
use crate::game::game_map::MAP_TILESETS;
use crate::game::menus::{MenuTypes, Modal};
use crate::game::popup_text::{PopTextRingbuffer, PopupIcon};
use crate::title_ss::{
    OUTPUT_ONLINEPNGTOOLS, OUTPUT_ONLINEPNGTOOLS_FLAGS, OUTPUT_ONLINEPNGTOOLS_HEIGHT,
    OUTPUT_ONLINEPNGTOOLS_WIDTH,
};
use crate::{spritesheet, wasm4::*};

use crate::game::game_state::{GameMode, GameState, GameStateText, RunType};

/// draw the tiles in the map, relative to the camera.
pub fn draw_map(game_state: &GameState) {
    let map = &game_state.map;
    let camera = &game_state.camera;

    let tileset = &MAP_TILESETS[game_state.tileset_idx];

    for chunk in &map.chunks {
        for row in 0..chunk.bound.height {
            for col in 0..chunk.bound.width {
                let map_tile_i = chunk.get_tile(col, row);
                match map_tile_i {
                    0 => {}
                    tile_idx => {
                        let tile_i: usize = tileset[tile_idx as usize] as usize; // *tile_idx as usize;
                        if tile_i == 0 {
                            continue;
                        } // trace(format!("Tile {tile_i}"));
                        let chunk_x_offset: i32 = (TILE_WIDTH_PX) as i32 * chunk.bound.x;
                        let chunk_y_offset: i32 = (TILE_HEIGHT_PX) as i32 * chunk.bound.y;
                        let x_loc = (chunk_x_offset + col as i32 * TILE_HEIGHT_PX as i32)
                            - camera.current_viewing_x_offset as i32;
                        let y_loc = (chunk_y_offset + row as i32 * TILE_WIDTH_PX as i32)
                            - camera.current_viewing_y_offset as i32;

                        if x_loc >= 0 - TILE_WIDTH_PX as i32
                            && x_loc < SCREEN_WIDTH_PX as i32
                            && y_loc >= 0 - TILE_HEIGHT_PX as i32
                            && y_loc < SCREEN_HEIGHT_PX as i32
                        {
                            blit_sub(
                                game_state.spritesheet,
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

pub fn prep_drawing(game_state: &GameState) {
    // PREPARE TO RENDER THE MAP & ENTITIES
    unsafe {
        *PALETTE = spritesheet::KITTY_SPRITESHEET_PALETTES[game_state.pallette_idx];
    }
    unsafe { *DRAW_COLORS = spritesheet::KITTY_SPRITESHEET_DRAW_COLORS }
}

// just draw a spriteframe at a location. Put a colored layer behind it, like layertext() does.
fn draw_spriteframe(
    spritesheet: &[u8],
    spriteframe: &spritesheet::SpriteFrame,
    spritesheet_stride: u32,
    x: i32,
    y: i32,
) {
    let cf = spriteframe;
    for (xx, yy, colors) in [
        (x, y, 0x1111),
        (x + 1, y + 1, 0x1111),
        (x, y, spritesheet::KITTY_SPRITESHEET_DRAW_COLORS),
    ] {
        unsafe { *DRAW_COLORS = colors }
        blit_sub(
            spritesheet,
            xx,
            yy,
            cf.width as u32,
            cf.height as u32,
            cf.start_x as u32,
            cf.start_y as u32,
            spritesheet_stride,
            spritesheet::KITTY_SPRITESHEET_FLAGS,
        );
    }
}

pub fn draw_clouds(game_state: &GameState) {
    // DRAW CLOUDS
    for cloud in game_state.clouds.iter() {
        let cam: &Camera = &game_state.camera;
        let cloud_sprite: &spritesheet::Sprite =
            spritesheet::Sprite::from_preset(&spritesheet::PresetSprites::Cloud);
        blit_sub(
            game_state.spritesheet,
            (cloud.x - cam.current_viewing_x_offset) as i32,
            (cloud.y - cam.current_viewing_y_offset) as i32,
            cloud_sprite.frames[0].width as u32,
            cloud_sprite.frames[0].height as u32,
            cloud_sprite.frames[0].start_x as u32,
            cloud_sprite.frames[0].start_y as u32,
            game_state.spritesheet_stride as u32,
            spritesheet::KITTY_SPRITESHEET_FLAGS
                | if cloud.vx <= 0.0 { 0 } else { BLIT_FLIP_X }
                | if cloud.vy >= 0.0 { 0 } else { BLIT_FLIP_Y },
        );
    }
}

pub fn draw_popup_text(game_state: &GameState) {
    let popup_texts_rb: &PopTextRingbuffer = &game_state.popup_text_ringbuffer;
    let camera = game_state.camera;
    popup_texts_rb.texts.iter().flatten().for_each(|p| {
        const T_BEFORE_BLINK: u32 = 60;
        if p.duration_timer < T_BEFORE_BLINK || p.duration_timer % 6 < 3 {
            let (dx, dy) = (
                (p.x_pos - camera.current_viewing_x_offset) as i32,
                (p.y_pos - camera.current_viewing_y_offset) as i32,
            );
            layertext(&p.text, dx, dy);
            match p.icon {
                PopupIcon::None => {}
                PopupIcon::Clock => match game_state.settings.run_type {
                    RunType::TimedMode => {
                        draw_spriteframe(
                            game_state.spritesheet,
                            &spritesheet::Sprite::from_preset(&spritesheet::PresetSprites::Clock)
                                .frames[0],
                            game_state.spritesheet_stride as u32,
                            dx,
                            dy - 1,
                        );
                    }
                    _ => {
                        layertext("Sc", dx - 8, dy);
                    }
                },
                PopupIcon::CatHead => draw_spriteframe(
                    game_state.spritesheet,
                    &spritesheet::Sprite::from_preset(&spritesheet::PresetSprites::CatHead).frames
                        [0],
                    game_state.spritesheet_stride as u32,
                    dx + 1,
                    dy + 1,
                ),
                PopupIcon::DownArrow => {
                    text(*b"\x87", dx + 40, dy);
                }
            }
        }
    });
}

/// Draw text with a soft background under
fn layertext(t: &str, x: i32, y: i32) {
    unsafe { *DRAW_COLORS = 0x0001 }
    text(t, x + 1, y);
    text(t, x, y + 1);
    text(t, x + 1, y + 1);
    unsafe { *DRAW_COLORS = 0x0002 }

    text(t, x, y);
}

pub fn draw_ability_cards(game_state: &GameState, player_idx: u8) {
    // DRAW ABILITY CARDS
    unsafe { *DRAW_COLORS = spritesheet::KITTY_SPRITESHEET_DRAW_COLORS }
    match &game_state.players[player_idx as usize] {
        OptionallyEnabledPlayer::Enabled(p) => {
            for card in p.card_stack.cards.iter() {
                if let Some(c) = &card {
                    // trace(&format!["{}", i]);
                    blit_sub(
                        game_state.spritesheet,
                        c.x_pos as i32,
                        c.y_pos as i32,
                        c.sprite.frames[0].width as u32,
                        c.sprite.frames[0].height as u32,
                        c.sprite.frames[0].start_x as u32,
                        c.sprite.frames[0].start_y as u32,
                        (game_state.spritesheet_stride) as u32,
                        spritesheet::KITTY_SPRITESHEET_FLAGS,
                    );
                }
            }
        }
        OptionallyEnabledPlayer::Disabled => {}
    }
}

/// DRAW BLURRED BACKGROUND BEHIND SCORE AND TIME TEXTS IN-GAME
fn draw_modal_bg(pf: &AbsoluteBoundingBox<f32, f32>, style: u8, color: u16) {
    unsafe { *DRAW_COLORS = color }
    let p: AbsoluteBoundingBox<i32, u32> = AbsoluteBoundingBox {
        x: pf.x as i32,
        y: pf.y as i32,
        width: pf.width as u32,
        height: pf.height as u32,
    };

    unsafe {
        *DRAW_COLORS = 0x0001;
    }

    if let 1 = style {
        rect(p.x, p.y, p.width, p.height);
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

    unsafe { *DRAW_COLORS = color }
    // borders

    if style == 1 {
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
}

enum ModalTextStyling {
    Normal,
    Bold,
}

fn draw_modal_text(m: &Modal, st: &str, x: i32, y: i32, styling: Option<ModalTextStyling>) {
    let style_to_apply = styling.unwrap_or(ModalTextStyling::Normal);
    let color = match style_to_apply {
        ModalTextStyling::Normal => 0x0002,
        ModalTextStyling::Bold => 0x0004,
    };
    unsafe { *DRAW_COLORS = color }
    text(
        st,
        m.actual_position.x as i32 + x,
        m.actual_position.y as i32 + y,
    );
}

pub fn draw_maingame_status_shadows(game_state: &GameState) {
    let GameMode::NormalPlay = &game_state.game_mode else {
        return;
    };

    // Draw blur sections for the status bars on the bottom and top of the screen.
    draw_modal_bg(
        &AbsoluteBoundingBox {
            x: -1.0,
            y: 0.0,
            width: 162.0,
            height: 10.0,
        },
        0,
        0x0001,
    );

    draw_modal_bg(
        &AbsoluteBoundingBox {
            x: -1.0,
            y: 150.0,
            width: 162.0,
            height: 10.0,
        },
        0,
        0x0001,
    );
}

const TOP_UI_TEXT_Y: i32 = 2;
const BOTTOM_UI_TEXT_Y: i32 = SCREEN_HEIGHT_PX as i32 - 8;

pub fn draw_statuses(game_state: &GameState) {
    let GameStateText {
        world_level_text,
        score_text,
        speedrun_seed_text,
        found_kitties_text,
        time_left_text,
    } = game_state.get_texts();
    // DRAW SCORE, LEVEL, # KITTIES during normal play
    layertext(&world_level_text, 0, BOTTOM_UI_TEXT_Y);
    layertext(&score_text, 60, BOTTOM_UI_TEXT_Y);
    layertext(&found_kitties_text, 9, TOP_UI_TEXT_Y);
    match game_state.settings.run_type {
        RunType::TimedMode => {
            draw_spriteframe(
                game_state.spritesheet,
                &spritesheet::Sprite::from_preset(&spritesheet::PresetSprites::Clock).frames[0],
                game_state.spritesheet_stride as u32,
                48,
                TOP_UI_TEXT_Y - 1,
            );
            layertext(&time_left_text, 9 + 6 * 8, TOP_UI_TEXT_Y);
        }

        RunType::Speedrun(_) => {
            layertext(&speedrun_seed_text, 1, TOP_UI_TEXT_Y + 10);
        }

        _ => {}
    }
    draw_spriteframe(
        game_state.spritesheet,
        &spritesheet::Sprite::from_preset(&spritesheet::PresetSprites::CatHead).frames[0],
        game_state.spritesheet_stride as u32,
        1,
        TOP_UI_TEXT_Y + 1,
    );
}

pub fn draw_modals(game_state: &GameState) {
    let GameStateText {
        world_level_text,
        score_text,
        speedrun_seed_text,
        ..
    } = &game_state.get_texts();

    let Some(m) = &game_state.menu else {
        return;
    };

    draw_modal_bg(&m.actual_position, 1, 0x0002);

    let text_timer = m.text_timer();

    let modal_offs = |x: i32, y: i32| {
        (
            m.actual_position.x as i32 + x,
            m.actual_position.y as i32 + y,
        )
    };

    if m.ready_to_show_text() {
        // let cursor_opt: u8;
        match m.menu_type {
            MenuTypes::WonLevel => {
                const BLINK_START: u32 = 50;
                const BLINK_TITLE_PERIOD: u32 = 17;
                if text_timer < BLINK_START || (text_timer / BLINK_TITLE_PERIOD).is_multiple_of(2) {
                    // draw_modal_text("Found!!", 12, 15, None);
                    draw_modal_text(m, &world_level_text, 16, 12, None);
                    draw_modal_text(m, "Clear!", 16, 22, None);
                }
            }
            MenuTypes::StartLevel => {
                draw_modal_text(m, &world_level_text, 16, 12, None);
                draw_modal_text(m, "Start!", 16, 22, None);
                draw_modal_text(
                    m,
                    &format!["+{}", game_state.countdown_and_score_bonus],
                    33,
                    35,
                    None,
                );
                let (xx, yy) = modal_offs(25, 34);

                match game_state.settings.run_type {
                    RunType::TimedMode => {
                        draw_spriteframe(
                            game_state.spritesheet,
                            &spritesheet::Sprite::from_preset(&spritesheet::PresetSprites::Clock)
                                .frames[0],
                            game_state.spritesheet_stride as u32,
                            xx,
                            yy,
                        );
                    }
                    _ => {
                        draw_modal_text(m, "Sc", 33 - 2 * 8, 35, None);
                    }
                }
            }
            MenuTypes::Done => {
                const BLINK_START: u32 = 50;
                const BLINK_TITLE_PERIOD: u32 = 17;
                if text_timer < BLINK_START || (text_timer / BLINK_TITLE_PERIOD).is_multiple_of(2) {
                    draw_modal_text(m, "Time's Up!", 20, 14, None);
                }

                draw_modal_text(m, &format!["End: {}", &world_level_text], 8, 30, None);
                draw_modal_text(m, &score_text, 8, 40, None);
            }
            MenuTypes::WonGame => {
                const BLINK_START: u32 = 50;
                const BLINK_TITLE_PERIOD: u32 = 17;
                if text_timer < BLINK_START || (text_timer / BLINK_TITLE_PERIOD).is_multiple_of(2) {
                    draw_modal_text(m, "YOU WON!!!", 20, 14, None);
                }

                draw_modal_text(m, &format!["End: {}", world_level_text], 8, 30, None);
                draw_modal_text(m, &score_text, 8, 40, None);

                if let RunType::Speedrun(_) = game_state.settings.run_type {
                    draw_modal_text(m, &speedrun_seed_text, 8, 50, None);
                }
            }
            MenuTypes::StartGameMessage => {
                draw_modal_text(m, "-- GOAL --", 30, 10, Some(ModalTextStyling::Bold));
                draw_modal_text(m, "Find all the", 20, 25, None);
                draw_modal_text(m, "kitties in time!", 10, 40, None);
                draw_modal_text(m, "-- CONTROLS --", 14, 100, Some(ModalTextStyling::Bold));

                draw_modal_text(m, "     to move,", 24, 114, None);
                draw_modal_text(m, " =jump,  =card", 16, 126, None);
                let (xx, yy) = modal_offs(0, 0);

                if game_state.song_timer % 30 >= 15 {
                    unsafe { *DRAW_COLORS = 0x0004 }
                    text(*b"\x84", xx + 32, yy + 114);
                    text(*b"\x85", xx + 48, yy + 114);
                    text(*b"\x80", xx + 15, yy + 126);
                    text(*b"\x81", xx + 79, yy + 126);
                }

                draw_spriteframe(
                    game_state.spritesheet,
                    &spritesheet::Sprite::from_preset(&spritesheet::PresetSprites::CatHead).frames
                        [0],
                    game_state.spritesheet_stride as u32,
                    xx + 20,
                    yy + 62,
                );

                draw_modal_text(m, " = # kittes", 28, 62, None);

                if let RunType::TimedMode = game_state.settings.run_type {
                    draw_spriteframe(
                        game_state.spritesheet,
                        &spritesheet::Sprite::from_preset(&spritesheet::PresetSprites::Clock)
                            .frames[0],
                        game_state.spritesheet_stride as u32,
                        xx + 20,
                        yy + 78,
                    );
                    draw_modal_text(m, " = time left", 28, 78, None);
                }
            }
            MenuTypes::Setup => {
                fn get_run_type_text(run_type: RunType) -> String {
                    match run_type {
                        RunType::Casual => "casual".to_owned(),
                        RunType::TimedMode => "Timed".to_owned(),
                        RunType::Speedrun(_) => "Seeded".to_owned(),
                        RunType::Chaos => "???".to_owned(),
                    }
                }
                let style_for_run_type = |rt: &RunType| {
                    if std::mem::discriminant(rt)
                        == std::mem::discriminant(&game_state.settings.run_type)
                    {
                        Some(ModalTextStyling::Bold)
                    } else {
                        None
                    }
                };
                draw_spriteframe(
                    game_state.spritesheet,
                    &spritesheet::Sprite::from_preset(&spritesheet::PresetSprites::CatHead).frames
                        [0],
                    game_state.spritesheet_stride as u32,
                    20,
                    62,
                );
                draw_modal_text(
                    m,
                    &get_run_type_text(RunType::Casual),
                    24,
                    10,
                    style_for_run_type(&RunType::Casual),
                );
                draw_modal_text(
                    m,
                    &get_run_type_text(RunType::TimedMode),
                    24,
                    25,
                    style_for_run_type(&RunType::TimedMode),
                );
                draw_modal_text(
                    m,
                    &get_run_type_text(RunType::Speedrun(0)),
                    24,
                    40,
                    style_for_run_type(&RunType::Speedrun(0)),
                );
                draw_modal_text(
                    m,
                    &get_run_type_text(RunType::Chaos),
                    24,
                    55,
                    style_for_run_type(&RunType::Chaos),
                );
            }
        }
    }
}

/// Draw a character on-screen, relative to the camera.
fn draw_character(
    spritesheet: &[u8],
    spritesheet_stride: &usize,
    camera: &Camera,
    character: MovingEntity,
) {
    let the_char: &Character;

    match character {
        MovingEntity::OptionalPlayer(optionally_enabled_player) => {
            match optionally_enabled_player {
                OptionallyEnabledPlayer::Enabled(p) => {
                    the_char = &p.character;
                }
                OptionallyEnabledPlayer::Disabled => return,
            }
        }
        MovingEntity::Npc(npc) => {
            the_char = npc;
        }
    }

    let i = the_char.current_sprite_i as usize;
    blit_sub(
        spritesheet,
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
                KittyStates::OnCeiling(_) => BLIT_FLIP_Y,
                _ => 0,
            },
    );
}

fn render_title(game_state: &GameState, y: i32) {
    // RENDER THE TITLE
    unsafe { *DRAW_COLORS = 0x0034 }
    const TITLE_X: i32 = 5;
    let title_y_osc = match game_state.song_timer {
        0..=TIMER_INTERACTIVE_START => 0,
        _ => {
            (5f32
                * num::Float::sin(
                    (game_state.song_timer - TIMER_INTERACTIVE_START) as f32 * 0.05f32,
                )) as i32
        }
    };
    for row in 0..OUTPUT_ONLINEPNGTOOLS_HEIGHT as i32 {
        blit_sub(
            &OUTPUT_ONLINEPNGTOOLS,
            TITLE_X
                + (3000000f32
                    * (1f32 / (1f32 + num::Float::powf(game_state.song_timer as f32, 3f32)))
                    * num::Float::sin((game_state.song_timer as f32 + row as f32 * 4f32) * 0.1f32))
                    as i32,
            y + title_y_osc + row,
            OUTPUT_ONLINEPNGTOOLS_WIDTH,
            1,
            0,
            row as u32,
            OUTPUT_ONLINEPNGTOOLS_WIDTH,
            OUTPUT_ONLINEPNGTOOLS_FLAGS,
        )
    }
    unsafe {
        *PALETTE = spritesheet::KITTY_SPRITESHEET_PALETTES[game_state.pallette_idx];
    }
    unsafe { *DRAW_COLORS = spritesheet::KITTY_SPRITESHEET_DRAW_COLORS }
}

const TIMER_INTERACTIVE_START: u32 = 100;
const TITLE_Y: i32 = 15;

pub fn draw_game(game_state: &GameState, player_idx: u8) {
    prep_drawing(game_state);

    // MOVE AND RENDER THE PLAYERS
    {
        let optional_players: &[OptionallyEnabledPlayer; 4] = &game_state.players;
        for optional_player in optional_players.iter() {
            draw_character(
                game_state.spritesheet,
                &game_state.spritesheet_stride,
                &game_state.camera,
                MovingEntity::OptionalPlayer(optional_player),
            );
        }
    }

    // DRAW NPCS
    for npc in game_state.npcs.iter() {
        draw_character(
            game_state.spritesheet,
            &game_state.spritesheet_stride,
            &game_state.camera,
            MovingEntity::Npc(npc),
        );
    }

    // ------ RENDER THE MAP -----------
    draw_map(game_state);

    draw_clouds(game_state);

    draw_maingame_status_shadows(game_state);

    draw_modals(game_state);
    // Depending on what gamemode we're in, we do different update steps.
    match &game_state.game_mode {
        GameMode::NormalPlay => {
            // DRAW POPUPS
            draw_popup_text(game_state);

            draw_ability_cards(game_state, player_idx);

            draw_statuses(game_state);
        }
        GameMode::StartScreen => {
            // SETUP TITLE MUSIC AND COLORS
            unsafe { *DRAW_COLORS = 0x0002 }

            // SHOW TITLE-SCREEN SUBTEXT
            if game_state.menu.is_none() {
                if game_state.song_timer >= TIMER_INTERACTIVE_START {
                    draw_modal_bg(
                        &AbsoluteBoundingBox {
                            x: 15.0,
                            y: 105.0,
                            width: 130.0,
                            height: 40.0,
                        },
                        0,
                        0x0001,
                    );
                    unsafe { *DRAW_COLORS = 0x0004 };
                    if game_state.song_timer % 30 >= 15 {
                        text("Any key: play", 24, 110);
                    }

                    unsafe { *DRAW_COLORS = 0x0002 };
                    text("by CanyonTurtle", 20, 125);
                    text(" & BurntSugar  ", 20, 135);
                    text(
                        format!["ver. {}.{}.{}", MAJOR_VERSION, MINOR_VERSION, INCR_VERSION],
                        40,
                        150,
                    );
                }

                render_title(game_state, TITLE_Y);
            }
        }
    }
}
