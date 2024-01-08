use core::cell::RefCell;

use super::cloud::Cloud;
use super::entities::Player;
use super::game_constants::{COUNTDOWN_TIMER_START, START_DIFFICULTY_LEVEL, LEVELS_PER_MOOD, MAP_GEN_SETTINGS};
use super::menus::GameMode;
use super::popup_text::PopTextRingbuffer;
use super::{
    camera::Camera,
    entities::{Character, OptionallyEnabledPlayer},
    game_constants::{
        // MAX_N_TILES_IN_CHUNK, MAP_CHUNK_MAX_SIDE_LEN, MAP_CHUNK_MIN_SIDE_LEN,
        MAX_N_NPCS,
        TILE_HEIGHT_PX, TILE_WIDTH_PX,
    },
    game_map::GameMap,
    mapchunk::{MapChunk, TileAlignedBoundingBox},
    rng::Rng,
};
use crate::game::ability_cards::AbilityCardStack;
use crate::game::game_map::MAP_TILESETS;
use crate::game::music::SONGS;
use crate::kitty_ss;
use crate::spritesheet::{self, KITTY_SPRITESHEET_PALETTES};

pub struct GameState<'a> {
    pub players: RefCell<[OptionallyEnabledPlayer; 4]>,
    pub npcs: RefCell<Vec<Character>>,
    pub spritesheet: &'a [u8],
    pub spritesheet_stride: usize,
    pub background_tiles: &'static Vec<spritesheet::Sprite>,
    pub map: GameMap,
    pub camera: RefCell<Camera>,
    pub rng: RefCell<Rng>,
    pub game_mode: GameMode,
    pub countdown_timer_msec: RefCell<u32>,
    pub countdown_paused: bool,
    pub godmode: bool,
    pub pallette_idx: usize,
    pub song_idx: usize,
    pub song_timer: u32,
    pub difficulty_level: u32,
    pub total_npcs_to_find: u32,
    pub score: RefCell<u32>,
    pub popup_text_ringbuffer: RefCell<PopTextRingbuffer>,
    pub tileset_idx: RefCell<usize>,
    pub map_gen_settings_idx: RefCell<usize>,
    pub tutorial_text_counter: RefCell<u8>,
    pub clouds: RefCell<Vec<Cloud>>
}

impl GameState<'static> {
    pub fn new() -> GameState<'static> {
        let characters = [
            OptionallyEnabledPlayer::Enabled(Player {
                character: Character::new(spritesheet::PresetSprites::MainCat),
                card_stack: AbilityCardStack { cards: Vec::new() },
            }),
            OptionallyEnabledPlayer::Disabled,
            OptionallyEnabledPlayer::Disabled,
            OptionallyEnabledPlayer::Disabled,
        ];

        let rng = Rng::new();

        // (0..N_NPCS).map(|mut x| {
        //     x %= 7;
        //     let preset = match x {
        //         0 => spritesheet::PresetSprites::Kitty1,
        //         1 => spritesheet::PresetSprites::Kitty2,
        //         2 => spritesheet::PresetSprites::Kitty3,
        //         3 => spritesheet::PresetSprites::Kitty4,
        //         4 => spritesheet::PresetSprites::Lizard,
        //         5 => spritesheet::PresetSprites::Pig,
        //         6 => spritesheet::PresetSprites::BirdIsntReal,
        //         _ => spritesheet::PresetSprites::Pig
        //     };
        //     Character::new(preset)
        // }).collect::<Vec<Character>>()

        GameState {
            players: RefCell::new(characters),
            npcs: RefCell::new(Vec::new()),

            spritesheet: kitty_ss::KITTY_SPRITESHEET,
            spritesheet_stride: spritesheet::KITTY_SPRITESHEET_STRIDE as usize,
            background_tiles: spritesheet::Sprite::get_spritesheet(),
            map: GameMap::create_map(),
            camera: RefCell::new(Camera {
                current_viewing_x_offset: 0.0,
                current_viewing_y_offset: 0.0,
                current_viewing_x_target: 0.0,
                current_viewing_y_target: 0.0,
            }),
            rng: RefCell::new(rng),
            game_mode: GameMode::StartScreen,
            countdown_timer_msec: RefCell::new(60 * 3),
            countdown_paused: false,
            godmode: false,
            pallette_idx: 0,
            song_idx: 0,
            song_timer: 0,
            difficulty_level: START_DIFFICULTY_LEVEL,
            total_npcs_to_find: 3,
            score: RefCell::new(0),
            popup_text_ringbuffer: RefCell::new(PopTextRingbuffer {
                texts: [None, None, None, None, None, None, None, None, None, None],
                next_avail_idx: 0,
            }),
            tileset_idx: RefCell::new(0),
            map_gen_settings_idx: RefCell::new(0),
            tutorial_text_counter: RefCell::new(0),
            clouds: RefCell::new(Vec::new()),
        }
    }

    pub fn new_game(self: &mut Self) {
        let cdt: &mut u32 = &mut self.countdown_timer_msec.borrow_mut();
        *cdt = COUNTDOWN_TIMER_START;
        *self.score.borrow_mut() = 0;
        *self.tutorial_text_counter.borrow_mut() = 0;
    }

    pub fn get_n_enabled_players(self: &Self) -> u8 {
        let mut sum = 0;
        for op in self.players.borrow().iter() {
            sum += match &op {
                OptionallyEnabledPlayer::Enabled(_) => 1,
                OptionallyEnabledPlayer::Disabled => 0,
            };
        }
        sum
    }

    pub fn regenerate_map(self: &mut Self) {
        self.godmode = false;


        
        let new_song_idx =
            1 + ((self.difficulty_level as usize - 1) / LEVELS_PER_MOOD) % (SONGS.len() - 1);

        let new_pallete_idx = ((self.difficulty_level as usize - 1) / LEVELS_PER_MOOD) % KITTY_SPRITESHEET_PALETTES.len();
        self.pallette_idx = new_pallete_idx;

        if new_song_idx != self.song_idx {
            self.song_timer = 0;
        }
        self.song_idx = new_song_idx;

        // set the tileset
        {
            *self.tileset_idx.borrow_mut() = ((self.difficulty_level as usize - 1) / LEVELS_PER_MOOD) % MAP_TILESETS.len();
        }
        

        // set the map generation settings
        {
            *self.map_gen_settings_idx.borrow_mut() = ((self.difficulty_level as usize - 1) / LEVELS_PER_MOOD) % MAP_GEN_SETTINGS.len();
        }



        let map_gen_setting = &MAP_GEN_SETTINGS[*self.map_gen_settings_idx.borrow()];
        let map_chunk_min_side_len = map_gen_setting.chunk_min_side_len;
        let map_chunk_max_side_len = map_gen_setting.chunk_max_side_len;
        let max_n_tiles_in_chunk = map_gen_setting.max_n_tiles_per_chunk;
        
        // an average-sized map is ~ 30x30 = 900 blocks. Anything smaller is more twisty and denser. Make those
        // twistier maps smaller by a linear factor.

        let max_n_tiles_in_map: u32 = (0.7 * 2048.0) as u32 + (map_gen_setting.linear_mapsize_mult * 0.25 * 2048.0) as u32 * self.difficulty_level * self.get_n_enabled_players() as u32;


        // self.timer = 0;
        let map = &mut self.map;
        map.num_tiles = 0;
        map.chunks.clear();
        let rng = &mut self.rng.borrow_mut();

        for optional_player in self.players.borrow_mut().iter_mut() {
            match optional_player {
                OptionallyEnabledPlayer::Enabled(p) => {
                    p.character.x_pos = 10.0;
                    p.character.y_pos = 10.0;
                    p.character.can_fly = false;
                }
                OptionallyEnabledPlayer::Disabled => {}
            }
        }

        let npcs = &mut self.npcs.borrow_mut();

        npcs.clear();

        self.total_npcs_to_find =
            (1 + (self.difficulty_level / 3) + rng.next() as u32 % 3).min(MAX_N_NPCS as u32);

        let countdown_and_score_bonus = (5 + self.difficulty_level.min(20)) * 60;

        let cdt: &mut u32 = &mut self.countdown_timer_msec.borrow_mut();
        *cdt += countdown_and_score_bonus;
        *self.score.borrow_mut() += countdown_and_score_bonus;

        match self.difficulty_level {
            1 => {
                *cdt = COUNTDOWN_TIMER_START;
                *self.score.borrow_mut() = 0;
            }
            _ => {}
        }

        // generate the NPCs before making the chunks.
        for _ in 0..self.total_npcs_to_find {
            let x = rng.next() % 1000;
            let preset = match x {
                0..=200 => spritesheet::PresetSprites::Kitty1,
                201..=400 => spritesheet::PresetSprites::Kitty2,
                401..=600 => spritesheet::PresetSprites::Kitty3,
                601..=800 => spritesheet::PresetSprites::Kitty4,
                801..=850 => spritesheet::PresetSprites::Lizard,
                851..=900 => spritesheet::PresetSprites::Pig,
                901..=950 => spritesheet::PresetSprites::BirdIsntReal,
                951..=1000 => spritesheet::PresetSprites::Kitty1,

                _ => unreachable!(),
            };
            npcs.push(Character::new(preset));
        }

        let mut current_chunk_locations: Vec<TileAlignedBoundingBox> = Vec::new();

        match current_chunk_locations.try_reserve(1) {
            Ok(_) => {
                current_chunk_locations.push(TileAlignedBoundingBox::init(0, 0, 32, 32));
            }
            Err(_) => {
                // crate::trace("fail");
                return;
            }
        }
        // place the chunks randomly.
        let mut tile_count = 0;

        

        // let max_n_chunks = 8 + self.difficulty_level * 4;
        'generate_chunks: loop {
            if tile_count >= max_n_tiles_in_map {
                break 'generate_chunks;
            }
            // attempt to place a new chunk
            // if in viable location, place this chunk
            'generate_one_chunk: loop {
                // trace("placing chunk");
                // choose a new viable chunk size

                let mut chunk_wid: usize;
                let mut chunk_hei: usize;
                'find_place_for_chunk: loop {
                    chunk_wid = map_chunk_min_side_len
                        + (rng.next() as usize % (map_chunk_max_side_len - map_chunk_min_side_len));
                    chunk_hei = map_chunk_min_side_len
                        + (rng.next() as usize % (map_chunk_max_side_len - map_chunk_min_side_len));
                    // trace(format!("{chunk_wid} {chunk_hei}"));
                    if chunk_hei * chunk_wid <= max_n_tiles_in_chunk {
                        if map.try_fit_chunk_into(chunk_wid, chunk_hei) {
                            break 'find_place_for_chunk;
                        } else {
                            // n_retries += 1;
                            // if n_retries > 1000 {
                            //     panic!("out of memory for the tilemap");
                            // }
                            break 'generate_chunks;
                        }
                    }
                }

                let r_offs_1: i32 = rng.next() as i32 % map_chunk_min_side_len as i32
                    - (map_chunk_min_side_len as f32 / 2.0) as i32;

                let random_chunk_from_list_i =
                    (rng.next() % current_chunk_locations.len() as u64) as usize;
                let vertical_stack = rng.next() % 2 == 1;
                let positive_stack = rng.next() % 2 == 1;
                let rand_bound = &current_chunk_locations[random_chunk_from_list_i];
                let new_chunk_location: TileAlignedBoundingBox;

                // const VARIATION_FROM_CHUNK_DIMS: usize = 0;

                // let n_chunk_wid = num::clamp(MAP_CHUNK_MIN_N_COLS + (rng.next() as usize % (MAP_CHUNK_MAX_N_COLS - MAP_CHUNK_MIN_N_COLS)), MAP_CHUNK_MIN_N_COLS, rand_chunk.width + VARIATION_FROM_CHUNK_DIMS);
                // let n_chunk_hei = num::clamp(MAP_CHUNK_MIN_N_ROWS + (rng.next() as usize % (MAP_CHUNK_MAX_N_ROWS - MAP_CHUNK_MIN_N_ROWS)), MAP_CHUNK_MIN_N_ROWS, rand_chunk.height + VARIATION_FROM_CHUNK_DIMS);

                if vertical_stack {
                    if positive_stack {
                        new_chunk_location = TileAlignedBoundingBox::init(
                            rand_bound.x + r_offs_1,
                            rand_bound.y + rand_bound.height as i32,
                            chunk_wid,
                            chunk_hei,
                        );
                    } else {
                        new_chunk_location = TileAlignedBoundingBox::init(
                            rand_bound.x + r_offs_1,
                            rand_bound.y - chunk_hei as i32,
                            chunk_wid,
                            chunk_hei,
                        );
                    }
                } else {
                    if positive_stack {
                        new_chunk_location = TileAlignedBoundingBox::init(
                            rand_bound.x + rand_bound.width as i32,
                            rand_bound.y + r_offs_1,
                            chunk_wid,
                            chunk_hei,
                        );
                    } else {
                        new_chunk_location = TileAlignedBoundingBox::init(
                            rand_bound.x - chunk_wid as i32,
                            rand_bound.y + r_offs_1,
                            chunk_wid,
                            chunk_hei,
                        );
                    }
                }
                let mut is_viable_spot = true;

                fn shares_enough_axes_with_other_bounds(
                    potential_bound: &TileAlignedBoundingBox,
                    source_bound: &TileAlignedBoundingBox,
                    side_len: usize,
                ) -> bool {
                    let b1: &TileAlignedBoundingBox = potential_bound;
                    let b2: &TileAlignedBoundingBox = source_bound;

                    fn do_for_one_side(
                        b1: &TileAlignedBoundingBox,
                        b2: &TileAlignedBoundingBox,
                        side_len: usize,
                    ) -> bool {
                        if b1.y + b1.height as i32 == b2.y {
                            if (b1.x + b1.width as i32 - b2.x).min(b2.x + b2.width as i32 - b1.x)
                                >= side_len as i32
                            {
                                return true;
                            } else {
                                return false;
                            }
                        }

                        if b1.x + b1.width as i32 == b2.x {
                            if (b1.y + b1.height as i32 - b2.y).min(b2.y + b2.height as i32 - b1.y)
                                >= side_len as i32
                            {
                                return true;
                            } else {
                                return false;
                            }
                        }
                        true
                    }

                    do_for_one_side(&b1, &b2, side_len) && do_for_one_side(&b2, b1, side_len)
                }

                // ensure it shares enough adjacency with source chunk
                if !shares_enough_axes_with_other_bounds(&rand_bound, &new_chunk_location, map_chunk_min_side_len) {
                    is_viable_spot = false;
                }

                for other_bound in &current_chunk_locations {
                    // if it collides with existing chunk, disallow
                    if new_chunk_location.y + new_chunk_location.height as i32 > other_bound.y {
                        if new_chunk_location.y < other_bound.y + other_bound.height as i32 {
                            if new_chunk_location.x + new_chunk_location.width as i32
                                > other_bound.x
                            {
                                if new_chunk_location.x < other_bound.x + other_bound.width as i32 {
                                    is_viable_spot = false;
                                }
                            }
                        }
                    }
                    // if it doesn't collide, but it share too little with any adjacent chunks, it's also invalid
                    if !shares_enough_axes_with_other_bounds(&other_bound, &new_chunk_location, map_chunk_min_side_len) {
                        is_viable_spot = false;
                    }
                }

                if is_viable_spot {
                    // trace(format!("pushing chunk {new_chunk_location:?}"));
                    match current_chunk_locations.try_reserve(1) {
                        Ok(_) => {
                            current_chunk_locations.push(new_chunk_location);
                            tile_count += (chunk_hei * chunk_wid) as u32;
                            map.num_tiles += chunk_hei * chunk_wid;
                            break 'generate_one_chunk;
                        }
                        Err(_) => {
                            break 'generate_chunks;
                        }
                    }
                }
            }
        }

        'init_the_chunks: for current_chunk_location in current_chunk_locations.into_iter() {
            let mut chunk = MapChunk::init();

            chunk.bound = current_chunk_location;

            match chunk.initialize() {
                true => {}
                false => {
                    break 'init_the_chunks;
                }
            }

            // for col in 0..MAP_CHUNK_N_COLS {
            //     tiles[MAP_CHUNK_N_ROWS - GROUND_TILE_OFFSET][col] = 1;
            // }

            //const CHUNK_BORDER_MATERIAL: u8 = 6;

            // const POSSIBLE_BUILDING_MATERIALS: [u8; 1] = [6];

            let corrupt_materials: [u8; 7] = [9, 10, 11, 12, 13, 14, 15];
            const CORRUPT_CHANCE: f32 = 0.2;

            fn get_material(normal: u8, corrupt: u8, chance: f32, rng: &mut Rng) -> u8 {
                if (rng.next() as u8 % 255) as f32 > 255.0 * chance {
                    return normal;
                }
                corrupt
            }

            // left and right walls
            for row in 1..chunk.bound.height - 1 as usize {
                let corrupt_material: u8 =
                    corrupt_materials[rng.next() as usize % corrupt_materials.len()];
                let left_material = get_material(7, corrupt_material, CORRUPT_CHANCE, rng);
                let right_material = get_material(3, corrupt_material, CORRUPT_CHANCE, rng);

                chunk.set_tile(0, row, left_material);
                chunk.set_tile(chunk.bound.width as usize - 1, row, right_material);
            }

            // top and bottom walls
            for col in 1..chunk.bound.width - 1 as usize {
                let corrupt_material: u8 =
                    corrupt_materials[rng.next() as usize % corrupt_materials.len()];
                let top_material = get_material(1, corrupt_material, CORRUPT_CHANCE, rng);
                let bottom_material = get_material(5, corrupt_material, CORRUPT_CHANCE, rng);
                chunk.set_tile(col, 0, top_material);
                chunk.set_tile(col, chunk.bound.height as usize - 1, bottom_material);
            }

            // corners
            chunk.set_tile(0, 0, 8);
            chunk.set_tile(chunk.bound.width as usize - 1, chunk.bound.height as usize - 1, 4);
            chunk.set_tile(chunk.bound.width as usize - 1, 0, 2);
            chunk.set_tile(0, chunk.bound.height as usize - 1, 6);

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

        // spawn npcs (disallow spawning in origin chunk)
        for i in 0..npcs.len() {
            let rand_chunk_i = rng.next() as usize % (map.chunks.len() - 1) + 1;
            let chunk: &MapChunk = &map.chunks[rand_chunk_i];
            npcs[i].x_pos = chunk.bound.x as f32 * TILE_WIDTH_PX as f32 + 10.0;
            npcs[i].y_pos = chunk.bound.y as f32 * TILE_HEIGHT_PX as f32 + 10.0;
        }

        // reset NPCs
        for npc in npcs.iter_mut() {
            npc.following_i = None;
        }
    }
}
