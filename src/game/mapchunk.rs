use super::game_constants::{TILE_HEIGHT_PX, TILE_WIDTH_PX};

pub struct TileAlignedBoundingBox {
    pub x: i32,
    pub y: i32,
    pub width: usize,
    pub height: usize,
}

impl TileAlignedBoundingBox {
    pub fn init(x: i32, y: i32, w: usize, h: usize) -> Self {
        TileAlignedBoundingBox {
            x,
            y,
            width: w,
            height: h,
        }
    }
}

pub struct MapChunk {
    pub tiles: Vec<u8>,
    pub bound: TileAlignedBoundingBox,
}

pub enum OutOfChunkBound {
    Out,
}

impl MapChunk {
    pub fn init() -> Self {
        MapChunk {
            tiles: Vec::new(),
            bound: TileAlignedBoundingBox {
                y: 1,
                x: 1,
                width: 1,
                height: 1,
            },
        }
    }

    pub fn clamp_coords(&self, x: usize, y: usize) -> (usize, usize) {
        let clamped_x = num::clamp(x, 0, self.bound.width - 1);
        let clamped_y = num::clamp(y, 0, self.bound.height - 1);
        (clamped_x, clamped_y)
    }
    pub fn set_tile(&mut self, x: usize, y: usize, val: u8) {
        let clamped_coords = self.clamp_coords(x, y);

        let logical_idx = clamped_coords.1 * self.bound.width + clamped_coords.0;
        let actual_idx = logical_idx / 2;
        // crate::trace(format!["l: {}", logical_idx]);
        // crate::trace(format!["a: {}", actual_idx]);
        // crate::trace(format!["len: {}", self.tiles.len()]);
        // even tiles will be in the lower 4 bits
        // odd tiles will be the upper 4 bits
        let mut prior;
        {
            prior = self.tiles[actual_idx];
        }

        if logical_idx.is_multiple_of(2) {
            prior &= 0xf0;
            prior |= val & 0x0f;
            self.tiles[actual_idx] = prior;
            // crate::trace("e1");
        } else {
            prior &= 0x0f;
            prior |= (((val as u32) << 4) & 0xf0) as u8;
            self.tiles[actual_idx] = prior;
            // crate::trace("e2");
        }
        // crate::trace("set tile inside");
    }

    pub fn get_tile(&self, x: usize, y: usize) -> u8 {
        let clamped_coords = self.clamp_coords(x, y);

        let logical_idx = clamped_coords.1 * self.bound.width + clamped_coords.0;
        let actual_idx: usize = logical_idx / 2;

        // even tiles will be in the lower 4 bits
        // odd tiles will be the upper 4 bits
        let current;
        {
            current = self.tiles[actual_idx];
        }

        if logical_idx.is_multiple_of(2) {
            current & 0x0f
        } else {
            // 0// (current << 4) & 0xf0
            (((current as u32) >> 4) & 0x0f) as u8
        }
    }

    pub fn is_tile_idx_inside_tile_aligned_bound(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.bound.width as i32 && y >= 0 && y < self.bound.height as i32
    }

    pub fn get_tile_abs(&self, abs_x: i32, abs_y: i32) -> Result<u8, OutOfChunkBound> {
        let rel_x =
            ((abs_x - self.bound.x * TILE_WIDTH_PX as i32) as f32 / TILE_WIDTH_PX as f32) as i32;
        let rel_y =
            ((abs_y - self.bound.y * TILE_HEIGHT_PX as i32) as f32 / TILE_HEIGHT_PX as f32) as i32;

        if self.is_tile_idx_inside_tile_aligned_bound(rel_x, rel_y) {
            return Result::Ok(self.get_tile(rel_x as usize, rel_y as usize));
        }
        Result::Err(OutOfChunkBound::Out)
    }

    pub fn initialize(&mut self) -> bool {
        self.tiles.clear();
        let n_bytes_for_chunk_storage = (self.bound.width * self.bound.height) / 2 + 2;
        match self.tiles.try_reserve_exact(n_bytes_for_chunk_storage) {
            Ok(_) => {
                for _ in 0..n_bytes_for_chunk_storage {
                    self.tiles.push(0);
                }
                true
            }
            Err(_) => false,
        }
    }
}
