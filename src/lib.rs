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

const BUILDING_SUGGESTED_MIN_WIDTH: usize = 5; // 3;
const BUILDING_SUGGESTED_MAX_WIDTH: usize = 10; // 14;
const BUILDING_SUGGESTED_MIN_HEIGHT: usize = 3; // 3;
const BUILDING_SUGGESTED_MAX_HEIGHT: usize = 15; // 12;

const N_BUILDINGS_PER_CHUNK: usize = 50;

const MAP_CHUNK_N_ROWS: usize = 32;
const MAP_CHUNK_N_COLS: usize = 32;
const MAP_N_CHUNKS: i32 = 10;
const N_NPCS: i32 = 14;

const GROUND_TILE_OFFSET: usize = 1;

const TILE_WIDTH_PX: usize = 5;
const TILE_HEIGHT_PX: usize = 5;

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
    tiles: [[u8; MAP_CHUNK_N_COLS]; MAP_CHUNK_N_ROWS],
    chunk_i: i32,
    chunk_j: i32,
}

struct GameMap {
    chunks: Vec<MapChunk>

}

fn drawmap(game_state: &GameState) {
    let map = &game_state.map;
    let camera = &game_state.camera;

    for chunk in &map.chunks {
        let tiles = chunk.tiles;
        for row in 0..tiles.len() {
            for col in 0..tiles[row].len() {
                let map_tile_i = &tiles[row][col];
                match map_tile_i {
                    0 => {},
                    tile_idx => {
                        let tile_i: usize = *tile_idx as usize - 1; // *tile_idx as usize;
                        // trace(format!("Tile {tile_i}"));
                        let chunk_x_offset: i32 = (TILE_WIDTH_PX * MAP_CHUNK_N_COLS) as i32 * chunk.chunk_j;
                        let chunk_y_offset: i32 = (TILE_HEIGHT_PX * MAP_CHUNK_N_ROWS) as i32 * chunk.chunk_i;
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

fn create_map(rng: &mut Rng) -> GameMap {
    let mut chunks: Vec<MapChunk> = (0..MAP_N_CHUNKS).map(|i| MapChunk {
        tiles: [[0; MAP_CHUNK_N_COLS]; MAP_CHUNK_N_ROWS],
        chunk_i: 0,
        chunk_j: i
    }).collect();

    let map = GameMap { chunks: chunks};


    map
}

fn regenerate_map(game_state: &mut GameState) {

    let chunks = &mut game_state.map.chunks;
    let rng = &mut game_state.rng;
    for chunk in chunks {

        chunk.tiles = [[0 as u8; MAP_CHUNK_N_COLS]; MAP_CHUNK_N_ROWS];

        let tiles = &mut chunk.tiles;
        // for col in 0..MAP_CHUNK_N_COLS {
        //     tiles[MAP_CHUNK_N_ROWS - GROUND_TILE_OFFSET][col] = 1;
        // }
        
        

        

        fn spawn_rectangular_structures(tiles: &mut [[u8; MAP_CHUNK_N_COLS]; MAP_CHUNK_N_ROWS], rng: &mut Rng) {
            let mut inside_start_xs: [u8; N_BUILDINGS_PER_CHUNK] = [0; N_BUILDINGS_PER_CHUNK];
            let mut inside_start_ys: [u8; N_BUILDINGS_PER_CHUNK] = [0; N_BUILDINGS_PER_CHUNK];
            let mut inside_end_xs: [u8; N_BUILDINGS_PER_CHUNK] = [0; N_BUILDINGS_PER_CHUNK];
            let mut inside_end_ys: [u8; N_BUILDINGS_PER_CHUNK] = [0; N_BUILDINGS_PER_CHUNK];

            let are_doors_on_right: bool = (rng.next() as u8) < 127;

            for i in 0..N_BUILDINGS_PER_CHUNK {



                let building_min_width: usize = num::clamp(BUILDING_SUGGESTED_MIN_WIDTH, 1, MAP_CHUNK_N_COLS);
                let building_max_width: usize = num::clamp(BUILDING_SUGGESTED_MAX_WIDTH, building_min_width, MAP_CHUNK_N_COLS);

                let building_min_height: usize = num::clamp(BUILDING_SUGGESTED_MIN_HEIGHT, 1, MAP_CHUNK_N_ROWS);
                let building_max_height: usize = num::clamp(BUILDING_SUGGESTED_MAX_HEIGHT, building_min_height, MAP_CHUNK_N_ROWS);
 
                const POSSIBLE_BUILDING_MATERIALS: [u8; 1] = [6];
                const CORRUPT_MATERIALS: [u8; 7] = [7, 8, 9, 10, 11, 12, 13];
                const CORRUPT_CHANCE: f32 = 0.35;
                
                fn get_material(normal: u8, corrupt: u8, chance: f32, rng: &mut Rng) -> u8 {
                    if (rng.next() as u8 % 255) as f32 > 255.0 * chance {
                        return normal;
                    }
                    corrupt
                }
        
                // spawn structure
                let building_width: usize = building_min_width + rng.next() as usize % (building_max_width - building_min_width);
                let building_height: usize = building_min_height + rng.next() as usize % (building_max_height - building_min_height);
        
        
                let building_chunk_loc_x: usize = 1 + rng.next() as usize % (MAP_CHUNK_N_COLS - building_width - 1) ;
                let building_chunk_loc_y: usize = 1 + rng.next() as usize % (MAP_CHUNK_N_ROWS - building_height - 1);
        
                inside_start_xs[i] = building_chunk_loc_x as u8 + 1;
                inside_start_ys[i] = building_chunk_loc_y as u8 + 1;
                inside_end_xs[i] = building_chunk_loc_x as u8 + building_width as u8;
                inside_end_ys[i] = building_chunk_loc_y as u8 + building_height as u8;


                let building_material: u8 = POSSIBLE_BUILDING_MATERIALS[rng.next() as usize % POSSIBLE_BUILDING_MATERIALS.len()];
                
                const DOOR_HEIGHT: usize = 3;
        
                for col in building_chunk_loc_x..building_chunk_loc_x+building_width {
                    let corrupt_material: u8 = CORRUPT_MATERIALS[rng.next() as usize % CORRUPT_MATERIALS.len()]; 
                    let material = get_material(building_material, corrupt_material, CORRUPT_CHANCE, rng);
                    // top
                    tiles[building_chunk_loc_y][col] = material;
        
                    let material2 = get_material(building_material, corrupt_material, CORRUPT_CHANCE, rng);
                    // bottom
                    tiles[building_chunk_loc_y + building_height][col] = material2;
                }
        
                // door
                let door_x: usize;
                let no_door_x: usize;
        
                if are_doors_on_right {
                    door_x = building_chunk_loc_x;
                    no_door_x = building_chunk_loc_x + building_width;
                } else {
                    door_x = building_chunk_loc_x + building_width;
                    no_door_x = building_chunk_loc_x;
                }
                for row in building_chunk_loc_y..=building_chunk_loc_y+building_height  {
                    // left
                    
        
                    
        
                    // door
                    
                    if row == building_chunk_loc_y + building_height || row < building_chunk_loc_y + building_height - DOOR_HEIGHT {
                        // right
                        let corrupt_material: u8 = CORRUPT_MATERIALS[rng.next() as usize % CORRUPT_MATERIALS.len()]; 
                        let material = get_material(building_material, corrupt_material, CORRUPT_CHANCE, rng);
                        tiles[row][door_x] = material;
                    }
                    let corrupt_material: u8 = CORRUPT_MATERIALS[rng.next() as usize % CORRUPT_MATERIALS.len()]; 
                    let material2 = get_material(building_material, corrupt_material, CORRUPT_CHANCE, rng);
                    tiles[row][no_door_x] = material2;
                }

                for i in 0..N_BUILDINGS_PER_CHUNK {
                    for row in inside_start_ys[i]..inside_end_ys[i] {
                        for col in inside_start_xs[i]..inside_end_xs[i] {
                            tiles[row as usize][col as usize] = 0;
                        }
                    }
                }
            }
            
        }


        spawn_rectangular_structures(tiles, rng);
        
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
        let mut rng = Rng::new();
        GameState {
            player_1: Character::new(40, spritesheet::PresetSprites::MainCat),
            npcs: (0..N_NPCS).map(|mut x| {
                x %= 6;
                let preset = match x {
                    0 => spritesheet::PresetSprites::Kitty1,
                    1 => spritesheet::PresetSprites::Kitty2,
                    2 => spritesheet::PresetSprites::Kitty3,
                    3 => spritesheet::PresetSprites::Kitty4,
                    4 => spritesheet::PresetSprites::Lizard,
                    5 => spritesheet::PresetSprites::Pig,
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
            map: create_map(&mut rng),
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

    character.x_pos = num::clamp(character.x_pos, 5.0, (TILE_WIDTH_PX * MAP_N_CHUNKS as usize * MAP_CHUNK_N_COLS - 5 - character.sprite.frames[character.current_sprite_i as usize].positioning.width) as f32);
    character.y_pos = num::clamp(character.y_pos, 0.0, 160.0 - TILE_HEIGHT_PX as f32 * GROUND_TILE_OFFSET as f32 - (character.sprite.frames[character.current_sprite_i as usize].positioning.height as f32));
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
        
                game_state.camera.current_viewing_x_offset = num::clamp(game_state.player_1.x_pos - 80.0, 0.0, MAP_N_CHUNKS as f32 * TILE_WIDTH_PX as f32 * MAP_CHUNK_N_COLS as f32);
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
                    drop(&mut game_state.map.chunks);
                    text("Spawning map...", 20, 50);
                    regenerate_map(&mut game_state);
                }
            }
        }
        
        


    });
}
