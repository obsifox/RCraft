use std::collections::HashMap;
use rminecraft_world::{BlockState, ChunkColumn};

pub struct AABB {
    pub min_x: f64,
    pub min_y: f64,
    pub min_z: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub max_z: f64,
}

impl AABB {
    pub fn new(min_x: f64, min_y: f64, min_z: f64, max_x: f64, max_y: f64, max_z: f64) -> Self {
        Self { min_x, min_y, min_z, max_x, max_y, max_z }
    }

    pub fn player_box(x: f64, y: f64, z: f64) -> Self {
        let half_w = 0.3; // player width 0.6
        let height = 1.8;
        Self {
            min_x: x - half_w,
            min_y: y,
            min_z: z - half_w,
            max_x: x + half_w,
            max_y: y + height,
            max_z: z + half_w,
        }
    }

    pub fn intersects(&self, other: &AABB) -> bool {
        self.min_x < other.max_x && self.max_x > other.min_x &&
        self.min_y < other.max_y && self.max_y > other.min_y &&
        self.min_z < other.max_z && self.max_z > other.min_z
    }
}

pub struct PlayerPhysics {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}

impl PlayerPhysics {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            x,
            y,
            z,
            vx: 0.0,
            vy: 0.0,
            vz: 0.0,
            yaw: 0.0,
            pitch: 0.0,
            on_ground: false,
        }
    }

    pub fn tick(&mut self, world: &HashMap<(i32, i32), ChunkColumn>) {
        // Minecraft gravity: -0.08 per tick, drag: 0.98
        self.vy = (self.vy - 0.08) * 0.98;
        self.vx *= 0.91;
        self.vz *= 0.91;

        // Proposed new position
        let new_y = self.y + self.vy;

        // Simple ground collision check
        let foot_block_y = new_y.floor() as i32;
        let cx = (self.x.floor() as i32) >> 4;
        let cz = (self.z.floor() as i32) >> 4;
        let rx = (self.x.floor() as i32) & 15;
        let rz = (self.z.floor() as i32) & 15;

        if let Some(chunk) = world.get(&(cx, cz)) {
            let block = chunk.get_block(rx, foot_block_y, rz);
            if block.is_opaque() && self.vy < 0.0 {
                // Landed on ground
                self.y = (foot_block_y + 1) as f64;
                self.vy = 0.0;
                self.on_ground = true;
            } else {
                self.y = new_y;
                self.on_ground = false;
            }
        } else {
            self.y = new_y;
        }

        self.x += self.vx;
        self.z += self.vz;
    }

    pub fn jump(&mut self) {
        if self.on_ground {
            self.vy = 0.42; // Minecraft jump velocity
            self.on_ground = false;
        }
    }
}
