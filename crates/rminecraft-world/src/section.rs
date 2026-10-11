use crate::block::BlockState;

pub const SECTION_SIZE: usize = 16;
pub const BLOCKS_PER_SECTION: usize = SECTION_SIZE * SECTION_SIZE * SECTION_SIZE; // 4096

#[derive(Clone)]
pub struct ChunkSection {
    pub blocks: [BlockState; BLOCKS_PER_SECTION],
    pub block_light: [u8; 2048],
    pub sky_light: [u8; 2048],
}

impl Default for ChunkSection {
    fn default() -> Self {
        Self {
            blocks: [BlockState::AIR; BLOCKS_PER_SECTION],
            block_light: [0; 2048],
            sky_light: [255; 2048],
        }
    }
}

impl ChunkSection {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn index(x: usize, y: usize, z: usize) -> usize {
        (y * SECTION_SIZE + z) * SECTION_SIZE + x
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> BlockState {
        if x < 16 && y < 16 && z < 16 {
            self.blocks[Self::index(x, y, z)]
        } else {
            BlockState::AIR
        }
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, state: BlockState) {
        if x < 16 && y < 16 && z < 16 {
            let idx = Self::index(x, y, z);
            self.blocks[idx] = state;
        }
    }
}
