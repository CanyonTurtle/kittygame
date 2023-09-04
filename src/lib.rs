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
use game::{game_constants::{TILE_WIDTH_PX, TILE_HEIGHT_PX, MAP_CHUNK_MIN_SIDE_LEN, MAP_CHUNK_MAX_SIDE_LEN, MAP_CHUNK_MAX_N_TILES, X_LEFT_BOUND, X_RIGHT_BOUND, Y_LOWER_BOUND, Y_UPPER_BOUND, GameMode}, game_state::GameState, mapchunk::{MapChunk, TileAlignedBoundingBox}, entities::{MovingEntity, Character, KittyStates}, camera::Camera};
use num;
mod game;
use game::game_map::GameMap;
use std::cell::RefCell;
use wasm4::*;

use crate::game::{entities::OptionallyEnabledPlayer, rng::Rng};



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











fn create_map() -> GameMap {
    let chunks: Vec<MapChunk> = Vec::new();

    let map = GameMap { 
        chunks: chunks,
        num_tiles: 0,
    };


    map
}

fn regenerate_map(game_state: &mut GameState) {

    let map = &mut game_state.map;
    map.num_tiles = 0;
    map.chunks.clear();
    let rng = &mut game_state.rng;

    for optional_player in game_state.players.borrow_mut().iter_mut() {
        match optional_player {
            OptionallyEnabledPlayer::Enabled(p) => {
                p.x_pos = 10.0;
                p.y_pos = 10.0;
            }
            OptionallyEnabledPlayer::Disabled => {

            }
        }
    }

    let npcs = &mut game_state.npcs.borrow_mut();



    let mut current_chunk_locations: Vec<TileAlignedBoundingBox> = vec![TileAlignedBoundingBox::init(0, 0, 32, 32)];

    // place the chunks randomly.
    'generate_chunks: loop {
        // attempt to place a new chunk
        // if in viable location, place this chunk
        loop {
            // trace("placing chunk");
            // choose a new viable chunk size

            let mut chunk_wid: usize;
            let mut chunk_hei: usize;
            loop {
                chunk_wid = MAP_CHUNK_MIN_SIDE_LEN + (rng.borrow_mut().next() as usize % (MAP_CHUNK_MAX_SIDE_LEN - MAP_CHUNK_MIN_SIDE_LEN));
                chunk_hei = MAP_CHUNK_MIN_SIDE_LEN + (rng.borrow_mut().next() as usize % (MAP_CHUNK_MAX_SIDE_LEN - MAP_CHUNK_MIN_SIDE_LEN));
                // trace(format!("{chunk_wid} {chunk_hei}"));
                if chunk_hei * chunk_wid <= MAP_CHUNK_MAX_N_TILES {
                    if map.try_fit_chunk_into(chunk_wid, chunk_hei) {
                        break
                    } else {
                        // n_retries += 1;
                        // if n_retries > 1000 {
                        //     panic!("out of memory for the tilemap");
                        // }
                        break 'generate_chunks;
                    }
                }
            }

            let rng = &mut rng.borrow_mut();

            let r_offs_1: i32 = rng.next() as i32 % MAP_CHUNK_MIN_SIDE_LEN as i32 - (MAP_CHUNK_MIN_SIDE_LEN as f32 / 2.0) as i32;

            let random_chunk_from_list_i = (rng.next() % current_chunk_locations.len() as u64) as usize;
            let vertical_stack = rng.next() % 2 == 1;
            let positive_stack = rng.next() % 2 == 1;
            let rand_bound = &current_chunk_locations[random_chunk_from_list_i];
            let new_chunk_location: TileAlignedBoundingBox;

            // const VARIATION_FROM_CHUNK_DIMS: usize = 0;

            // let n_chunk_wid = num::clamp(MAP_CHUNK_MIN_N_COLS + (rng.next() as usize % (MAP_CHUNK_MAX_N_COLS - MAP_CHUNK_MIN_N_COLS)), MAP_CHUNK_MIN_N_COLS, rand_chunk.width + VARIATION_FROM_CHUNK_DIMS);
            // let n_chunk_hei = num::clamp(MAP_CHUNK_MIN_N_ROWS + (rng.next() as usize % (MAP_CHUNK_MAX_N_ROWS - MAP_CHUNK_MIN_N_ROWS)), MAP_CHUNK_MIN_N_ROWS, rand_chunk.height + VARIATION_FROM_CHUNK_DIMS);

            if vertical_stack {
                if positive_stack {
                    new_chunk_location = TileAlignedBoundingBox::init(rand_bound.x + r_offs_1, rand_bound.y + rand_bound.height as i32, chunk_wid, chunk_hei);
                } else {
                    new_chunk_location = TileAlignedBoundingBox::init(rand_bound.x + r_offs_1, rand_bound.y - chunk_hei as i32, chunk_wid, chunk_hei);
                }
            } else {
                if positive_stack {
                    new_chunk_location = TileAlignedBoundingBox::init(rand_bound.x + rand_bound.width as i32, rand_bound.y + r_offs_1, chunk_wid, chunk_hei);
                } else {
                    new_chunk_location = TileAlignedBoundingBox::init(rand_bound.x - chunk_wid as i32, rand_bound.y + r_offs_1, chunk_wid, chunk_hei);
                }
            }
            let mut is_viable_spot = true;
            for other_bound in &current_chunk_locations {
                // if it collides with existing chunk, disallow
                if new_chunk_location.y + new_chunk_location.height as i32 > other_bound.y {
                    if new_chunk_location.y < other_bound.y + other_bound.height as i32 {
                        if new_chunk_location.x + new_chunk_location.width as i32 > other_bound.x {
                            if new_chunk_location.x < other_bound.x + other_bound.width as i32 {
                                is_viable_spot = false;
                            }
                        }
                    }
                }
            }

            if is_viable_spot {
                // trace(format!("pushing chunk {new_chunk_location:?}"));
                current_chunk_locations.push(new_chunk_location);
                break;
            }
        }
    }

    for (i, current_chunk_location) in current_chunk_locations.into_iter().enumerate() {
        let mut chunk = MapChunk::init();
        
        chunk.bound = current_chunk_location;

        // spawn an npc here if needed
        if i < npcs.len() {
            npcs[i].x_pos = chunk.bound.x as f32 * TILE_WIDTH_PX as f32 + 10.0;
            npcs[i].y_pos = chunk.bound.y as f32 * TILE_HEIGHT_PX as f32 + 10.0;
        }

        // chunk.tiles.clear();
        // for _ in 0..chunk.bound.height {
        //     let mut chunk_row: Vec<u8> = Vec::new();
        //     for _ in 0..chunk.bound.width {
        //         chunk_row.push(0);
        //     }
        //     chunk.tiles.push(chunk_row);
        // }
        chunk.reset_chunk();
        
        

        // for col in 0..MAP_CHUNK_N_COLS {
        //     tiles[MAP_CHUNK_N_ROWS - GROUND_TILE_OFFSET][col] = 1;
        // }

        const CHUNK_BORDER_MATERIAL: u8 = 6;

        // const POSSIBLE_BUILDING_MATERIALS: [u8; 1] = [6];
        const CORRUPT_MATERIALS: [u8; 7] = [7, 8, 9, 10, 11, 12, 13];
        const CORRUPT_CHANCE: f32 = 0.2;
        
        fn get_material(normal: u8, corrupt: u8, chance: f32, rng: &mut Rng) -> u8 {
            if (rng.next() as u8 % 255) as f32 > 255.0 * chance {
                return normal;
            }
            corrupt
        }

        let rng_ref = &mut rng.borrow_mut();

        for row in 0..chunk.bound.height as usize {
            let corrupt_material: u8 = CORRUPT_MATERIALS[rng_ref.next() as usize % CORRUPT_MATERIALS.len()];
            let material = get_material(CHUNK_BORDER_MATERIAL, corrupt_material, CORRUPT_CHANCE, rng_ref);
            chunk.set_tile(0, row, material);
            chunk.set_tile(chunk.bound.width as usize - 1, row, material);
        }
        for col in 0..chunk.bound.width as usize {
            let corrupt_material: u8 = CORRUPT_MATERIALS[rng_ref.next() as usize % CORRUPT_MATERIALS.len()];
            let material = get_material(CHUNK_BORDER_MATERIAL, corrupt_material, CORRUPT_CHANCE, rng_ref);
            chunk.set_tile(col, 0, material);
            chunk.set_tile(col, chunk.bound.height as usize - 1, material);
        }
        

        // fn spawn_rectangular_structures(chunk: &mut MapChunk, rng: &mut Rng) {
        //     // trace(format!("spawning structure with {chunk_width} {chunk_height}"));
        //     let mut inside_start_xs: [u8; N_BUILDINGS_PER_CHUNK] = [0; N_BUILDINGS_PER_CHUNK];
        //     let mut inside_start_ys: [u8; N_BUILDINGS_PER_CHUNK] = [0; N_BUILDINGS_PER_CHUNK];
        //     let mut inside_end_xs: [u8; N_BUILDINGS_PER_CHUNK] = [0; N_BUILDINGS_PER_CHUNK];
        //     let mut inside_end_ys: [u8; N_BUILDINGS_PER_CHUNK] = [0; N_BUILDINGS_PER_CHUNK];

        //     let are_doors_on_right: bool = (rng.next() as u8) < 127;

        //     for i in 0..N_BUILDINGS_PER_CHUNK {

        //         // trace("starting spawn structure");


        //         let building_min_width: usize = num::clamp(BUILDING_SUGGESTED_MIN_WIDTH, 1, chunk.bound.width as usize);
        //         let building_max_width: usize = num::clamp(BUILDING_SUGGESTED_MAX_WIDTH, building_min_width, chunk.bound.width as usize);

        //         let building_min_height: usize = num::clamp(BUILDING_SUGGESTED_MIN_HEIGHT, 1, chunk.bound.height as usize);
        //         let building_max_height: usize = num::clamp(BUILDING_SUGGESTED_MAX_HEIGHT, building_min_height, chunk.bound.height as usize);
 
        //         // trace("established mins and maxes");
        //         const POSSIBLE_BUILDING_MATERIALS: [u8; 1] = [6];
        //         const CORRUPT_MATERIALS: [u8; 7] = [7, 8, 9, 10, 11, 12, 13];
        //         const CORRUPT_CHANCE: f32 = 0.2;
                
        //         fn get_material(normal: u8, corrupt: u8, chance: f32, rng: &mut Rng) -> u8 {
        //             if (rng.next() as u8 % 255) as f32 > 255.0 * chance {
        //                 return normal;
        //             }
        //             corrupt
        //         }
        
        //         // spawn structure
        //         let building_width: usize = building_min_width + (rng.next() as usize % (core::cmp::max(building_max_width - building_min_width, 1)));
        //         let building_height: usize = building_min_height + (rng.next() as usize % (core::cmp::max(building_max_height - building_min_height, 1)));
        
        //         // trace("got building dims");

        //         // trace(format!("Building width: {building_width}, chunk_width: {chunk_width} "));
        //         // trace(format!("Building height: {building_height}, chunk_height: {chunk_height} "));


        //         let building_chunk_loc_x: usize = (rng.next() as u64 % (core::cmp::max(chunk.bound.width as i64 - building_width as i64, 1)) as u64) as usize;
        //         let building_chunk_loc_y: usize = (rng.next() as u64 % (core::cmp::max(chunk.bound.height as i64 - building_height as i64, 1)) as u64) as usize;
        
        //         // trace("got modded loc");
        //         inside_start_xs[i] = building_chunk_loc_x as u8 + 1;
        //         inside_start_ys[i] = building_chunk_loc_y as u8 + 1;
        //         inside_end_xs[i] = building_chunk_loc_x as u8 + building_width as u8 - 1;
        //         inside_end_ys[i] = building_chunk_loc_y as u8 + building_height as u8 - 1;


        //         let building_material: u8 = POSSIBLE_BUILDING_MATERIALS[rng.next() as usize % POSSIBLE_BUILDING_MATERIALS.len()];
                
        //         const DOOR_HEIGHT: usize = MIN_BUILDING_DIM as usize;
        
        //         // trace("beginning spawn top/bottom");

        //         for col in building_chunk_loc_x..building_chunk_loc_x+building_width {
        //             // trace(format!("{col} {building_chunk_loc_x} {building_width} {building_chunk_loc_y}"));
        //             let corrupt_material: u8 = CORRUPT_MATERIALS[rng.next() as usize % CORRUPT_MATERIALS.len()]; 
        //             let material = get_material(building_material, corrupt_material, CORRUPT_CHANCE, rng);
        //             // trace("used rng");
        //             // top
        //             chunk.set_tile(col, building_chunk_loc_y, material);
        //             // tiles[building_chunk_loc_y][col] = material;
                    
        //             // trace("set tile 1");
        //             let material2 = get_material(building_material, corrupt_material, CORRUPT_CHANCE, rng);
        //             // bottom
        //             chunk.set_tile(col, building_chunk_loc_y + building_height, material2);
        //             // tiles[building_chunk_loc_y + building_height][col] = material2;
        //         }
        
        //         // trace("finished spawning top bottom");
        //         // // door
        //         let door_x: usize;
        //         let no_door_x: usize;
        
        //         if are_doors_on_right {
        //             door_x = building_chunk_loc_x;
        //             no_door_x = building_chunk_loc_x + building_width - 1;
        //         } else {
        //             door_x = building_chunk_loc_x + building_width - 1;
        //             no_door_x = building_chunk_loc_x;
        //         }
        //         for row in building_chunk_loc_y..=building_chunk_loc_y+building_height  {
        //             // left
                    
        
                    
        
        //             // door
                    
        //             if !USING_DOORS || row == building_chunk_loc_y + building_height || (row as i32) < building_chunk_loc_y as i32 + building_height as i32 - DOOR_HEIGHT as i32 {
        //                 // right
        //                 let corrupt_material: u8 = CORRUPT_MATERIALS[rng.next() as usize % CORRUPT_MATERIALS.len()]; 
        //                 let material = get_material(building_material, corrupt_material, CORRUPT_CHANCE, rng);
        //                 chunk.set_tile(door_x, row, material);
        //                 // tiles[row][door_x] = material;
        //             }
        //             let corrupt_material: u8 = CORRUPT_MATERIALS[rng.next() as usize % CORRUPT_MATERIALS.len()]; 
        //             let material2 = get_material(building_material, corrupt_material, CORRUPT_CHANCE, rng);
        //             chunk.set_tile(no_door_x, row, material2);
        //             // tiles[row][no_door_x] = material2;
        //         }
        //         // trace("finished door");

        //         for i in 0..N_BUILDINGS_PER_CHUNK {
        //             for row in inside_start_ys[i]..inside_end_ys[i] {
        //                 for col in inside_start_xs[i]..inside_end_xs[i] {
        //                     chunk.set_tile(col as usize, row as usize, 0);
        //                     // tiles[row as usize][col as usize] = 0;
        //                 }
        //             }
        //         }
        //         // trace("finished deleting building insides");
        //     }
        //     // trace("Finished spawning structure");
        // }


        // spawn_rectangular_structures(&mut chunk, rng);
        
        map.add_chunk(chunk);
    }

    // for chunk in &mut chunks {
    //     let tiles = &mut chunk.tiles;
    //     for col in 0..MAP_CHUNK_N_COLS {
    //         tiles[MAP_CHUNK_N_ROWS - GROUND_TILE_OFFSET][col] = 1;
    //     }
    // }
    // for chunk in &mut chunks {
    //     const WIGGLE_ROOM: i32 = 1;
    //     let tiles = &mut chunk.tiles;
    //     for row in 0..MAP_CHUNK_N_ROWS - GROUND_TILE_OFFSET - WIGGLE_ROOM as usize - 1 {
    //         for col in WIGGLE_ROOM as usize..MAP_CHUNK_N_COLS - WIGGLE_ROOM as usize {
    //             let mut rand_num = rng.next() as u8;
    //             rand_num %= 9;
    //             if rand_num >= 9 {
    //                 rand_num = 0;
    //             } else {
    //                 rand_num += 5;
    //             }
                
    //             tiles[row][col] = rand_num;
    //         }
    //     }
        
    // }
    // for row in 0..MAP_CHUNK_N_ROWS - GROUND_TILE_OFFSET {
    //     chunks[0].tiles[row][0] = 2;
    //     let l = chunks.len() - 1;
    //     chunks[l].tiles[row][MAP_CHUNK_N_ROWS - 1] = 3;
    // }


    
}

thread_local!(static GAME_STATE_HOLDER: RefCell<GameState<'static>> = RefCell::new(GameState::new()));

fn check_absolute_point_inside_tile_aligned_bound(x: i32, y: i32, bound: &TileAlignedBoundingBox) -> bool {
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

struct AbsoluteBoundingBox {
    x: i32,
    y: i32,
    width: usize,
    height: usize
}
fn check_absolue_bound_partially_inside_tile_aligned_bound(absolute_bound: &AbsoluteBoundingBox, tile_aligned_bound: &TileAlignedBoundingBox) -> bool {
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

fn raycast_axis_aligned(horizontal: bool, dist_per_iter: f32, abs_start_pt: (i32, i32), ray_displacement: f32, chunk: &MapChunk) -> (f32, bool) {

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

fn update_pos(map: &GameMap, moving_entity: MovingEntity, input: u8) {
    
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
        spritesheet::KITTY_SS_FLAGS | if the_char.facing_right { 0 } else { BLIT_FLIP_X },
    );
}

static mut PREVIOUS_GAMEPAD: [u8; 4] = [0, 0, 0, 0];
#[no_mangle]
fn update() {
    GAME_STATE_HOLDER.with(|game_cell| {
        let mut game_state = game_cell.borrow_mut();
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
                        game_state.camera.borrow_mut().current_viewing_x_offset = num::clamp(player.x_pos - 80.0, X_LEFT_BOUND as f32, X_RIGHT_BOUND as f32);
                        game_state.camera.borrow_mut().current_viewing_y_offset = num::clamp(player.y_pos - 80.0, Y_LOWER_BOUND as f32, Y_UPPER_BOUND as f32);
                        
                    }
                }
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
                let mut inputs: Vec<u8> = vec![];
        
                for _ in 0..game_state.npcs.borrow().len() {
                    let rngg = &mut game_state.rng.borrow_mut();
                    let rand_val = (rngg.next() % 255) as u8;
                    if rand_val < 20 {
                        inputs.push(0x10);
                    }
                    else if rand_val < 40 {
                        inputs.push(0x20);
                    }
                    else if rand_val < 42 {
                        inputs.push(BUTTON_1);
                    }
                    else {
                        inputs.push(0x0);
                    }
                    
                }
        
                for (i, npc) in game_state.npcs.borrow_mut().iter_mut().enumerate() {
                    update_pos(&game_state.map, MovingEntity::NPC(npc), inputs[i]);
                    drawcharacter(&game_state.spritesheet, &game_state.spritesheet_stride, &game_state.camera.borrow(), MovingEntity::NPC(npc));
                }

                
                drawmap(&game_state);
                
                if btns_pressed_this_frame[0] & BUTTON_2 != 0 {
                    regenerate_map(&mut game_state);
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
                    regenerate_map(&mut game_state);
                }
            }
        }
        
        


    });
}
