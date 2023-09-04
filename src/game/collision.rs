use crate::spritesheet;

use super::{mapchunk::{TileAlignedBoundingBox, MapChunk}, game_constants::{TILE_WIDTH_PX, TILE_HEIGHT_PX, X_LEFT_BOUND, X_RIGHT_BOUND, Y_LOWER_BOUND, Y_UPPER_BOUND}, game_map::GameMap, entities::{MovingEntity, Character, OptionallyEnabledPlayer, KittyStates}};

use crate::wasm4::*;

pub fn check_absolute_point_inside_tile_aligned_bound(x: i32, y: i32, bound: &TileAlignedBoundingBox) -> bool {
    let bound_absolute_left_x: i32 = bound.x * TILE_WIDTH_PX as i32;
    let bound_absolute_right_x: i32 = bound_absolute_left_x + bound.width as i32 * TILE_WIDTH_PX as i32;
    let bound_absolute_lower_y: i32 = bound.y * TILE_HEIGHT_PX as i32;
    let bound_absolute_upper_y: i32 = bound_absolute_lower_y + bound.height as i32 * TILE_HEIGHT_PX as i32;

    if x > bound_absolute_left_x {
        if x < bound_absolute_right_x {
            if y > bound_absolute_lower_y {
                if y < bound_absolute_upper_y {
                    return true
                }
            }
        }
    }
    false
}

pub struct AbsoluteBoundingBox {
    x: i32,
    y: i32,
    width: usize,
    height: usize
}
pub fn check_absolue_bound_partially_inside_tile_aligned_bound(absolute_bound: &AbsoluteBoundingBox, tile_aligned_bound: &TileAlignedBoundingBox) -> bool {
    let lowerleft = (absolute_bound.x, absolute_bound.y);
    let lowerright = (absolute_bound.x + absolute_bound.width as i32, absolute_bound.y);
    let upperleft = (absolute_bound.x, absolute_bound.y + absolute_bound.height as i32);
    let upperright = (absolute_bound.x + absolute_bound.width as i32, absolute_bound.y + absolute_bound.height as i32);
    if !check_absolute_point_inside_tile_aligned_bound(lowerleft.0, lowerleft.1, tile_aligned_bound) {
        if !check_absolute_point_inside_tile_aligned_bound(lowerright.0, lowerright.1, tile_aligned_bound) {
            if !check_absolute_point_inside_tile_aligned_bound(upperleft.0, upperleft.1, tile_aligned_bound) {
                if !check_absolute_point_inside_tile_aligned_bound(upperright.0, upperright.1, tile_aligned_bound) {
                    return false
                }
            }
        } 
    }
    true
}

pub fn raycast_axis_aligned(horizontal: bool, dist_per_iter: f32, abs_start_pt: (i32, i32), ray_displacement: f32, chunk: &MapChunk) -> (f32, bool) {

    let positive: bool = ray_displacement > 0.0;

    let mut required_backing_up: bool = false;

    let ray_x_dist_per_iter;
    let ray_y_dist_per_iter;
    
    // has the opposite sign of the ray displacement, for N iters.

    let mut vertical_ray: f32 = 0.0;
    let mut horizontal_ray: f32 = 0.0;

    let mut ret_val = ray_displacement;

    let start_pt;
    
    
    if horizontal {
        start_pt = (abs_start_pt.0, abs_start_pt.1 - 1);
        if positive {
            ray_x_dist_per_iter = dist_per_iter;
            ray_y_dist_per_iter = 0.0;
        } else {
            ray_x_dist_per_iter = -dist_per_iter;
            ray_y_dist_per_iter = 0.0;
        }
    }
    
    else {
        start_pt = (abs_start_pt.0, abs_start_pt.1);
        if positive {
            ray_x_dist_per_iter = 0.0;
            ray_y_dist_per_iter = dist_per_iter;
        } else {
            ray_x_dist_per_iter = 0.0;
            ray_y_dist_per_iter = -dist_per_iter;
        }
    }

    

    let mut should_clamp_x = false;
    let mut should_clamp_y = false;

    

    // travel along, see if it's colliding with things

    let mut on_first_iter: bool = true;

    loop {


        

        
        // make sure this ray even is in the chunk in the first place
        match chunk.get_tile_abs(start_pt.0 + horizontal_ray as i32, start_pt.1 + vertical_ray as i32) {
            Ok(tile) => {
                if tile != 0 {
                    // if we hit a tile first thing, we need to back up until we DON'T hit anything.
                    if on_first_iter {
                        required_backing_up = true;
                        on_first_iter = false;
                    }
                    // if we're not backing up and we hit something, we're done
                    else if !required_backing_up {
                        
                        // text(format!["hit tile {lowerleft_vertical_ray}"], 10, 50);
                        if horizontal {
                            should_clamp_x = true;
                        } else {
                            should_clamp_y = true;
                        }
                        
                        break
                    }

                    
                } else {
                    // if we don't hit a tile first thing, we should cast forward until we DO hit something.
                    if on_first_iter {
                        required_backing_up = false;
                        on_first_iter = false;
                    } else if required_backing_up {
                        if horizontal {
                            should_clamp_x = true;
                        } else {
                            should_clamp_y = true;
                        }
                        
                        break
                    }
                }
            }
            Err(_) => {
                break
            }
        }

        if !on_first_iter && !required_backing_up {
            if horizontal {
                if horizontal_ray.abs() > ray_displacement.abs() {
                    break
                }
            } else {
                if vertical_ray.abs() > ray_displacement.abs() {
                    break
                }
            }
        }
        
        let dir = match required_backing_up {
            true => -1.0,
            false => 1.0
        };
        vertical_ray += ray_y_dist_per_iter * dir;
        horizontal_ray += ray_x_dist_per_iter * dir;

    }
    if should_clamp_x {
        ret_val = vertical_ray;
    } else if should_clamp_y {
        ret_val = horizontal_ray;
    }

    (ret_val, required_backing_up)
}

pub fn update_pos(map: &GameMap, moving_entity: MovingEntity, input: u8) {
    
    let character: &mut Character;

    match moving_entity {
        MovingEntity::OptionalPlayer(optionally_enabled_player) => {
            match optionally_enabled_player {
                OptionallyEnabledPlayer::Enabled(ch) => {
                    character = ch;
                }
                OptionallyEnabledPlayer::Disabled => {
                    if input != 0 {
                        *optionally_enabled_player = OptionallyEnabledPlayer::Enabled(Character::new(spritesheet::PresetSprites::MainCat));
                        match optionally_enabled_player {
                            OptionallyEnabledPlayer::Enabled(ch) => {
                                character = ch;
                            }
                            _ => {
                                return
                            }
                        }
                    }
                    else {
                        return
                    }
                }
            }
        }
        MovingEntity::NPC(npc) => {
            character = npc;
        }
    }
    
    let btn_accel = 0.6;
    let hop_v: f32 = -4.0;
    let h_decay = 0.8;
    if input & BUTTON_LEFT != 0 {
        character.x_vel -= btn_accel;
        character.facing_right = false;
        character.state = KittyStates::Moving1;
        character.current_sprite_i = 1;
    } else if input & BUTTON_RIGHT != 0 {
        character.x_vel += btn_accel;
        character.facing_right = true;
        character.state = KittyStates::Moving2;
        character.current_sprite_i = 2;
    } else {
        character.x_vel *= h_decay;
        character.state = KittyStates::Idle;
        character.current_sprite_i = 0;
    }
    if input & BUTTON_1 != 0 {
        character.y_vel = hop_v;
        character.state = KittyStates::Jump;
        character.current_sprite_i = 3;
    } else if input & BUTTON_DOWN != 0 {
    }


    let gravity = 0.3;
    character.y_vel += gravity;
    character.x_vel = num::clamp(character.x_vel, -character.x_vel_cap, character.x_vel_cap);
    character.y_vel = num::clamp(character.y_vel, -character.y_vel_cap, character.y_vel_cap);


    // now, we need to check if moving in the current direction would collide with anything.
    // Since before moving we can assume we are in a valid location, as long as this collision
    // logic places us in another valid location, we'll be okay.

   
    // trace("will check--------------------");
    // look at each chunk, and see if the player is inside it
    for chunk in map.chunks.iter() {
        
        // trace("checking chn");

        let char_positioning = character.sprite.frames[character.current_sprite_i as usize].positioning;
        let char_bound = AbsoluteBoundingBox {
            x: character.x_pos as i32,
            y: character.y_pos as i32,
            width: char_positioning.width,
            height: char_positioning.height,
        };


        

        // if the sprite is inside this chunk, we now need to check to see if moving along our velocity
        if check_absolue_bound_partially_inside_tile_aligned_bound(&char_bound, &chunk.bound) {
            // text(format!["Player in ch {i}"], 10, 10);

            // VERTICAL COLLISION
            // take the y velocity, and start at the edge of the player's bounding box, and project the line outward, to check for collisions.


            // text(format!["moving up"], 10, 20);
            // CHECK TOP LEFT CORNER
            // create the decreasing vertical vector, and incrementally travel until it is touching something


            

            const RAYCAST_DIST_PER_ITER: f32 = 1.0;

            let lowerleft_checker_location = (char_bound.x, char_bound.y);
            let lowerright_checker_location = (char_bound.x + char_bound.width as i32, char_bound.y);
            let upperleft_checker_location = (char_bound.x, char_bound.y + char_bound.height as i32);
            let upperright_checker_location = (char_bound.x + char_bound.width as i32, char_bound.y + char_bound.height as i32);
            if character.y_vel < 0.0 {
                let backed_up;
                (character.y_vel, backed_up) = raycast_axis_aligned(false, RAYCAST_DIST_PER_ITER, lowerleft_checker_location, character.y_vel, chunk);
                if !backed_up {
                    (character.y_vel, _) = raycast_axis_aligned(false, RAYCAST_DIST_PER_ITER, lowerright_checker_location, character.y_vel, chunk);
                }
            }
            else if character.y_vel >= 0.0 {
                let backed_up;
                (character.y_vel, backed_up) = raycast_axis_aligned(false, RAYCAST_DIST_PER_ITER, upperleft_checker_location, character.y_vel, chunk);
                if !backed_up { 
                    (character.y_vel, _) = raycast_axis_aligned(false, RAYCAST_DIST_PER_ITER, upperright_checker_location, character.y_vel, chunk);
                }
            }

            if character.x_vel < 0.0 {
                let backed_up;
                (character.x_vel, backed_up) = raycast_axis_aligned(true, RAYCAST_DIST_PER_ITER, lowerleft_checker_location, character.x_vel, chunk);
                if !backed_up {
                    (character.x_vel, _) = raycast_axis_aligned(true, RAYCAST_DIST_PER_ITER, upperleft_checker_location, character.x_vel, chunk);
                }
            }
            else if character.x_vel >= 0.0 {
                let backed_up;
                (character.x_vel, backed_up) = raycast_axis_aligned(true, RAYCAST_DIST_PER_ITER, lowerright_checker_location, character.x_vel, chunk);
                if !backed_up {
                    (character.x_vel, _) = raycast_axis_aligned(true, RAYCAST_DIST_PER_ITER, upperright_checker_location, character.x_vel, chunk);
                }
            }   
            
            
        }
        
    }

    character.x_pos += character.x_vel;
    character.y_pos += character.y_vel;

    character.x_pos = num::clamp(character.x_pos, X_LEFT_BOUND as f32, X_RIGHT_BOUND as f32);
    character.y_pos = num::clamp(character.y_pos, Y_LOWER_BOUND as f32, Y_UPPER_BOUND as f32);

    character.count += 1;

}
