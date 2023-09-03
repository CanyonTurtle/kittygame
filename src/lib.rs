// ideas
//
// custom tilemap code? or hand write? custom tilemap code is preferrable.
#![allow(unused)]
#[cfg(feature = "buddy-alloc")]
mod alloc;
mod spritesheet;
mod wasm4;
use num;
use std::cell::RefCell;
use wasm4::*;

// BUILDING PROFILE #1: baseline
// const BUILDING_SUGGESTED_MIN_WIDTH: usize = 8; // 3;
// const BUILDING_SUGGESTED_MAX_WIDTH: usize = 11; // 14;
// const BUILDING_SUGGESTED_MIN_HEIGHT: usize = 2; // 3;
// const BUILDING_SUGGESTED_MAX_HEIGHT: usize = 6; // 12;

// const N_BUILDINGS_PER_CHUNK: usize = 30;
// const USING_DOORS: bool = true;
const MAP_CHUNK_MIN_SIDE_LEN: usize = 7;
const MAP_CHUNK_MAX_SIDE_LEN: usize = 50;

const MAP_CHUNK_MAX_N_TILES: usize = 800;

const TOTAL_TILES_IN_MAP: usize = 30000;

const N_NPCS: i32 = 10;
  
const TILE_WIDTH_PX: usize = 5;
const TILE_HEIGHT_PX: usize = 5;

const X_LEFT_BOUND: i32 = -2000;
const X_RIGHT_BOUND: i32 = 2000;
const Y_LOWER_BOUND: i32 = -1000;
const Y_UPPER_BOUND: i32 = 1000;

// const MIN_BUILDING_DIM: i32 = 4;

#[derive(PartialEq, Eq, Hash)]
enum KittyStates {
    Idle,
    Moving1,
    Moving2,
    Jump,
}
impl Character {
    fn new(x_pos: i32, sprite_type: spritesheet::PresetSprites) -> Character {
        Character {
            x_pos: 10 as f32,
            y_pos: 10.0,
            x_vel: 0.0,
            y_vel: 0.0,
            x_vel_cap: 2.0,
            y_vel_cap: 7.0,
            count: 0,
            facing_right: true,
            state: KittyStates::Idle,
            current_sprite_i: 0,
            sprite: spritesheet::Sprite::from_preset(sprite_type),
        }
    }
}

#[derive(Clone, Copy)]

struct Camera {
    current_viewing_x_offset: f32,
    current_viewing_y_offset: f32,
}

struct TileAlignedBoundingBox {
    x: i32,
    y: i32,
    width: usize,
    height: usize
}

struct MapChunk {
    tiles: Vec<u8>,
    bound: TileAlignedBoundingBox
}

enum OutOfChunkBound {
    OUT
}

impl MapChunk {

    fn init() -> Self {
        let chunk = MapChunk {
            tiles: vec![],
            bound: TileAlignedBoundingBox {
                y: 1,
                x: 1,
                width: 1,
                height: 1,
            }
        };

        chunk
    }

    fn clamp_coords(self: &Self, x: usize, y: usize) -> (usize, usize) {
        let clamped_x = num::clamp(x, 0, self.bound.width as usize - 1);
        let clamped_y = num::clamp(y, 0, self.bound.height as usize - 1);
        (clamped_x, clamped_y)
    }
    fn set_tile(self: &mut Self, x: usize, y: usize, val: u8) {
        let clamped_coords = self.clamp_coords(x, y);

        self.tiles[clamped_coords.1 * self.bound.width as usize + clamped_coords.0] = val;
    }

    fn get_tile(self: &Self, x: usize, y: usize) -> u8 {
        let clamped_coords = self.clamp_coords(x, y);
        self.tiles[clamped_coords.1 * self.bound.width as usize + clamped_coords.0]
    }

    fn is_tile_idx_inside_tile_aligned_bound(self: &Self, x: i32, y: i32) -> bool {
        if x >= 0 {
            if x < self.bound.width as i32 {
                if y >= 0 {
                    if y < self.bound.height as i32 {
                        return true
                    }
                }
            }
        }
        false
    }

    fn get_tile_abs(self: &Self, abs_x: i32, abs_y: i32) -> Result<u8, OutOfChunkBound> {
        let rel_x = ((abs_x - self.bound.x * TILE_WIDTH_PX as i32) as f32 / TILE_WIDTH_PX as f32) as i32;
        let rel_y = ((abs_y - self.bound.y * TILE_HEIGHT_PX as i32) as f32 / TILE_HEIGHT_PX as f32) as i32;

        if self.is_tile_idx_inside_tile_aligned_bound(rel_x, rel_y) {
            return Result::Ok(self.get_tile(rel_x as usize, rel_y as usize));
        }
        return Result::Err(OutOfChunkBound::OUT);
    }

    fn reset_chunk(self: &mut Self) {
        self.tiles.clear();
        for _ in 0..self.bound.height {
            for _ in 0..self.bound.width {
                self.tiles.push(0);
            }
        }
    }
}

struct GameMap {
    chunks: Vec<MapChunk>,
    num_tiles: usize
}

impl GameMap {
    fn try_fit_chunk_into(self: &mut Self, width: usize, height: usize) -> bool {
        let new_tile_size = width * height;
        let new_prospective_size = self.num_tiles + new_tile_size;
        if new_prospective_size <= TOTAL_TILES_IN_MAP {
            self.num_tiles = new_prospective_size;
            return true;
        }
        false
    }

    fn link_chunk_to_touching_chunks(self: &mut Self, chunk: &mut MapChunk) {
        for other_chunk in self.chunks.iter_mut() {

            fn fuse_horizontal(
                chunk: &mut MapChunk,
                other_chunk: &mut MapChunk,
            ) {
                // check to see if the other chunk touches the top of the new chunk
                if 
                    other_chunk.bound.y + other_chunk.bound.height as i32 == chunk.bound.y &&
                    other_chunk.bound.x + other_chunk.bound.width as i32 > chunk.bound.x &&
                    other_chunk.bound.x < chunk.bound.x + chunk.bound.width as i32
                {
                    let min_x = core::cmp::max(chunk.bound.x, other_chunk.bound.x);
                    let max_x = core::cmp::min(chunk.bound.x + chunk.bound.width as i32, other_chunk.bound.x + other_chunk.bound.width as i32);
                    for absolute_coord_x in min_x + 1..max_x - 1 {
                        let rel_chunk_x = absolute_coord_x - chunk.bound.x;
                        if rel_chunk_x > 0 && rel_chunk_x < chunk.bound.width as i32 - 1 {
                            chunk.set_tile(rel_chunk_x as usize, 0 as usize, 0);
                        }
                        let rel_other_chunk_x = absolute_coord_x - other_chunk.bound.x;
                        if rel_other_chunk_x > 0 && rel_other_chunk_x < other_chunk.bound.width as i32 - 1 {
                            other_chunk.set_tile(rel_other_chunk_x as usize, other_chunk.bound.height as usize - 1, 0)
                        }
                    }
                }
            }

            fn fuse_vertical(
                chunk: &mut MapChunk,
                other_chunk: &mut MapChunk,
            ) {
                // check to see if the other chunk touches the left of the new chunk
                if 
                    other_chunk.bound.x + other_chunk.bound.width as i32 == chunk.bound.x &&
                    other_chunk.bound.y + other_chunk.bound.height as i32 > chunk.bound.y &&
                    other_chunk.bound.y < chunk.bound.y + chunk.bound.height as i32
                {
                    let min_y = core::cmp::max(chunk.bound.y, other_chunk.bound.y);
                    let max_y = core::cmp::min(chunk.bound.y + chunk.bound.height as i32, other_chunk.bound.y + other_chunk.bound.height as i32);
                    for absolute_coord_y in min_y + 1..max_y - 1 {
                        let rel_chunk_y = absolute_coord_y - chunk.bound.y;
                        if rel_chunk_y > 0 && rel_chunk_y < chunk.bound.height as i32 - 1 {
                            chunk.set_tile(0, rel_chunk_y as usize, 0);
                        }
                        let rel_other_chunk_y = absolute_coord_y - other_chunk.bound.y;
                        if rel_other_chunk_y > 0 && rel_other_chunk_y < other_chunk.bound.height as i32 - 1 {
                            other_chunk.set_tile(other_chunk.bound.width as usize - 1, rel_other_chunk_y as usize, 0)
                        }
                    }
                }
            }

            fuse_horizontal(chunk, other_chunk);
            fuse_horizontal(other_chunk, chunk);

            fuse_vertical(chunk, other_chunk);
            fuse_vertical(other_chunk, chunk);
            

            
            
        }
    }

    fn add_chunk(self: & mut Self, mut chunk: MapChunk) {
        self.link_chunk_to_touching_chunks(&mut chunk);
        self.chunks.push(chunk);
    }
}

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

struct Character {
    x_pos: f32,
    y_pos: f32,
    x_vel: f32,
    y_vel: f32,
    x_vel_cap: f32,
    y_vel_cap: f32,
    count: i32,
    facing_right: bool,
    state: KittyStates,
    current_sprite_i: i32,
    sprite: spritesheet::Sprite,
}

#[derive(Debug)]
pub struct Rng(u128);

impl Rng {
    pub fn new() -> Self {
        Self(0x7369787465656E2062797465206E756Du128 | 1)
    }

    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(0x2360ED051FC65DA44385DF649FCCF645);
        let rot = (self.0 >> 122) as u32;
        let xsl = ((self.0 >> 64) as u64) ^ (self.0 as u64);
        xsl.rotate_right(rot)     
    }
}

enum GameMode {
    StartScreen,
    NormalPlay
}


enum OptionallyEnabledPlayer {
    Enabled(Character),
    Disabled
}

enum MovingEntity<'a> {
    OptionalPlayer(&'a mut OptionallyEnabledPlayer),
    NPC(&'a mut Character)
}

struct GameState<'a> {
    players: RefCell<[OptionallyEnabledPlayer; 4]>,
    npcs: RefCell<Vec<Character>>,
    spritesheet: &'a [u8],
    spritesheet_stride: usize,
    background_tiles: Vec<spritesheet::Sprite>,
    map: GameMap,
    camera: RefCell<Camera>,
    rng: RefCell<Rng>,
    game_mode: GameMode,
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

    for npc in game_state.npcs.borrow_mut().iter_mut() {
        npc.x_pos = 10.0;
        npc.y_pos = 10.0;
    }

    impl TileAlignedBoundingBox {
        fn init(x: i32, y: i32, w: usize, h: usize) -> Self {
            return TileAlignedBoundingBox { x:x, y: y, width: w, height: h }
        }
    }

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

    for current_chunk_location in current_chunk_locations {
        let mut chunk = MapChunk::init();
        
        chunk.bound = current_chunk_location;

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

impl GameState<'static> {
    fn new() -> GameState<'static> {

        let characters = [
            OptionallyEnabledPlayer::Enabled(Character::new(0, spritesheet::PresetSprites::MainCat)),
            OptionallyEnabledPlayer::Enabled(Character::new(10, spritesheet::PresetSprites::MainCat)),
            OptionallyEnabledPlayer::Enabled(Character::new(20, spritesheet::PresetSprites::MainCat)),
            OptionallyEnabledPlayer::Enabled(Character::new(30, spritesheet::PresetSprites::MainCat)),
        ];

        let rng = Rng::new();
        GameState {
            players: RefCell::new(characters),
            npcs: RefCell::new((0..N_NPCS).map(|mut x| {
                x %= 7;
                let preset = match x {
                    0 => spritesheet::PresetSprites::Kitty1,
                    1 => spritesheet::PresetSprites::Kitty2,
                    2 => spritesheet::PresetSprites::Kitty3,
                    3 => spritesheet::PresetSprites::Kitty4,
                    4 => spritesheet::PresetSprites::Lizard,
                    5 => spritesheet::PresetSprites::Pig,
                    6 => spritesheet::PresetSprites::BirdIsntReal,
                    _ => spritesheet::PresetSprites::Pig
                };
                Character::new((x * 2000) % 300 , preset)
            }).collect::<Vec<Character>>()),
            // npcs: vec![
            //     Character::new(500, spritesheet::PresetSprites::Kitty2),
            //     Character::new(400, spritesheet::PresetSprites::Kitty3),
            //     Character::new(300, spritesheet::PresetSprites::Kitty4),
            //     Character::new(200, spritesheet::PresetSprites::Pig),
            //     Character::new(100, spritesheet::PresetSprites::Lizard),
            // ],
            spritesheet: &spritesheet::KITTY_SS,
            spritesheet_stride: spritesheet::KITTY_SS_STRIDE,
            background_tiles: vec![
                spritesheet::Sprite::from_preset(spritesheet::PresetSprites::LineTop),
                spritesheet::Sprite::from_preset(spritesheet::PresetSprites::LineLeft),
                spritesheet::Sprite::from_preset(spritesheet::PresetSprites::LineRight),
                spritesheet::Sprite::from_preset(spritesheet::PresetSprites::LineBottom),
                spritesheet::Sprite::from_preset(spritesheet::PresetSprites::SolidWhite),
                spritesheet::Sprite::from_preset(spritesheet::PresetSprites::SeethroughWhite),
                spritesheet::Sprite::from_preset(spritesheet::PresetSprites::TopleftSolidCorner),
                spritesheet::Sprite::from_preset(spritesheet::PresetSprites::ToprightSolidCorner),
                spritesheet::Sprite::from_preset(spritesheet::PresetSprites::BottomleftSolidCorner),
                spritesheet::Sprite::from_preset(spritesheet::PresetSprites::BottomrightSolidCorner),
                spritesheet::Sprite::from_preset(spritesheet::PresetSprites::ColumnTop),
                spritesheet::Sprite::from_preset(spritesheet::PresetSprites::ColumnMiddle),
                spritesheet::Sprite::from_preset(spritesheet::PresetSprites::ColumnBottom),
            ],
            map: create_map(),
            camera: RefCell::new(Camera { current_viewing_x_offset: 0.0, current_viewing_y_offset: 0.0 }),
            rng: RefCell::new(rng),
            game_mode: GameMode::StartScreen
        }
    }
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

fn raycast_axis_aligned(horizontal: bool, positive: bool, dist_per_iter: f32, abs_start_pt: (i32, i32), ray_len: f32, chunk: &MapChunk) -> f32 {
    let ray_x_dist_per_iter;
    let ray_y_dist_per_iter;
    
    let mut vertical_ray: f32 = 0.0;
    let mut horizontal_ray: f32 = 0.0;

    let mut ret_val = ray_len;

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
    loop {
        if horizontal {
            if horizontal_ray.abs() > ray_len.abs() {
                break
            }
        } else {
            if vertical_ray.abs() > ray_len.abs() {
                break
            }
        }
        
        // make sure this ray even is in the chunk in the first place
        match chunk.get_tile_abs(start_pt.0 + horizontal_ray as i32, start_pt.1 + vertical_ray as i32) {
            Ok(tile) => {
                if tile != 0 {
                    // text(format!["hit tile {lowerleft_vertical_ray}"], 10, 50);
                    if horizontal {
                        should_clamp_x = true;
                    } else {
                        should_clamp_y = true;
                    }
                    
                    break
                }
            }
            Err(e) => {
                break
            }
        }
        vertical_ray += ray_y_dist_per_iter;
        horizontal_ray += ray_x_dist_per_iter;

    }
    if should_clamp_x {
        ret_val = vertical_ray;
    } else if should_clamp_y {
        ret_val = horizontal_ray;
    }

    ret_val
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
                    return
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

   

    let mut actual_x_vel_this_frame: f32 = character.x_vel;
    let mut actual_y_vel_this_frame: f32 = character.y_vel;
    // trace("will check--------------------");
    // look at each chunk, and see if the player is inside it
    for (i, chunk) in map.chunks.iter().enumerate() {
        
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
                character.y_vel = raycast_axis_aligned(false, false, RAYCAST_DIST_PER_ITER, lowerleft_checker_location, character.y_vel, chunk);
                character.y_vel = raycast_axis_aligned(false, false, RAYCAST_DIST_PER_ITER, lowerright_checker_location, character.y_vel, chunk);
            }
            else if character.y_vel > 0.0 {
                character.y_vel = raycast_axis_aligned(false, true, RAYCAST_DIST_PER_ITER, upperleft_checker_location, character.y_vel, chunk);
                character.y_vel = raycast_axis_aligned(false, true, RAYCAST_DIST_PER_ITER, upperright_checker_location, character.y_vel, chunk);
            }

            if character.x_vel < 0.0 {
                character.x_vel = raycast_axis_aligned(true, false, RAYCAST_DIST_PER_ITER, lowerleft_checker_location, character.x_vel, chunk);
                character.x_vel = raycast_axis_aligned(true, false, RAYCAST_DIST_PER_ITER, upperleft_checker_location, character.x_vel, chunk);
            }
            else if character.x_vel > 0.0 {
                character.x_vel = raycast_axis_aligned(true, true, RAYCAST_DIST_PER_ITER, lowerright_checker_location, character.x_vel, chunk);
                character.x_vel = raycast_axis_aligned(true, true, RAYCAST_DIST_PER_ITER, upperright_checker_location, character.x_vel, chunk);
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
