// all credit for this RNG code goes to analog-hors on Github.
// MIT License

// Copyright (c) 2021 analog-hors

// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

pub struct Rng(u128);




impl Rng {
    pub fn new() -> Self {
        Self(0x7369787465656E2062797465206E756Du128 | 1)
    }

    pub fn new_from_seed(n_prefix_draws: u32) -> Self {
        let mut new_rng = Rng::new();
        for _ in 0..n_prefix_draws {
            new_rng.next();
        }
        new_rng
    }

    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(0x2360ED051FC65DA44385DF649FCCF645);
        let rot = (self.0 >> 122) as u32;
        let xsl = ((self.0 >> 64) as u64) ^ (self.0 as u64);
        xsl.rotate_right(rot)     
    }
}

// Allow us to use RNG the same way, regardless of whether its fixed seed or input-based seeding
pub enum GameRng {
    FixedSeed(Rng, Rng),
    Random(Rng)
}

impl GameRng {
    pub fn next_for_worldgen(&mut self) -> u64 {
        match self {
            Self::FixedSeed(fixed_rng, _) => {
                fixed_rng.next()
            },
            Self::Random(rng) => rng.next()
        }
    }

    pub fn next_for_input(&mut self) -> u64 {
        match self {
            Self::FixedSeed(_, input_rng) => {
                input_rng.next()
            },
            Self::Random(rng) => rng.next()
        }
    }
}