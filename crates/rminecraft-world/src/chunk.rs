use std::io::{self, Cursor, Read};
use byteorder::{BigEndian, ReadBytesExt};
use rminecraft_protocol::types::read_varint;
use crate::section::ChunkSection;
use crate::block::BlockState;

pub const SECTIONS_PER_CHUNK: usize = 16;

pub struct ChunkColumn {
    pub x: i32,
    pub z: i32,
    pub sections: [Option<ChunkSection>; SECTIONS_PER_CHUNK],
    pub biomes: [u8; 256],
}

impl ChunkColumn {
    pub fn new(x: i32, z: i32) -> Self {
        Self {
            x,
            z,
            sections: Default::default(),
            biomes: [1; 256], // Default Plains (1)
        }
    }

    pub fn get_block(&self, rel_x: i32, y: i32, rel_z: i32) -> BlockState {
        if rel_x < 0 || rel_x >= 16 || rel_z < 0 || rel_z >= 16 || y < 0 || y >= 256 {
            return BlockState::AIR;
        }
        let section_idx = (y / 16) as usize;
        let local_y = (y % 16) as usize;
        if let Some(ref section) = self.sections[section_idx] {
            section.get_block(rel_x as usize, local_y, rel_z as usize)
        } else {
            BlockState::AIR
        }
    }

    pub fn set_block(&mut self, rel_x: i32, y: i32, rel_z: i32, block: BlockState) {
        if rel_x < 0 || rel_x >= 16 || rel_z < 0 || rel_z >= 16 || y < 0 || y >= 256 {
            return;
        }
        let section_idx = (y / 16) as usize;
        let local_y = (y % 16) as usize;
        if self.sections[section_idx].is_none() {
            self.sections[section_idx] = Some(ChunkSection::new());
        }
        if let Some(ref mut section) = self.sections[section_idx] {
            section.set_block(rel_x as usize, local_y, rel_z as usize, block);
        }
    }

    /// Parses 1.12.2 Chunk Data packet payload into ChunkColumn
    pub fn parse_1_12_2(
        chunk_x: i32,
        chunk_z: i32,
        full_chunk: bool,
        primary_bit_mask: i32,
        data: &[u8],
        has_sky_light: bool,
    ) -> io::Result<Self> {
        let mut column = ChunkColumn::new(chunk_x, chunk_z);
        let mut cursor = Cursor::new(data);

        for s in 0..SECTIONS_PER_CHUNK {
            if (primary_bit_mask & (1 << s)) != 0 {
                let mut section = ChunkSection::new();
                let bits_per_block = cursor.read_u8()?;
                
                let palette: Option<Vec<i32>> = if bits_per_block <= 8 {
                    let palette_len = read_varint(&mut cursor)?;
                    let mut pal = Vec::with_capacity(palette_len as usize);
                    for _ in 0..palette_len {
                        pal.push(read_varint(&mut cursor)?);
                    }
                    Some(pal)
                } else {
                    None
                };

                let data_array_len = read_varint(&mut cursor)? as usize;
                let mut data_array = Vec::with_capacity(data_array_len);
                for _ in 0..data_array_len {
                    data_array.push(cursor.read_u64::<BigEndian>()?);
                }

                // Unpack block states from 64-bit longs
                let mask = (1u64 << bits_per_block) - 1;
                let blocks_per_long = 64 / (bits_per_block as usize);
                let mut block_idx = 0;

                for long_val in data_array {
                    let mut current = long_val;
                    for _ in 0..blocks_per_long {
                        if block_idx >= 4096 {
                            break;
                        }
                        let raw_id = (current & mask) as usize;
                        let state_id = if let Some(ref pal) = palette {
                            if raw_id < pal.len() { pal[raw_id] as u16 } else { 0 }
                        } else {
                            raw_id as u16
                        };
                        section.blocks[block_idx] = BlockState::from_raw(state_id);
                        current >>= bits_per_block;
                        block_idx += 1;
                    }
                }

                // Block light (2048 bytes)
                cursor.read_exact(&mut section.block_light)?;

                // Sky light (2048 bytes) if overworld
                if has_sky_light {
                    cursor.read_exact(&mut section.sky_light)?;
                }

                column.sections[s] = Some(section);
            }
        }

        // Biome array (256 bytes) if full chunk
        if full_chunk && (cursor.position() as usize + 256 <= data.len()) {
            cursor.read_exact(&mut column.biomes)?;
        }

        Ok(column)
    }
}
