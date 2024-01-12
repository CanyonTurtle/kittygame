
use super::cloud::Cloud;
use super::entities::{Player, WarpAbility};
use super::game_constants::{COUNTDOWN_TIMER_START, START_DIFFICULTY_LEVEL, LEVELS_PER_MOOD, MAP_GEN_SETTINGS};
use super::menus::GameMode;
use super::popup_text::PopTextRingbuffer;
use super::rng::GameRng;
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

// Games can either be fixed-seed and timed for speedrunning, or random.
type RunSeed = u32;
pub enum RunType {
    Random,
    Speedrun(RunSeed)
}

// pub enum Difficulty {
//     Easy,
//     Medium,
//     Hard
// }

pub struct GameSettings {
    pub run_type: RunType,
    // pub difficulty: Difficulty
}


pub struct GameState<'a> {
    pub players: [OptionallyEnabledPlayer; 4],
    pub npcs: Vec<Character>,
    pub spritesheet: &'a [u8],
    pub spritesheet_stride: usize,
    pub background_tiles: &'static Vec<spritesheet::Sprite>,
    pub map: GameMap,
    pub camera: Camera,
    pub rng: GameRng,
    pub game_mode: GameMode,
    pub countdown_timer_msec: u32,
    pub countdown_paused: bool,
    pub godmode: bool,
    pub pallette_idx: usize,
    pub song_idx: usize,
    pub song_timer: u32,
    pub difficulty_level: u32,
    pub total_npcs_to_find: u32,
    pub score: u32,
    pub popup_text_ringbuffer: PopTextRingbuffer,
    pub tileset_idx: usize,
    pub map_gen_settings_idx: usize,
    pub tutorial_text_counter: u8,
    pub clouds: Vec<Cloud>,
    pub countdown_and_score_bonus: u32,
    pub settings: GameSettings,
    pub speedrun_timer_msec: u32,
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

        let rng = GameRng::Random(Rng::new());

        GameState {
            players: characters,
            npcs: Vec::new(),

            spritesheet: kitty_ss::KITTY_SPRITESHEET,
            spritesheet_stride: spritesheet::KITTY_SPRITESHEET_STRIDE as usize,
            background_tiles: spritesheet::Sprite::get_spritesheet(),
            map: GameMap::create_map(),
            camera: Camera {
                current_viewing_x_offset: 0.0,
                current_viewing_y_offset: 0.0,
                current_viewing_x_target: 0.0,
                current_viewing_y_target: 0.0,
            },
            rng,
            game_mode: GameMode::StartScreen,
            countdown_timer_msec: 60 * 3,
            countdown_paused: false,
            godmode: false,
            pallette_idx: 0,
            song_idx: 0,
            song_timer: 0,
            difficulty_level: START_DIFFICULTY_LEVEL,
            total_npcs_to_find: 3,
            score: 0,
            popup_text_ringbuffer: PopTextRingbuffer {
                texts: [None, None, None, None, None, None, None, None, None, None],
                next_avail_idx: 0,
            },
            tileset_idx: 0,
            map_gen_settings_idx: 0,
            tutorial_text_counter: 0,
            clouds: Vec::new(),
            countdown_and_score_bonus: 0,
            settings: GameSettings{
                run_type: RunType::Random,
                // difficulty: Difficulty::Medium
            },
            speedrun_timer_msec: 0,
        }
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
            self.tileset_idx = ((self.difficulty_level as usize - 1) / LEVELS_PER_MOOD) % MAP_TILESETS.len();
        }
        

        // set the map generation settings
        {
            self.map_gen_settings_idx = ((self.difficulty_level as usize - 1) / LEVELS_PER_MOOD) % MAP_GEN_SETTINGS.len();
        }



        let map_gen_setting = &MAP_GEN_SETTINGS[self.map_gen_settings_idx];
        let map_chunk_min_side_len = map_gen_setting.chunk_min_side_len;
        let map_chunk_max_side_len = map_gen_setting.chunk_max_side_len;
        let max_n_tiles_in_chunk = map_gen_setting.max_n_tiles_per_chunk;
        
        // an average-sized map is ~ 30x30 = 900 blocks. Anything smaller is more twisty and denser. Make those
        // twistier maps smaller by a linear factor.

        let max_n_tiles_in_map: u32 = (0.7 * 2048.0) as u32 + (map_gen_setting.linear_mapsize_mult * 0.25 * 2048.0) as u32 * self.difficulty_level;


        let map = &mut self.map;
        map.num_tiles = 0;
        map.chunks.clear();
        let rng = &mut self.rng;

        for optional_player in self.players.iter_mut() {
            match optional_player {
                OptionallyEnabledPlayer::Enabled(p) => {
                    p.character.x_pos = 10.0;
                    p.character.y_pos = 10.0;
                    p.character.can_fly = false;
                }
                OptionallyEnabledPlayer::Disabled => {}
            }
        }

        let npcs = &mut self.npcs;

        npcs.clear();

        self.total_npcs_to_find =
            (1 + (self.difficulty_level / 3) + rng.next_for_worldgen() as u32 % 3).min(MAX_N_NPCS as u32);

        self.countdown_and_score_bonus = (4 + self.difficulty_level.min(20) / 3) * 60;

        self.countdown_timer_msec += self.countdown_and_score_bonus;
        self.countdown_timer_msec = self.countdown_timer_msec.min(100 * 60 - 1);
        self.score += self.countdown_and_score_bonus;

        match self.difficulty_level {
            START_DIFFICULTY_LEVEL => {
                self.countdown_timer_msec = COUNTDOWN_TIMER_START;
                self.score = 0;
                self.tutorial_text_counter = 0;

                // Reset warping ability only on a new game. Keep the ability each level.
                for optional_player in self.players.iter_mut() {
                    match optional_player {
                        OptionallyEnabledPlayer::Enabled(p) => {
                            p.character.warp_ability = WarpAbility::CannotWarp;
                        }
                        OptionallyEnabledPlayer::Disabled => {}
                    }
                }
            }
            _ => {}
        }

        // generate the NPCs before making the chunks.
        for _ in 0..self.total_npcs_to_find {
            let x = rng.next_for_worldgen() % 1000;
            let preset = match x {
                0..=200 => spritesheet::PresetSprites::Kitty1, // 20 % chance
                201..=400 => spritesheet::PresetSprites::Kitty2, // 20 % chance
                401..=600 => spritesheet::PresetSprites::Kitty3, // 20 % chance
                601..=800 => spritesheet::PresetSprites::Kitty4, // 20 % chance
                801..=900 => spritesheet::PresetSprites::Pig, // 10 % chance
                901..=980 => spritesheet::PresetSprites::BirdIsntReal, // 8 % chance
                _ => spritesheet::PresetSprites::Lizard, // <2 % chance
            };
            npcs.push(Character::new(preset));
        }

        let mut current_chunk_locations: Vec<TileAlignedBoundingBox> = Vec::new();

        match current_chunk_locations.try_reserve(1) {
            Ok(_) => {
                current_chunk_locations.push(TileAlignedBoundingBox::init(0, 0, 32, 32));
            }
            Err(_) => {
                return;
            }
        }
        // place the chunks randomly.
        let mut tile_count = 0;

        'generate_chunks: loop {
            if tile_count >= max_n_tiles_in_map {
                break 'generate_chunks;
            }
            // attempt to place a new chunk
            // if in viable location, place this chunk
            'generate_one_chunk: loop {
                // choose a new viable chunk size

                let mut chunk_wid: usize;
                let mut chunk_hei: usize;
                'find_place_for_chunk: loop {
                    chunk_wid = map_chunk_min_side_len
                        + (rng.next_for_worldgen() as usize % (map_chunk_max_side_len - map_chunk_min_side_len));
                    chunk_hei = map_chunk_min_side_len
                        + (rng.next_for_worldgen() as usize % (map_chunk_max_side_len - map_chunk_min_side_len));
                    if chunk_hei * chunk_wid <= max_n_tiles_in_chunk {
                        if map.try_fit_chunk_into(chunk_wid, chunk_hei) {
                            break 'find_place_for_chunk;
                        } else {
                            break 'generate_chunks;
                        }
                    }
                }

                let r_offs_1: i32 = rng.next_for_worldgen() as i32 % map_chunk_min_side_len as i32
                    - (map_chunk_min_side_len as f32 / 2.0) as i32;

                let random_chunk_from_list_i =
                    (rng.next_for_worldgen() % current_chunk_locations.len() as u64) as usize;
                let vertical_stack = rng.next_for_worldgen() % 2 == 1;
                let positive_stack = rng.next_for_worldgen() % 2 == 1;
                let rand_bound = &current_chunk_locations[random_chunk_from_list_i];
                let new_chunk_location: TileAlignedBoundingBox;

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

            let corrupt_materials: [u8; 7] = [9, 10, 11, 12, 13, 14, 15];
            const CORRUPT_CHANCE: f32 = 0.2;

            fn get_material(normal: u8, corrupt: u8, chance: f32, rng: &mut GameRng) -> u8 {
                if (rng.next_for_worldgen() as u8 % 255) as f32 > 255.0 * chance {
                    return normal;
                }
                corrupt
            }

            // left and right walls
            for row in 1..chunk.bound.height - 1 as usize {
                let corrupt_material: u8 =
                    corrupt_materials[rng.next_for_worldgen() as usize % corrupt_materials.len()];
                let left_material = get_material(7, corrupt_material, CORRUPT_CHANCE, rng);
                let right_material = get_material(3, corrupt_material, CORRUPT_CHANCE, rng);

                chunk.set_tile(0, row, left_material);
                chunk.set_tile(chunk.bound.width as usize - 1, row, right_material);
            }

            // top and bottom walls
            for col in 1..chunk.bound.width - 1 as usize {
                let corrupt_material: u8 =
                    corrupt_materials[rng.next_for_worldgen() as usize % corrupt_materials.len()];
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

            map.add_chunk(chunk);
        }

        // spawn npcs (disallow spawning in origin chunk)
        for i in 0..npcs.len() {
            let rand_chunk_i = rng.next_for_worldgen() as usize % (map.chunks.len() - 1) + 1;
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
