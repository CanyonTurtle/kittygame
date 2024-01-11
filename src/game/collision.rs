
use crate::{
    game::{ability_cards::AbilityCardTypes, entities::{WarpAbility, WarpState}, popup_text::{PopTextRingbuffer, PopupIcon}},
    spritesheet,
};

use super::{
    entities::{Character, KittyStates, OptionallyEnabledPlayer},
    game_constants::{
        TILE_HEIGHT_PX, TILE_WIDTH_PX, X_LEFT_BOUND, X_RIGHT_BOUND, Y_LOWER_BOUND, Y_UPPER_BOUND,
    },
    game_map::GameMap,
    game_state::GameState,
    mapchunk::{MapChunk, TileAlignedBoundingBox}, cloud::Cloud,
};

use crate::wasm4::*;

pub fn check_absolute_point_inside_tile_aligned_bound(
    x: i32,
    y: i32,
    bound: &TileAlignedBoundingBox,
) -> bool {
    let bound_absolute_left_x: i32 = bound.x * TILE_WIDTH_PX as i32;
    let bound_absolute_right_x: i32 =
        bound_absolute_left_x + bound.width as i32 * TILE_WIDTH_PX as i32;
    let bound_absolute_lower_y: i32 = bound.y * TILE_HEIGHT_PX as i32;
    let bound_absolute_upper_y: i32 =
        bound_absolute_lower_y + bound.height as i32 * TILE_HEIGHT_PX as i32;

    if x > bound_absolute_left_x {
        if x < bound_absolute_right_x {
            if y > bound_absolute_lower_y {
                if y < bound_absolute_upper_y {
                    return true;
                }
            }
        }
    }
    false
}

#[derive(Clone)]
pub struct AbsoluteBoundingBox<P, W> {
    pub x: P,
    pub y: P,
    pub width: W,
    pub height: W,
}
pub fn check_absolue_bound_partially_inside_tile_aligned_bound(
    absolute_bound: &AbsoluteBoundingBox<i32, u32>,
    tile_aligned_bound: &TileAlignedBoundingBox,
) -> bool {
    let lowerleft = (absolute_bound.x, absolute_bound.y);
    let lowerright = (
        absolute_bound.x + absolute_bound.width as i32,
        absolute_bound.y,
    );
    let upperleft = (
        absolute_bound.x,
        absolute_bound.y + absolute_bound.height as i32,
    );
    let upperright = (
        absolute_bound.x + absolute_bound.width as i32,
        absolute_bound.y + absolute_bound.height as i32,
    );
    if !check_absolute_point_inside_tile_aligned_bound(lowerleft.0, lowerleft.1, tile_aligned_bound)
    {
        if !check_absolute_point_inside_tile_aligned_bound(
            lowerright.0,
            lowerright.1,
            tile_aligned_bound,
        ) {
            if !check_absolute_point_inside_tile_aligned_bound(
                upperleft.0,
                upperleft.1,
                tile_aligned_bound,
            ) {
                if !check_absolute_point_inside_tile_aligned_bound(
                    upperright.0,
                    upperright.1,
                    tile_aligned_bound,
                ) {
                    return false;
                }
            }
        }
    }
    true
}

pub fn check_absolute_bounding_box_partially_inside_another(
    bound: &AbsoluteBoundingBox<i32, u32>,
    other: &AbsoluteBoundingBox<i32, u32>,
) -> bool {
    bound.x + bound.width as i32 > other.x
        && bound.x < other.x + other.width as i32
        && bound.y + bound.height as i32 > other.y
        && bound.y < other.y + other.height as i32
}

pub fn get_bound_of_character(character: &Character) -> AbsoluteBoundingBox<i32, u32> {
    let char_positioning = character.sprite.frames[character.current_sprite_i as usize];
    AbsoluteBoundingBox {
        x: character.x_pos as i32,
        y: character.y_pos as i32,
        width: char_positioning.width as u32,
        height: char_positioning.height as u32,
    }
}

pub fn check_entity_collisions(game_state: GameState) -> GameState {
    let mut new_game_state = game_state;
    const N_PLAYER_NPC_COLLISIONS_TO_CHECK_AT_MOST: usize = 10;
    let mut npc_hitlist: [(u8, u8); N_PLAYER_NPC_COLLISIONS_TO_CHECK_AT_MOST] =
        [(0, 0); N_PLAYER_NPC_COLLISIONS_TO_CHECK_AT_MOST];
    let mut hitlist_i: u8 = 0;
    for (i, opt_p) in new_game_state.players.iter().enumerate() {
        if let OptionallyEnabledPlayer::Enabled(p) = opt_p {
            for (j, npc2) in new_game_state.npcs.iter().enumerate() {
                let did_hit: bool;
                {
                    let player_bound = get_bound_of_character(&p.character);

                    let npc2_bound = get_bound_of_character(npc2);
                    did_hit = check_absolute_bounding_box_partially_inside_another(
                        &player_bound,
                        &npc2_bound,
                    );
                }
                match did_hit {
                    true => {
                        // npcs hit
                        if hitlist_i < npc_hitlist.len() as u8 {
                            npc_hitlist[hitlist_i as usize] = (i as u8, j as u8);
                            hitlist_i += 1;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    for (hit_p_i, hit_npc_i) in &npc_hitlist[..hitlist_i as usize] {
        let opt_p = &mut new_game_state.players[*hit_p_i as usize];

        if let OptionallyEnabledPlayer::Enabled(p) = opt_p {
            let npc = &mut new_game_state.npcs[*hit_npc_i as usize];

            let pop_x = npc.x_pos;
            let pop_y = npc.y_pos;

            match npc.following_i {
                None => {
                    // add score popup if this was newly found, update score
                    let popup_texts_rb: &mut PopTextRingbuffer =
                        &mut new_game_state.popup_text_ringbuffer;

                    let gained_amount = 1 * 60;

                    popup_texts_rb.add_new_popup(pop_x - 7.0, pop_y, format![" +{}", gained_amount/60].to_string(), PopupIcon::CatHead);

                    // add card
                    let abil_card_type = match npc.sprite_type {
                        spritesheet::PresetSprites::Kitty1 | spritesheet::PresetSprites::Kitty2 | spritesheet::PresetSprites::Kitty3 | spritesheet::PresetSprites::Kitty4 => AbilityCardTypes::Kitty,
                        spritesheet::PresetSprites::Pig => AbilityCardTypes::Piggy,
                        spritesheet::PresetSprites::Lizard => AbilityCardTypes::Lizard,
                        spritesheet::PresetSprites::BirdIsntReal => AbilityCardTypes::Bird,
                        _ => AbilityCardTypes::Kitty,
                    };

                    // spawn some clouds
                    for dir in [(1.0, 0.0), (0.5, 0.86), (-0.5, 0.86), (-1.0, 0.0), (-0.5, -0.86), (0.5, -0.86)] {
                        const CARD_CLOUD_SPEED: f32 = 4.0;

                        let vx = CARD_CLOUD_SPEED * dir.0;
                        let vy = CARD_CLOUD_SPEED * dir.1;
                        new_game_state.clouds = Cloud::try_push_cloud(new_game_state.clouds, npc.x_pos + 2.0, npc.y_pos + 3.0, vx, vy);

                    }

                    let npc_p = new_game_state.camera.cvt_world_to_screen_coords(npc.x_pos, npc.y_pos);
                    p.card_stack.try_push_card(abil_card_type, npc_p.0, npc_p.1);


                    let gained_amount = 1 * 60;
                    new_game_state.countdown_timer_msec += gained_amount;
                    new_game_state.countdown_timer_msec = new_game_state.countdown_timer_msec.min(100 * 60 - 1);
                    new_game_state.score += gained_amount;
                }
                Some(_) => {}
            }

            // p.y_pos -= 2.0;
            npc.following_i = Some(*hit_p_i);
        }
    }

    new_game_state
}

pub struct CollisionResult {
    allowable_displacement: i32,
    collided: bool,
    backed_up: bool,
}

pub fn raycast_axis_aligned(
    horizontal: bool,
    positive: bool,
    abs_start_pt: (i32, i32),
    ray_displacement: i32,
    chunk: &MapChunk,
) -> CollisionResult {
    const DIST_PER_ITER: i32 = 1;

    let mut collision_result = CollisionResult {
        allowable_displacement: ray_displacement,
        collided: true,
        backed_up: false,
    };

    let ray_x_dist_per_iter: i32;
    let ray_y_dist_per_iter: i32;

    // has the opposite sign of the ray displacement, for N iters.

    let mut vertical_ray: i32 = 0;
    let mut horizontal_ray: i32 = 0;

    let start_pt;

    if horizontal {
        start_pt = (abs_start_pt.0, abs_start_pt.1 - 1);
        if positive {
            ray_x_dist_per_iter = DIST_PER_ITER;
            ray_y_dist_per_iter = 0;
        } else {
            ray_x_dist_per_iter = -DIST_PER_ITER;
            ray_y_dist_per_iter = 0;
        }
    } else {
        start_pt = (abs_start_pt.0, abs_start_pt.1);
        if positive {
            ray_x_dist_per_iter = 0;
            ray_y_dist_per_iter = DIST_PER_ITER;
        } else {
            ray_x_dist_per_iter = 0;
            ray_y_dist_per_iter = -DIST_PER_ITER;
        }
    }

    let mut should_clamp_x = false;
    let mut should_clamp_y = false;

    // travel along, see if it's colliding with things

    let mut on_first_iter: bool = true;

    loop {
        // make sure this ray even is in the chunk in the first place
        match chunk.get_tile_abs(
            start_pt.0 + horizontal_ray as i32,
            start_pt.1 + vertical_ray as i32,
        ) {
            Ok(tile) => {
                if tile != 0 {
                    collision_result.collided = true;
                    // if we hit a tile first thing, we need to back up until we DON'T hit anything.
                    if on_first_iter {
                        collision_result.backed_up = true;
                        on_first_iter = false;
                    }
                    // if we're not backing up and we hit something, we're done
                    else if !collision_result.backed_up {
                        // text(format!["hit tile {lowerleft_vertical_ray}"], 10, 50);
                        if horizontal {
                            should_clamp_x = true;
                        } else {
                            should_clamp_y = true;
                        }

                        break;
                    }
                } else {
                    // if we don't hit a tile first thing, we should cast forward until we DO hit something.
                    if on_first_iter {
                        collision_result.backed_up = false;
                        on_first_iter = false;
                    }
                    // if we're backing up and we're not hitting something, we're done backing up
                    else if collision_result.backed_up {
                        if horizontal {
                            should_clamp_x = true;
                        } else {
                            should_clamp_y = true;
                        }

                        break;
                    }
                }
            }
            Err(_) => {
                break collision_result.collided = false;
            }
        }

        if !on_first_iter && !collision_result.backed_up {
            if horizontal {
                if horizontal_ray.abs() > ray_displacement.abs() {
                    collision_result.collided = false;
                    break;
                }
            } else {
                if vertical_ray.abs() > ray_displacement.abs() {
                    collision_result.collided = false;
                    break;
                }
            }
        }

        let dir = match collision_result.backed_up {
            true => -1,
            false => 1,
        };
        vertical_ray += ray_y_dist_per_iter * dir;
        horizontal_ray += ray_x_dist_per_iter * dir;
    }
    if should_clamp_x {
        collision_result.allowable_displacement = vertical_ray;
    } else if should_clamp_y {
        collision_result.allowable_displacement = horizontal_ray;
    }
    collision_result
}

// handle inputs of players and other characters.
pub fn update_pos(map: &GameMap, character: Character, input: u8, clouds: Vec<Cloud>) -> (Character, Vec<Cloud>) {

    let mut new_character = character;
    let mut new_clouds = clouds;

    const BTN_ACCEL: f32 = 0.85;
    const HOP_V: f32 = -5.0;
    const H_DECAY: f32 = 0.92;

    enum HorizontalMovementOutcome {
        ChangedDirection,
        StartedMoving,
        StoppedMoving,
        DoingSameThing,
    }

    fn handle_horizontal_input(the_char: Character, input: u8) -> (Character, HorizontalMovementOutcome) {

        let mut new_character = the_char;

        let ret;
        let previous_direction = new_character.is_facing_right;

        let mut moving_now = false;
        if input & BUTTON_LEFT != 0 {
            new_character.x_vel -= BTN_ACCEL;
            new_character.is_facing_right = false;
            moving_now = true;
        } else if input & BUTTON_RIGHT != 0 {
            new_character.x_vel += BTN_ACCEL;
            new_character.is_facing_right = true;
            moving_now = true;
        } else {
            new_character.x_vel *= H_DECAY;
            new_character.current_sprite_i = 0;
        }

        if moving_now && new_character.state == KittyStates::Sleeping {
            ret = HorizontalMovementOutcome::StartedMoving;
        } else if moving_now && previous_direction != new_character.is_facing_right {
            ret = HorizontalMovementOutcome::ChangedDirection;
            // trace("cd");
        } else if !moving_now && new_character.state != KittyStates::Sleeping {
            ret = HorizontalMovementOutcome::StoppedMoving;
        } else {
            ret = HorizontalMovementOutcome::DoingSameThing;
        }
        
        (new_character, ret)
    }

    fn handle_jumping(the_char: Character, input: u8, clouds: Vec<Cloud>) -> (Character, Vec<Cloud>, bool) {

        let mut new_character = the_char;
        let mut new_clouds = clouds;

        let mut allow_jump = true;
        match new_character.state { 
            KittyStates::JumpingUp(t) => match t {
                0 => {}
                1 => {
                    
                    const CLOUD_VX: f32 = 2.0;
                    const CLOUD_VY: f32 = 1.0;
                    let y = new_character.y_pos + (new_character.sprite.frames[new_character.current_sprite_i as usize].height as f32) * 1.2;
                    let x = new_character.x_pos + (new_character.sprite.frames[new_character.current_sprite_i as usize].width as f32) * 0.5;
                    new_clouds = Cloud::try_push_cloud(new_clouds, x, y, CLOUD_VX, CLOUD_VY);
                    new_clouds = Cloud::try_push_cloud(new_clouds, x, y, -CLOUD_VX, CLOUD_VY);
                }
                2..=10 => {}
                _ => {
                    allow_jump = false;
                }
            },
            _ => {}
        }

        if new_character.can_fly {
            allow_jump = true;
        }

        if allow_jump {
            if input & BUTTON_1 != 0 {
                new_character.state = KittyStates::JumpingUp(0);
                new_character.y_vel = HOP_V;
                return (new_character, new_clouds, true);
            }
        }
        (new_character, new_clouds, false)
    }

    const GRAVITY: f32 = 0.3;
    match new_character.state {
        KittyStates::HuggingWall(_) | KittyStates::OnCeiling(_) => {
            new_character.y_vel = 0.0;
        }
        _ => {
            new_character.y_vel += GRAVITY;
        }
    }

    match new_character.state {
        KittyStates::JumpingUp(t) => {
            (new_character, _) = handle_horizontal_input(new_character, input);
            (new_character, new_clouds, _) = handle_jumping(new_character, input, new_clouds);
            new_character.state = KittyStates::JumpingUp((t + 1).min(255));
        }
        KittyStates::HuggingWall(firstframe) => {
            if firstframe {
                if new_character.is_facing_right {
                    new_character.x_pos += new_character.sprite.frames[3].width as f32
                        - new_character.sprite.frames[4].width as f32;
                }
            }
            new_character.state = KittyStates::HuggingWall(false);
            let ret;
            (new_character, ret) = handle_horizontal_input(new_character, input);
            match ret {
                HorizontalMovementOutcome::ChangedDirection => {
                    new_character.state = KittyStates::JumpingUp(0);
                    // trace("Jump");
                }
                _ => {
                    let didjump;
                    (new_character, new_clouds, didjump) = handle_jumping(new_character, input, new_clouds);
                    if didjump {
                        new_character.is_facing_right = !new_character.is_facing_right;
                        const WALLJUMP_VX: f32 = 3.0;
                        let new_x_vel = match new_character.is_facing_right {
                            true => WALLJUMP_VX,
                            false => -WALLJUMP_VX,
                        };
                        new_character.x_vel = new_x_vel;
                        // #TODO find better spacing fix for walljump on right.
                        if !new_character.is_facing_right {
                            new_character.x_pos -= new_character.sprite.frames[3].width as f32
                                - new_character.sprite.frames[4].width as f32;
                        }
                    }
                }
            }
        }
        KittyStates::Sleeping => {
            let ret;
            (new_character, ret) = handle_horizontal_input(new_character, input);
            match ret {
                HorizontalMovementOutcome::StartedMoving => {
                    new_character.state = KittyStates::Walking(0);
                }
                _ => {}
            }
            (new_character, new_clouds, _) = handle_jumping(new_character, input, new_clouds);
        }
        KittyStates::Walking(t) => {
            let ret;
            (new_character, ret) = handle_horizontal_input(new_character, input);

            match ret {
                HorizontalMovementOutcome::DoingSameThing => {
                    new_character.state = KittyStates::Walking((t + 1) % 255);
                }
                HorizontalMovementOutcome::ChangedDirection => {
                    new_character.state = KittyStates::Walking(0);
                }
                HorizontalMovementOutcome::StoppedMoving => {
                    new_character.state = KittyStates::Sleeping;
                }
                _ => {
                    // character.state = KittyStates::Sleeping;
                }
            }
            (new_character, new_clouds, _) = handle_jumping(new_character, input, new_clouds);
        },
        KittyStates::OnCeiling(t) => {
            let ret;
            (new_character, ret) = handle_horizontal_input(new_character, input);

            match ret {
                HorizontalMovementOutcome::DoingSameThing => {
                    new_character.state = KittyStates::OnCeiling((t + 1) % 255);
                }
                HorizontalMovementOutcome::ChangedDirection => {
                    new_character.state = KittyStates::OnCeiling(0);
                }
                HorizontalMovementOutcome::StoppedMoving => {
                    new_character.state = KittyStates::Sleeping;
                }
                _ => {
                    // character.state = KittyStates::Sleeping;
                }
            }
            
            if t > 30 {
                (new_character, new_clouds, _) = handle_jumping(new_character, input, new_clouds);
            }
            
        },
    }

    fn get_sprite_i_from_anim_state(state: &KittyStates, discrete_y_vel: i32) -> i32 {
        match state {
            KittyStates::HuggingWall(_) => 4,
            KittyStates::JumpingUp(t) => {
                match t {
                    0 => 0,
                    _ => {
                        if discrete_y_vel < -1 {
                            3
                        } else if discrete_y_vel < 5 {
                            2
                        } else {
                            5
                        }
                    }
                }

                // match t {
                //     0..=21 => 3,
                //     22..=25 => 2,
                //     _ => 5
                // }
            }
            KittyStates::Sleeping => 0,
            KittyStates::Walking(t) | KittyStates::OnCeiling(t) => match (t / 6) % 2 {
                0 => 1,
                _ => 2,
            },
        }
    }

    new_character.x_vel = num::clamp(new_character.x_vel, -new_character.x_vel_cap, new_character.x_vel_cap);
    new_character.y_vel = num::clamp(new_character.y_vel, -new_character.y_vel_cap, new_character.y_vel_cap);

    // now, we need to check if moving in the current direction would collide with anything.
    // Since before moving we can assume we are in a valid location, as long as this collision
    // logic places us in another valid location, we'll be okay.

    let mut discretized_y_displacement_this_frame = new_character.y_vel as i32;
    let mut discretized_x_displacement_this_frame = new_character.x_vel as i32;

    // hotfix: if our y displacement is exactly zero, set it to 1, just so we
    // can properly check if we're colliding with the ground.
    match new_character.state {
        KittyStates::HuggingWall(_) => {}
        KittyStates::OnCeiling(_) => {
            if discretized_y_displacement_this_frame == 0 {
                discretized_y_displacement_this_frame = -1;
            }
        }
        _ => {
            if discretized_y_displacement_this_frame == 0 {
                discretized_y_displacement_this_frame = 1;
            }
        }
    }

    let mut touching_some_ground: bool = false;


    // trace("will check--------------------");
    // look at each chunk, and see if the player is inside it
    new_character.current_sprite_i =
        get_sprite_i_from_anim_state(&new_character.state, discretized_y_displacement_this_frame);
    let char_bound = get_bound_of_character(&new_character);
    let mut inside_at_least_one_chunk = false;
    for chunk in map.chunks.iter() {
        // trace("checking chn");

        // if the sprite is inside this chunk, we now need to check to see if moving along our velocity
        if check_absolue_bound_partially_inside_tile_aligned_bound(&char_bound, &chunk.bound) {
            inside_at_least_one_chunk = true;
            // text(format!["Player in ch {i}"], 10, 10);

            // VERTICAL COLLISION
            // take the y velocity, and start at the edge of the player's bounding box, and project the line outward, to check for collisions.

            // text(format!["moving up"], 10, 20);
            // CHECK TOP LEFT CORNER
            // create the decreasing vertical vector, and incrementally travel until it is touching something

            // let lowerleft_checker_location = (char_bound.x, char_bound.y);
            // let lowerright_checker_location = (char_bound.x + char_bound.width as i32 - 1, char_bound.y);
            // let upperleft_checker_location = (char_bound.x, char_bound.y + char_bound.height as i32 - 1);
            // let upperright_checker_location = (char_bound.x + char_bound.width as i32 - 1, char_bound.y + char_bound.height as i32 - 1);

            // fn check_collision_group_on_same_dir(
            //     points: &Vec<(i32, i32)>,
            //     character: &mut Character,
            //     displacement_to_set: &mut i32,
            //     horizontal: bool,
            //     positive: bool,
            // ) {

            // }

            let h_col_res_lower;
            let h_col_res_upper;
            let v_col_res_left;
            let v_col_res_right;

            let upper_y = char_bound.y + char_bound.height as i32 - 2;
            let lower_y = char_bound.y + 1;
            let left_x: i32 = char_bound.x + 1;
            let right_x: i32 = char_bound.x + char_bound.width as i32 - 2;

            let vert_y;
            let positive_y;

            // VERTICAL RAYCAST
            if discretized_y_displacement_this_frame > 0 {
                // GOING DOWNWARD
                positive_y = true;
                vert_y = upper_y;
            } else {
                positive_y = false;
                vert_y = lower_y;
            }
            v_col_res_left = raycast_axis_aligned(
                false,
                positive_y,
                (left_x, vert_y),
                discretized_y_displacement_this_frame,
                chunk,
            );
            v_col_res_right = raycast_axis_aligned(
                false,
                positive_y,
                (right_x, vert_y),
                discretized_y_displacement_this_frame,
                chunk,
            );

            let horizontal_x;
            let positive_x;

            // HORIZONTAL RAYCAST
            if discretized_x_displacement_this_frame > 0 {
                // GOING DOWNWARD
                positive_x = true;
                horizontal_x = right_x;
            } else {
                positive_x = false;
                horizontal_x = left_x;
            }
            h_col_res_lower = raycast_axis_aligned(
                true,
                positive_x,
                (horizontal_x, lower_y),
                discretized_x_displacement_this_frame,
                chunk,
            );
            h_col_res_upper = raycast_axis_aligned(
                true,
                positive_x,
                (horizontal_x, upper_y),
                discretized_x_displacement_this_frame,
                chunk,
            );

            if v_col_res_left.collided || v_col_res_right.collided {
                touching_some_ground = true;
                // if we collided against the top, automatically hang
                if positive_y == false {
                    new_character.state = match new_character.state {
                        KittyStates::OnCeiling(t) => KittyStates::OnCeiling(t+1),
                        KittyStates::HuggingWall(t) => KittyStates::HuggingWall(t),
                        _ => KittyStates::OnCeiling(0),
                    };
                }

                new_character.y_vel = 0.0;
                if v_col_res_left.backed_up {
                    discretized_y_displacement_this_frame =
                        v_col_res_left.allowable_displacement;
                } else if v_col_res_right.backed_up {
                    discretized_y_displacement_this_frame =
                        v_col_res_right.allowable_displacement;
                } else {
                    discretized_y_displacement_this_frame =
                        v_col_res_left.allowable_displacement.signum()
                            * v_col_res_left
                                .allowable_displacement
                                .abs_diff(v_col_res_left.allowable_displacement)
                                as i32;
                }

                // if positive_y {
                //     // && !h_col_res_lower.collided && !h_col_res_upper.collided {
                //     touching_some_ground = true;
                // }
            }

            if h_col_res_lower.collided || h_col_res_upper.collided {
                if !h_col_res_lower.collided && positive_y {
                    // if we are touching floor (pos y), and the bottom horizontal hit but
                    // not the top, allow us to hop up the ledge.

                    discretized_y_displacement_this_frame -= TILE_HEIGHT_PX as i32;
                } else {
                    // if the above special case isn't true, we hit a wall
                    new_character.x_vel = 0.0;

                    // if in free fall (after beginning of jump), allow hugging wall
                    match new_character.state {
                        KittyStates::JumpingUp(t) => match t {
                            0..=15 => {}
                            _ => {
                                new_character.state = KittyStates::HuggingWall(true);
                            }
                        },
                        KittyStates::OnCeiling(_) => {
                            new_character.state = KittyStates::HuggingWall(true);
                        }
                        _ => {}
                    }
                }

                if h_col_res_lower.backed_up {
                    discretized_x_displacement_this_frame =
                        h_col_res_lower.allowable_displacement;
                } else if h_col_res_upper.backed_up {
                    discretized_x_displacement_this_frame =
                        h_col_res_upper.allowable_displacement;
                } else {
                    discretized_x_displacement_this_frame =
                        h_col_res_lower.allowable_displacement.signum()
                            * h_col_res_upper
                                .allowable_displacement
                                .abs_diff(h_col_res_lower.allowable_displacement)
                                as i32;
                }
            }
            // // upward collision
            // if character.y_vel < 0.0 {
            //     let collision_res = raycast_axis_aligned(false, false, lowerleft_checker_location, discretized_y_displacement_this_frame, chunk);
            //     discretized_y_displacement_this_frame = collision_res.allowable_displacement;
            //     if collision_res.collided {
            //         character.y_vel = 0.0;
            //     }
            //     if !collision_res.backed_up {
            //         let second_collision_res = raycast_axis_aligned(false, false, lowerright_checker_location, discretized_y_displacement_this_frame, chunk);
            //         discretized_y_displacement_this_frame = second_collision_res.allowable_displacement;
            //         if second_collision_res.collided {
            //             character.y_vel = 0.0;
            //         }
            //     }
            // }

            // // downward collision
            // else {
            //     let collision_res = raycast_axis_aligned(false, true, upperleft_checker_location, discretized_y_displacement_this_frame, chunk);
            //     discretized_y_displacement_this_frame = collision_res.allowable_displacement;
            //     if collision_res.collided {
            //         character.y_vel = 0.0;
            //         touching_some_ground = true;

            //     } else {

            //     }
            //     if !collision_res.backed_up {
            //         let second_collision_res = raycast_axis_aligned(false, true, upperright_checker_location, discretized_y_displacement_this_frame, chunk);
            //         discretized_y_displacement_this_frame = second_collision_res.allowable_displacement;
            //         if second_collision_res.collided {
            //             touching_some_ground = true;
            //             character.y_vel = 0.0;
            //         }
            //     }
            // }

            // // left collision
            // if character.x_vel < 0.0 {
            //     let collision_res = raycast_axis_aligned(true, false, lowerleft_checker_location, discretized_x_displacement_this_frame, chunk);
            //     discretized_x_displacement_this_frame = collision_res.allowable_displacement;
            //     if collision_res.collided {
            //         // if in free fall (after beginning of jump), allow hugging wall
            //         match character.state {
            //             KittyStates::JumpingUp(t) => {
            //                 match t {
            //                     0..=15 => {

            //                     },
            //                     _ => {
            //                         character.state = KittyStates::HuggingWall(true);
            //                     }
            //                 }
            //             }
            //             _ => {}
            //         }
            //         character.x_vel = 0.0;
            //     }
            //     if !collision_res.backed_up {
            //         let second_collision_res = raycast_axis_aligned(true, false, upperleft_checker_location, discretized_x_displacement_this_frame, chunk);
            //         discretized_x_displacement_this_frame = second_collision_res.allowable_displacement;
            //         if second_collision_res.collided {
            //             character.x_vel = 0.0;
            //         }
            //     }
            // }

            // // right collision
            // else {

            //     let collision_res = raycast_axis_aligned(true, true, lowerright_checker_location, discretized_x_displacement_this_frame, chunk);
            //     discretized_x_displacement_this_frame = collision_res.allowable_displacement;
            //     if collision_res.collided {
            //         // if in free fall (after beginning of jump), allow hugging wall
            //         match character.state {
            //             KittyStates::JumpingUp(t) => {
            //                 match t {
            //                     0..=15 => {

            //                     },
            //                     _ => {
            //                         character.state = KittyStates::HuggingWall(true);
            //                     }
            //                 }
            //             }
            //             _ => {}
            //         }

            //         character.x_vel = 0.0;
            //     }
            //     if !collision_res.backed_up {
            //         let second_collision_res = raycast_axis_aligned(true, true, upperright_checker_location, discretized_x_displacement_this_frame, chunk);
            //         discretized_x_displacement_this_frame = second_collision_res.allowable_displacement;
            //         if second_collision_res.collided {
            //             character.x_vel = 0.0;
            //         }
            //     }
            // }
        }
    }

    // if anyone makes it out of bounds, drop them in the center of the map
    // (or if they use the lizard warp)
    if !inside_at_least_one_chunk {
        new_character.x_pos = 10.0;
        new_character.y_pos = 10.0;
    }
    

    if !touching_some_ground {
        // if we were walking and we fall off, change state
        match new_character.state {
            KittyStates::Walking(_) | KittyStates::Sleeping => {
                new_character.state = KittyStates::JumpingUp(30);
            }
            KittyStates::OnCeiling(_) => {
                new_character.state = KittyStates::JumpingUp(0);
            }
            _ => {}
        }
    } else {

 

        // if we hit the floor, stop jumping
        match new_character.state {
            KittyStates::JumpingUp(t) => match t {
                0..=15 => {
                    // Cloud::try_push_cloud(clouds, 0.0, 0.0, 5.0, 0.0);
                }
                _ => {
                    new_character.state = KittyStates::Walking(0);
                }
            },
            _ => {}
        }
    }

    // handle warping. If down is held, warp.
    if input & BUTTON_DOWN != 0 {
        match new_character.warp_ability {
            WarpAbility::CannotWarp => {},
            WarpAbility::CanWarp(warp_state) => {
                match warp_state {
                    WarpState::Charging(t) => {
                        if t >= 25 {
                            new_character.warp_ability = WarpAbility::CanWarp(WarpState::Ready);
                        } else {
                            new_character.warp_ability = WarpAbility::CanWarp(WarpState::Charging(t + 1));
                        }
                    },
                    WarpState::Ready => {
                        new_character.x_pos = 10.0;
                        new_character.y_pos = 10.0;
                        new_character.warp_ability = WarpAbility::CanWarp(WarpState::Charging(0));
                    }
                }
            }
        }
    } else {
        match new_character.warp_ability {
            WarpAbility::CanWarp(_) => {
                new_character.warp_ability = WarpAbility::CanWarp(WarpState::Charging(0));
            },
            _ => {}
        }
    }

    // character.current_sprite_i = get_sprite_i_from_anim_state(&character.state, discretized_y_displacement_this_frame);

    new_character.x_pos += discretized_x_displacement_this_frame as f32;
    new_character.y_pos += discretized_y_displacement_this_frame as f32;

    new_character.x_pos = num::clamp(new_character.x_pos, X_LEFT_BOUND as f32, X_RIGHT_BOUND as f32);
    new_character.y_pos = num::clamp(new_character.y_pos, Y_LOWER_BOUND as f32, Y_UPPER_BOUND as f32);

    new_character.count += 1;


    (new_character, new_clouds)
}
