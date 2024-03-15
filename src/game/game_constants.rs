// BUILDING PROFILE #1: baseline
// const BUILDING_SUGGESTED_MIN_WIDTH: usize = 8; // 3;
// const BUILDING_SUGGESTED_MAX_WIDTH: usize = 11; // 14;
// const BUILDING_SUGGESTED_MIN_HEIGHT: usize = 2; // 3;
// const BUILDING_SUGGESTED_MAX_HEIGHT: usize = 6; // 12;

// const N_BUILDINGS_PER_CHUNK: usize = 30;
// const USING_DOORS: bool = true;


pub struct MapGenSetting {
    pub chunk_min_side_len: usize,
    pub chunk_max_side_len: usize,
    pub max_n_tiles_per_chunk: usize,
    pub linear_mapsize_mult: f32,
}

pub const MAP_GEN_SETTINGS: [MapGenSetting; 9] = [
    MapGenSetting { // normal
        chunk_min_side_len: 6,
        chunk_max_side_len: 50,
        max_n_tiles_per_chunk: 400,
        linear_mapsize_mult: 1.0,
    },
    MapGenSetting{ // oblong
        chunk_min_side_len: 6,
        chunk_max_side_len: 25,
        max_n_tiles_per_chunk: 2000,
        linear_mapsize_mult: 1.0,
    },
    MapGenSetting{ // teeny tunnels
        chunk_min_side_len: 6,
        chunk_max_side_len: 12,
        max_n_tiles_per_chunk: 800,
        linear_mapsize_mult: 0.7,
    },
    MapGenSetting{ // large normal
        chunk_min_side_len: 20,
        chunk_max_side_len: 40,
        max_n_tiles_per_chunk: 1400,
        linear_mapsize_mult: 1.0,
    },
    MapGenSetting{ // small
        chunk_min_side_len: 6,
        chunk_max_side_len: 30,
        max_n_tiles_per_chunk: 500,
        linear_mapsize_mult: 1.0,
    },
    MapGenSetting{ // ridiculous tunnel
        chunk_min_side_len: 8,
        chunk_max_side_len: 10,
        max_n_tiles_per_chunk: 500,
        linear_mapsize_mult: 0.4,
    },
    MapGenSetting{ // chaotic
        chunk_min_side_len: 5,
        chunk_max_side_len: 60,
        max_n_tiles_per_chunk: 500,
        linear_mapsize_mult: 1.0,
    },
    MapGenSetting{ // spacey
        chunk_min_side_len: 12,
        chunk_max_side_len: 40,
        max_n_tiles_per_chunk: 2000,
        linear_mapsize_mult: 1.0,
    },
    MapGenSetting{ // underworld
        chunk_min_side_len: 5,
        chunk_max_side_len: 80,
        max_n_tiles_per_chunk: 5000,
        linear_mapsize_mult: 1.0,
    },
];

// pub const MAP_CHUNK_MIN_SIDE_LEN: usize = 6;
// pub const MAP_CHUNK_MAX_SIDE_LEN: usize = 50;

// pub const MAX_N_TILES_IN_CHUNK: usize = 400;

// The whole map cannot take more than ~25 kb (1/2 byte per tile)
pub const MAX_N_TILES_IN_WHOLE_MAP: usize = 25 * 2048;

pub const MAX_N_NPCS: usize = 20;
  
pub const TILE_WIDTH_PX: usize = 5;
pub const TILE_HEIGHT_PX: usize = 5;

pub const X_LEFT_BOUND: i32 = -5000;
pub const X_RIGHT_BOUND: i32 = 5000;
pub const Y_LOWER_BOUND: i32 = -5000;
pub const Y_UPPER_BOUND: i32 = 5000;

pub const COUNTDOWN_TIMER_START: u32 = 60 * 60;

pub const START_DIFFICULTY_LEVEL: u32 = 1;
pub const FINAL_LEVEL: u32 = 45;
pub const LEVELS_PER_MOOD: usize = 5;

pub const MAJOR_VERSION: u8 = 1;
pub const MINOR_VERSION: u8 = 2;
pub const INCR_VERSION: u8 = 1;