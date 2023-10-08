use super::{mapchunk::MapChunk, game_constants::MAX_N_TILES_IN_WHOLE_MAP};

pub struct GameMap {
    pub chunks: Vec<MapChunk>,
    pub num_tiles: usize
}

// pub struct MapTileSet {
//     pub top: u8,
//     pub top_right: u8,
//     pub right: u8,
//     pub bottom_right: u8,
//     pub bottom: u8,
//     pub bottom_left: u8,
//     pub left: u8,
//     pub topleft: u8,
//     pub middle: u8,
//     pub corrupt_materials: [u8; 8],
// }

// nothing     top      topright   right 
// bottomright bottom   bottomleft left
// topleft     middle   corrupt1   c2
// c3          c4       c5         c6  
pub const MAP_TILESETS: [[u8; 16]; 9] = [
    [ // normal
        0, 20, 0, 17,
        0, 18, 0, 19,
        0, 12, 9, 10,
        11, 12, 13, 14
    ],
    [ // oblong
        0, 20, 10, 9,
        13, 18, 11, 9,
        8, 9, 9, 10,
        11, 12, 13, 14
    ],
    [ // teeny tunnels
        0, 9, 27, 9,
        29, 9, 28, 9,
        25, 9, 9, 10,
        11, 12, 13, 14
    ],
    [ // large normal
        0, 12, 27, 12,
        29, 12, 28, 12,
        25, 30, 9, 10,
        11, 32, 31, 14
    ],
    [ // small (jungle vibes)
        0, 12, 27, 14,
        29, 12, 28, 14,
        25, 12, 9, 31,
        32, 32, 31, 32
    ],
    // for clouds stage use pillars and platforms! :D
    [ // rediculous tunnels
        0, 12, 10, 12,
        13, 12, 11, 12,
        8, 9, 9, 9,
        31, 9, 32, 32
    ],
    [ // chaotic
        0, 20, 0, 30,
        0, 18, 0, 30,
        0, 30, 31, 27,
        29, 25, 28, 32
    ],
    [ // spacey
        0, 20, 10, 15,
        13, 12, 11, 15,
        8, 12, 12, 10,
        11, 8, 13, 12
    ],
    [ // underworld
        26, 32, 26, 32,
        26, 32, 26, 32,
        0, 26, 25, 27,
        29, 31, 28, 28
    ],
];



impl GameMap {
    pub fn try_fit_chunk_into(self: &mut Self, width: usize, height: usize) -> bool {
        let new_tile_size = width * height;
        let new_prospective_size = self.num_tiles + new_tile_size;
        if new_prospective_size <= MAX_N_TILES_IN_WHOLE_MAP {
            // self.num_tiles = new_prospective_size;
            // crate::trace(self.num_tiles.to_string());
            return true;
        }
        false
    }

    pub fn link_chunk_to_touching_chunks(self: &mut Self, chunk: &mut MapChunk) {
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
                    for absolute_coord_x in min_x..max_x {
                        let rel_chunk_x = absolute_coord_x - chunk.bound.x;
                        let rel_other_chunk_x = absolute_coord_x - other_chunk.bound.x;
                        if rel_chunk_x > 0 && rel_chunk_x < chunk.bound.width as i32 - 1 {
                            chunk.set_tile(rel_chunk_x as usize, 0 as usize, 0);
                        }  
                        if rel_other_chunk_x > 0 && rel_other_chunk_x < other_chunk.bound.width as i32 - 1 {
                            other_chunk.set_tile(rel_other_chunk_x as usize, other_chunk.bound.height as usize - 1, 0)
                        }
                        if rel_other_chunk_x == 0 || rel_other_chunk_x == other_chunk.bound.width as i32 - 1 || rel_chunk_x == 0 || rel_chunk_x == chunk.bound.width as i32 - 1 {
                            chunk.set_tile(rel_chunk_x as usize, 0 as usize, 9);
                            other_chunk.set_tile(rel_other_chunk_x as usize, other_chunk.bound.height as usize - 1, 9)
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
                    for absolute_coord_y in min_y..max_y {
                        let rel_chunk_y = absolute_coord_y - chunk.bound.y;
                        let rel_other_chunk_y = absolute_coord_y - other_chunk.bound.y;
                        if rel_chunk_y > 0 && rel_chunk_y < chunk.bound.height as i32 - 1 {
                            chunk.set_tile(0, rel_chunk_y as usize, 0);
                        }
                        if rel_other_chunk_y > 0 && rel_other_chunk_y < other_chunk.bound.height as i32 - 1 {
                            other_chunk.set_tile(other_chunk.bound.width as usize - 1, rel_other_chunk_y as usize, 0)
                        }
                        if rel_other_chunk_y == 0 || rel_other_chunk_y == other_chunk.bound.height as i32 - 1 || rel_chunk_y == 0 || rel_chunk_y == chunk.bound.height as i32 - 1 {
                            chunk.set_tile(0, rel_chunk_y as usize, 9);
                            other_chunk.set_tile(other_chunk.bound.width as usize - 1, rel_other_chunk_y as usize, 9)
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

    pub fn add_chunk(self: & mut Self, mut chunk: MapChunk) {
        self.link_chunk_to_touching_chunks(&mut chunk);
        self.chunks.push(chunk);
    }

    pub fn create_map() -> GameMap {
        let chunks: Vec<MapChunk> = Vec::new();
    
        let map = GameMap { 
            chunks: chunks,
            num_tiles: 0,
        };
    
    
        map
    }
}
