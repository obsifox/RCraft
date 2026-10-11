#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BlockState {
    pub id: u16,
    pub meta: u8,
}

impl BlockState {
    pub const AIR: Self = Self { id: 0, meta: 0 };
    pub const STONE: Self = Self { id: 1, meta: 0 };
    pub const GRASS: Self = Self { id: 2, meta: 0 };
    pub const DIRT: Self = Self { id: 3, meta: 0 };
    pub const COBBLESTONE: Self = Self { id: 4, meta: 0 };
    pub const PLANKS_OAK: Self = Self { id: 5, meta: 0 };
    pub const BEDROCK: Self = Self { id: 7, meta: 0 };
    pub const WATER: Self = Self { id: 9, meta: 0 };
    pub const LAVA: Self = Self { id: 11, meta: 0 };
    pub const SAND: Self = Self { id: 12, meta: 0 };
    pub const GRAVEL: Self = Self { id: 13, meta: 0 };
    pub const GOLD_ORE: Self = Self { id: 14, meta: 0 };
    pub const IRON_ORE: Self = Self { id: 15, meta: 0 };
    pub const COAL_ORE: Self = Self { id: 16, meta: 0 };
    pub const LOG_OAK: Self = Self { id: 17, meta: 0 };
    pub const LEAVES_OAK: Self = Self { id: 18, meta: 0 };
    pub const GLASS: Self = Self { id: 20, meta: 0 };
    pub const DIAMOND_ORE: Self = Self { id: 56, meta: 0 };
    pub const DIAMOND_BLOCK: Self = Self { id: 57, meta: 0 };

    pub fn from_raw(raw: u16) -> Self {
        Self {
            id: raw >> 4,
            meta: (raw & 0xF) as u8,
        }
    }

    pub fn to_raw(&self) -> u16 {
        (self.id << 4) | (self.meta as u16 & 0xF)
    }

    pub fn is_air(&self) -> bool {
        self.id == 0
    }

    pub fn is_opaque(&self) -> bool {
        match self.id {
            0 => false,  // Air
            8 | 9 => false, // Water
            10 | 11 => false, // Lava
            18 => false, // Leaves
            20 => false, // Glass
            _ => true,
        }
    }

    pub fn texture_name_top(&self) -> &'static str {
        match self.id {
            1 => "stone",
            2 => "grass_top",
            3 => "dirt",
            4 => "cobblestone",
            5 => "planks_oak",
            7 => "bedrock",
            12 => "sand",
            13 => "gravel",
            14 => "gold_ore",
            15 => "iron_ore",
            16 => "coal_ore",
            17 => "log_oak_top",
            18 => "leaves_oak",
            20 => "glass",
            56 => "diamond_ore",
            57 => "diamond_block",
            _ => "dirt",
        }
    }

    pub fn texture_name_side(&self) -> &'static str {
        match self.id {
            1 => "stone",
            2 => "grass_side",
            3 => "dirt",
            4 => "cobblestone",
            5 => "planks_oak",
            7 => "bedrock",
            12 => "sand",
            13 => "gravel",
            14 => "gold_ore",
            15 => "iron_ore",
            16 => "coal_ore",
            17 => "log_oak",
            18 => "leaves_oak",
            20 => "glass",
            56 => "diamond_ore",
            57 => "diamond_block",
            _ => "dirt",
        }
    }

    pub fn texture_name_bottom(&self) -> &'static str {
        match self.id {
            2 => "dirt",
            17 => "log_oak_top",
            _ => self.texture_name_side(),
        }
    }
}
