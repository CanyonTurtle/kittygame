// ideas
//
// custom tilemap code? or hand write? custom tilemap code is preferrable.

#[cfg(feature = "buddy-alloc")]
mod alloc;
mod spritesheet;
mod wasm4;
use num;
use std::cell::RefCell;
use wasm4::*;

// BUILDING PROFILE #1: baseline
const BUILDING_SUGGESTED_MIN_WIDTH: usize = 8; // 3;
const BUILDING_SUGGESTED_MAX_WIDTH: usize = 11; // 14;
const BUILDING_SUGGESTED_MIN_HEIGHT: usize = 2; // 3;
const BUILDING_SUGGESTED_MAX_HEIGHT: usize = 6; // 12;

const N_BUILDINGS_PER_CHUNK: usize = 30;
const USING_DOORS: bool = true;
const MAP_CHUNK_MIN_N_ROWS: usize = 5;
const MAP_CHUNK_MAX_N_ROWS: usize = 60;
const MAP_CHUNK_MIN_N_COLS: usize = 5;
const MAP_CHUNK_MAX_N_COLS: usize = 60;
const MAP_CHUNK_MAX_N_TILES: usize = 400;

const TOTAL_TILES_IN_MAP: usize = 20000;

const N_NPCS: i32 = 14;
  
const TILE_WIDTH_PX: usize = 5;
const TILE_HEIGHT_PX: usize = 5;

const X_LEFT_BOUND: i32 = -2000;
const X_RIGHT_BOUND: i32 = 2000;
const Y_LOWER_BOUND: i32 = -500;
const Y_UPPER_BOUND: i32 = 500;

const MIN_BUILDING_DIM: i32 = 4;

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
            x_pos: x_pos as f32,
            y_pos: 0.0,
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


struct MapChunk {
    tiles: Vec<u8>,
    chunk_i: i32,
    chunk_j: i32,
    chunk_width: i32,
    chunk_height: i32,
}

impl MapChunk {

    fn init() -> Self {
        let chunk = MapChunk {
            tiles: vec![0],
            chunk_i: 1,
            chunk_j: 1,
            chunk_width: 1,
            chunk_height: 1,
        };

        chunk
    }

    fn clamp_coords(self: &Self, x: usize, y: usize) -> (usize, usize) {
        let clamped_x = num::clamp(x, 0, self.chunk_width as usize - 1);
        let clamped_y = num::clamp(y, 0, self.chunk_height as usize - 1);
        (clamped_x, clamped_y)
    }
    fn set_tile(self: &mut Self, x: usize, y: usize, val: u8) {
        let clamped_coords = self.clamp_coords(x, y);

        self.tiles[clamped_coords.1 * self.chunk_width as usize + clamped_coords.0] = val;
    }

    fn get_tile(self: &Self, x: usize, y: usize) -> u8 {
        let clamped_coords = self.clamp_coords(x, y);
        self.tiles[clamped_coords.1 * self.chunk_width as usize + clamped_coords.0]
    }

    fn reset_chunk(self: &mut Self) {
        self.tiles.clear();
        for _ in 0..self.chunk_height {
            for _ in 0..self.chunk_width {
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
}

fn drawmap(game_state: &GameState) {
    let map = &game_state.map;
    let camera = &game_state.camera;

    for chunk in &map.chunks {
        for row in 0..chunk.chunk_height {
            for col in 0..chunk.chunk_width {
                let map_tile_i = chunk.get_tile(col as usize, row as usize);
                match map_tile_i {
                    0 => {},
                    tile_idx => {
                        let tile_i: usize = tile_idx as usize - 1; // *tile_idx as usize;
                        // trace(format!("Tile {tile_i}"));
                        let chunk_x_offset: i32 = (TILE_WIDTH_PX) as i32 * chunk.chunk_j;
                        let chunk_y_offset: i32 = (TILE_HEIGHT_PX) as i32 * chunk.chunk_i;
                        let x_loc = (chunk_x_offset + col as i32 * TILE_HEIGHT_PX as i32) - camera.current_viewing_x_offset as i32;
                        let y_loc = (chunk_y_offset + row as i32 * TILE_WIDTH_PX as i32) - camera.current_viewing_y_offset as i32;

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

struct GameState<'a> {
    player_1: Character,
    npcs: Vec<Character>,
    spritesheet: &'a [u8],
    spritesheet_stride: usize,
    background_tiles: Vec<spritesheet::Sprite>,
    map: GameMap,
    camera: Camera,
    rng: Rng,
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

    let mut current_chunk_locations: Vec<(i32, i32, i32, i32)> = vec![(0, 0, 48, 16)];

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
                chunk_wid = MAP_CHUNK_MIN_N_COLS + (rng.next() as usize % (MAP_CHUNK_MAX_N_COLS - MAP_CHUNK_MIN_N_COLS));
                chunk_hei = MAP_CHUNK_MAX_N_ROWS + (rng.next() as usize % (MAP_CHUNK_MAX_N_ROWS - MAP_CHUNK_MIN_N_ROWS));
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

            let r_offs_1: i32 = rng.next() as i32 % 6 - 3;

            let random_chunk_from_list_i = (rng.next() % current_chunk_locations.len() as u64) as usize;
            let horizontal_edge = rng.next() % 2 == 1;
            let positive_side = rng.next() % 2 == 1;
            let rand_chunk = &current_chunk_locations[random_chunk_from_list_i];
            let new_chunk_location: (i32, i32, i32, i32);

            const VARIATION_FROM_CHUNK_DIMS: usize = 3;

            let n_chunk_wid = num::clamp(MAP_CHUNK_MIN_N_COLS + (rng.next() as usize % (MAP_CHUNK_MAX_N_COLS - MAP_CHUNK_MIN_N_COLS)), MAP_CHUNK_MIN_N_COLS, rand_chunk.2 as usize + VARIATION_FROM_CHUNK_DIMS);
            let n_chunk_hei = num::clamp(MAP_CHUNK_MIN_N_ROWS + (rng.next() as usize % (MAP_CHUNK_MAX_N_ROWS - MAP_CHUNK_MIN_N_ROWS)), MAP_CHUNK_MIN_N_ROWS, rand_chunk.3 as usize + VARIATION_FROM_CHUNK_DIMS);

            if positive_side {
                if horizontal_edge {
                    
                    new_chunk_location = (rand_chunk.0 - chunk_hei as i32, rand_chunk.1 + r_offs_1, n_chunk_wid as i32, chunk_hei as i32);
                } else {
                    new_chunk_location = (rand_chunk.0 + r_offs_1, rand_chunk.1 + chunk_wid as i32, chunk_wid as i32, n_chunk_hei as i32);
                }
            } else {
                if horizontal_edge {
                    new_chunk_location = (rand_chunk.0 + chunk_hei as i32, rand_chunk.1 + r_offs_1, n_chunk_wid as i32, chunk_hei as i32);
                } else {
                    new_chunk_location = (rand_chunk.0 + r_offs_1, rand_chunk.1 - chunk_wid as i32, chunk_wid as i32, n_chunk_hei as i32);
                }
            }
            let mut is_viable_spot = true;
            for other_chunk in &current_chunk_locations {
                // if it collides with existing chunk, disallow
                if new_chunk_location.0 + new_chunk_location.3 > other_chunk.0 {
                    if new_chunk_location.0 < other_chunk.0 + other_chunk.3 {
                        if new_chunk_location.1 + new_chunk_location.2 > other_chunk.1 {
                            if new_chunk_location.1 < other_chunk.1 + other_chunk.2 {
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
        
        chunk.chunk_i = current_chunk_location.0;
        chunk.chunk_j = current_chunk_location.1;
        chunk.chunk_width = current_chunk_location.2;
        chunk.chunk_height = current_chunk_location.3;
        // chunk.tiles.clear();
        // for _ in 0..chunk.chunk_height {
        //     let mut chunk_row: Vec<u8> = Vec::new();
        //     for _ in 0..chunk.chunk_width {
        //         chunk_row.push(0);
        //     }
        //     chunk.tiles.push(chunk_row);
        // }
        chunk.reset_chunk();
        
        

        // for col in 0..MAP_CHUNK_N_COLS {
        //     tiles[MAP_CHUNK_N_ROWS - GROUND_TILE_OFFSET][col] = 1;
        // }
        
        

        

        fn spawn_rectangular_structures(chunk: &mut MapChunk, rng: &mut Rng) {
            // trace(format!("spawning structure with {chunk_width} {chunk_height}"));
            let mut inside_start_xs: [u8; N_BUILDINGS_PER_CHUNK] = [0; N_BUILDINGS_PER_CHUNK];
            let mut inside_start_ys: [u8; N_BUILDINGS_PER_CHUNK] = [0; N_BUILDINGS_PER_CHUNK];
            let mut inside_end_xs: [u8; N_BUILDINGS_PER_CHUNK] = [0; N_BUILDINGS_PER_CHUNK];
            let mut inside_end_ys: [u8; N_BUILDINGS_PER_CHUNK] = [0; N_BUILDINGS_PER_CHUNK];

            let are_doors_on_right: bool = (rng.next() as u8) < 127;

            for i in 0..N_BUILDINGS_PER_CHUNK {

                // trace("starting spawn structure");


                let building_min_width: usize = num::clamp(BUILDING_SUGGESTED_MIN_WIDTH, 1, chunk.chunk_width as usize);
                let building_max_width: usize = num::clamp(BUILDING_SUGGESTED_MAX_WIDTH, building_min_width, chunk.chunk_width as usize);

                let building_min_height: usize = num::clamp(BUILDING_SUGGESTED_MIN_HEIGHT, 1, chunk.chunk_height as usize);
                let building_max_height: usize = num::clamp(BUILDING_SUGGESTED_MAX_HEIGHT, building_min_height, chunk.chunk_height as usize);
 
                // trace("established mins and maxes");
                const POSSIBLE_BUILDING_MATERIALS: [u8; 1] = [6];
                const CORRUPT_MATERIALS: [u8; 7] = [7, 8, 9, 10, 11, 12, 13];
                const CORRUPT_CHANCE: f32 = 0.2;
                
                fn get_material(normal: u8, corrupt: u8, chance: f32, rng: &mut Rng) -> u8 {
                    if (rng.next() as u8 % 255) as f32 > 255.0 * chance {
                        return normal;
                    }
                    corrupt
                }
        
                // spawn structure
                let building_width: usize = building_min_width + (rng.next() as usize % (core::cmp::max(building_max_width - building_min_width, 1)));
                let building_height: usize = building_min_height + (rng.next() as usize % (core::cmp::max(building_max_height - building_min_height, 1)));
        
                // trace("got building dims");

                // trace(format!("Building width: {building_width}, chunk_width: {chunk_width} "));
                // trace(format!("Building height: {building_height}, chunk_height: {chunk_height} "));


                let building_chunk_loc_x: usize = (rng.next() as u64 % (core::cmp::max(chunk.chunk_width as i64 - building_width as i64, 1)) as u64) as usize;
                let building_chunk_loc_y: usize = (rng.next() as u64 % (core::cmp::max(chunk.chunk_height as i64 - building_height as i64, 1)) as u64) as usize;
        
                // trace("got modded loc");
                inside_start_xs[i] = building_chunk_loc_x as u8 + 1;
                inside_start_ys[i] = building_chunk_loc_y as u8 + 1;
                inside_end_xs[i] = building_chunk_loc_x as u8 + building_width as u8 - 1;
                inside_end_ys[i] = building_chunk_loc_y as u8 + building_height as u8 - 1;


                let building_material: u8 = POSSIBLE_BUILDING_MATERIALS[rng.next() as usize % POSSIBLE_BUILDING_MATERIALS.len()];
                
                const DOOR_HEIGHT: usize = MIN_BUILDING_DIM as usize;
        
                // trace("beginning spawn top/bottom");

                for col in building_chunk_loc_x..building_chunk_loc_x+building_width {
                    // trace(format!("{col} {building_chunk_loc_x} {building_width} {building_chunk_loc_y}"));
                    let corrupt_material: u8 = CORRUPT_MATERIALS[rng.next() as usize % CORRUPT_MATERIALS.len()]; 
                    let material = get_material(building_material, corrupt_material, CORRUPT_CHANCE, rng);
                    // trace("used rng");
                    // top
                    chunk.set_tile(col, building_chunk_loc_y, material);
                    // tiles[building_chunk_loc_y][col] = material;
                    
                    // trace("set tile 1");
                    let material2 = get_material(building_material, corrupt_material, CORRUPT_CHANCE, rng);
                    // bottom
                    chunk.set_tile(col, building_chunk_loc_y + building_height, material2);
                    // tiles[building_chunk_loc_y + building_height][col] = material2;
                }
        
                // trace("finished spawning top bottom");
                // // door
                let door_x: usize;
                let no_door_x: usize;
        
                if are_doors_on_right {
                    door_x = building_chunk_loc_x;
                    no_door_x = building_chunk_loc_x + building_width - 1;
                } else {
                    door_x = building_chunk_loc_x + building_width - 1;
                    no_door_x = building_chunk_loc_x;
                }
                for row in building_chunk_loc_y..=building_chunk_loc_y+building_height  {
                    // left
                    
        
                    
        
                    // door
                    
                    if !USING_DOORS || row == building_chunk_loc_y + building_height || (row as i32) < building_chunk_loc_y as i32 + building_height as i32 - DOOR_HEIGHT as i32 {
                        // right
                        let corrupt_material: u8 = CORRUPT_MATERIALS[rng.next() as usize % CORRUPT_MATERIALS.len()]; 
                        let material = get_material(building_material, corrupt_material, CORRUPT_CHANCE, rng);
                        chunk.set_tile(door_x, row, material);
                        // tiles[row][door_x] = material;
                    }
                    let corrupt_material: u8 = CORRUPT_MATERIALS[rng.next() as usize % CORRUPT_MATERIALS.len()]; 
                    let material2 = get_material(building_material, corrupt_material, CORRUPT_CHANCE, rng);
                    chunk.set_tile(no_door_x, row, material2);
                    // tiles[row][no_door_x] = material2;
                }
                // trace("finished door");

                for i in 0..N_BUILDINGS_PER_CHUNK {
                    for row in inside_start_ys[i]..inside_end_ys[i] {
                        for col in inside_start_xs[i]..inside_end_xs[i] {
                            chunk.set_tile(col as usize, row as usize, 0);
                            // tiles[row as usize][col as usize] = 0;
                        }
                    }
                }
                // trace("finished deleting building insides");
            }
            // trace("Finished spawning structure");
        }


        spawn_rectangular_structures(&mut chunk, rng);
        
        map.chunks.push(chunk);
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
        let rng = Rng::new();
        GameState {
            player_1: Character::new(40, spritesheet::PresetSprites::MainCat),
            npcs: (0..N_NPCS).map(|mut x| {
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
            }).collect::<Vec<Character>>(),
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
            camera: Camera { current_viewing_x_offset: 0.0, current_viewing_y_offset: 0.0 },
            rng,
            game_mode: GameMode::StartScreen
        }
    }
}

thread_local!(static GAME_STATE_HOLDER: RefCell<GameState<'static>> = RefCell::new(GameState::new()));

fn update_pos(character: &mut Character, input: u8) {
    
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

    character.x_pos += character.x_vel;
    character.y_pos += character.y_vel;

    let gravity = 0.3;
    character.y_vel += gravity;

    character.x_pos = num::clamp(character.x_pos, X_LEFT_BOUND as f32, X_RIGHT_BOUND as f32);
    character.y_pos = num::clamp(character.y_pos, Y_LOWER_BOUND as f32, Y_UPPER_BOUND as f32);
    character.x_vel = num::clamp(character.x_vel, -character.x_vel_cap, character.x_vel_cap);
    character.y_vel = num::clamp(character.y_vel, -character.y_vel_cap, character.y_vel_cap);
    character.count += 1;
}

fn drawcharacter(spritesheet: &[u8], spritesheet_stride: &usize, camera: &Camera, character: &Character) {
    let i = character.current_sprite_i as usize;
    blit_sub(
        &spritesheet,
        character.x_pos as i32 - camera.current_viewing_x_offset as i32,
        character.y_pos as i32 - camera.current_viewing_y_offset as i32,
        character.sprite.frames[i].positioning.width as u32,
        character.sprite.frames[i].positioning.height as u32,
        character.sprite.frames[i].positioning.start_x as u32,
        character.sprite.frames[i].positioning.start_y as u32,
        *spritesheet_stride as u32,
        spritesheet::KITTY_SS_FLAGS | if character.facing_right { 0 } else { BLIT_FLIP_X },
    );
}

static mut PREVIOUS_GAMEPAD: u8 = 0;

#[no_mangle]
fn update() {
    GAME_STATE_HOLDER.with(|game_cell| {
        let mut game_state = game_cell.borrow_mut();
        let gamepad = unsafe { *GAMEPAD1 };
        let previous = unsafe {PREVIOUS_GAMEPAD};
        let pressed_this_frame = gamepad & (gamepad ^ previous);
        unsafe {PREVIOUS_GAMEPAD = gamepad};
        match game_state.game_mode {
            GameMode::NormalPlay => {
                
        
                
                update_pos(&mut game_state.player_1, gamepad);
        
                game_state.camera.current_viewing_x_offset = num::clamp(game_state.player_1.x_pos - 80.0, X_LEFT_BOUND as f32, X_RIGHT_BOUND as f32);
                game_state.camera.current_viewing_y_offset = num::clamp(game_state.player_1.y_pos - 80.0, Y_LOWER_BOUND as f32, Y_UPPER_BOUND as f32);
                // unsafe { *DRAW_COLORS = 0x1112 }
                // text("WELCOME TO KITTY GAME.          :D       xD                           WHAT IS POPPIN ITS YOUR BOY, THE KITTY GAME", 200 - game_state.camera.current_viewing_x_offset as i32, 130);
                
                // unsafe { *DRAW_COLORS = spritesheet::KITTY_SS_DRAW_COLORS }
                let mut inputs: Vec<u8> = vec![];
        
                for _ in 0..game_state.npcs.len() {
                    let rngg = &mut game_state.rng;
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
        
                for i in 0..game_state.npcs.len() {
                    update_pos(&mut game_state.npcs[i], inputs[i]);
                }
                for npc in &game_state.npcs {
                    drawcharacter(&game_state.spritesheet, &game_state.spritesheet_stride, &game_state.camera, &npc);
                }
                drawcharacter(&game_state.spritesheet, &game_state.spritesheet_stride, &game_state.camera, &game_state.player_1);
                drawmap(&game_state);
                
                if pressed_this_frame & BUTTON_2 != 0 {
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
            },
            GameMode::StartScreen => {
                unsafe { *DRAW_COLORS = 0x1112 }
                text("Any key: start", 20, 20);
                unsafe {
                    *PALETTE = spritesheet::KITTY_SS_PALLETE;
                }
                unsafe { *DRAW_COLORS = spritesheet::KITTY_SS_DRAW_COLORS }
                game_state.rng.next();
                if gamepad != 0 {
                    game_state.game_mode = GameMode::NormalPlay;
                    // drop(game_state.map.chunks);
                    text("Spawning map...", 20, 50);
                    regenerate_map(&mut game_state);
                }
            }
        }
        
        


    });
}
