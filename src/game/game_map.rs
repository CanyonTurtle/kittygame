use super::{mapchunk::MapChunk, game_constants::TOTAL_TILES_IN_MAP};

pub struct GameMap {
    pub chunks: Vec<MapChunk>,
    pub num_tiles: usize
}

impl GameMap {
    pub fn try_fit_chunk_into(self: &mut Self, width: usize, height: usize) -> bool {
        let new_tile_size = width * height;
        let new_prospective_size = self.num_tiles + new_tile_size;
        if new_prospective_size <= TOTAL_TILES_IN_MAP {
            self.num_tiles = new_prospective_size;
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
                            chunk.set_tile(rel_chunk_x as usize, 0 as usize, 6);
                            other_chunk.set_tile(rel_other_chunk_x as usize, other_chunk.bound.height as usize - 1, 6)
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
                            chunk.set_tile(0, rel_chunk_y as usize, 6);
                            other_chunk.set_tile(other_chunk.bound.width as usize - 1, rel_other_chunk_y as usize, 6)
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
