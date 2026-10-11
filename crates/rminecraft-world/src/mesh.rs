use crate::chunk::ChunkColumn;
use crate::block::BlockState;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub light: f32,
}

#[derive(Default, Clone)]
pub struct ChunkMesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

pub struct Mesher;

impl Mesher {
    pub fn build_chunk_mesh(chunk: &ChunkColumn) -> ChunkMesh {
        let mut mesh = ChunkMesh::default();
        let chunk_base_x = (chunk.x * 16) as f32;
        let chunk_base_z = (chunk.z * 16) as f32;

        for s in 0..16 {
            if let Some(ref section) = chunk.sections[s] {
                let section_base_y = (s * 16) as f32;

                for y in 0..16 {
                    for z in 0..16 {
                        for x in 0..16 {
                            let block = section.get_block(x, y, z);
                            if block.is_air() {
                                continue;
                            }

                            let world_x = chunk_base_x + x as f32;
                            let world_y = section_base_y + y as f32;
                            let world_z = chunk_base_z + z as f32;

                            // Check +Y (Top)
                            let top_block = if y < 15 {
                                section.get_block(x, y + 1, z)
                            } else if s < 15 {
                                chunk.get_block(x as i32, (s as i32 + 1) * 16, z as i32)
                            } else {
                                BlockState::AIR
                            };
                            if !top_block.is_opaque() {
                                Self::add_face(&mut mesh, world_x, world_y, world_z, Face::Top, 1.0);
                            }

                            // Check -Y (Bottom)
                            let bottom_block = if y > 0 {
                                section.get_block(x, y - 1, z)
                            } else if s > 0 {
                                chunk.get_block(x as i32, s as i32 * 16 - 1, z as i32)
                            } else {
                                BlockState::BEDROCK // Bedrock bottom
                            };
                            if !bottom_block.is_opaque() {
                                Self::add_face(&mut mesh, world_x, world_y, world_z, Face::Bottom, 0.5);
                            }

                            // Check +X (East)
                            let east_block = if x < 15 {
                                section.get_block(x + 1, y, z)
                            } else {
                                BlockState::AIR // Simplified boundary
                            };
                            if !east_block.is_opaque() {
                                Self::add_face(&mut mesh, world_x, world_y, world_z, Face::East, 0.7);
                            }

                            // Check -X (West)
                            let west_block = if x > 0 {
                                section.get_block(x - 1, y, z)
                            } else {
                                BlockState::AIR
                            };
                            if !west_block.is_opaque() {
                                Self::add_face(&mut mesh, world_x, world_y, world_z, Face::West, 0.7);
                            }

                            // Check +Z (South)
                            let south_block = if z < 15 {
                                section.get_block(x, y, z + 1)
                            } else {
                                BlockState::AIR
                            };
                            if !south_block.is_opaque() {
                                Self::add_face(&mut mesh, world_x, world_y, world_z, Face::South, 0.85);
                            }

                            // Check -Z (North)
                            let north_block = if z > 0 {
                                section.get_block(x, y, z - 1)
                            } else {
                                BlockState::AIR
                            };
                            if !north_block.is_opaque() {
                                Self::add_face(&mut mesh, world_x, world_y, world_z, Face::North, 0.85);
                            }
                        }
                    }
                }
            }
        }

        mesh
    }

    fn add_face(mesh: &mut ChunkMesh, x: f32, y: f32, z: f32, face: Face, light: f32) {
        let base_idx = mesh.vertices.len() as u32;

        let (positions, normal) = match face {
            Face::Top => (
                [
                    [x, y + 1.0, z],
                    [x, y + 1.0, z + 1.0],
                    [x + 1.0, y + 1.0, z + 1.0],
                    [x + 1.0, y + 1.0, z],
                ],
                [0.0, 1.0, 0.0],
            ),
            Face::Bottom => (
                [
                    [x, y, z + 1.0],
                    [x, y, z],
                    [x + 1.0, y, z],
                    [x + 1.0, y, z + 1.0],
                ],
                [0.0, -1.0, 0.0],
            ),
            Face::North => (
                [
                    [x + 1.0, y + 1.0, z],
                    [x + 1.0, y, z],
                    [x, y, z],
                    [x, y + 1.0, z],
                ],
                [0.0, 0.0, -1.0],
            ),
            Face::South => (
                [
                    [x, y + 1.0, z + 1.0],
                    [x, y, z + 1.0],
                    [x + 1.0, y, z + 1.0],
                    [x + 1.0, y + 1.0, z + 1.0],
                ],
                [0.0, 0.0, 1.0],
            ),
            Face::West => (
                [
                    [x, y + 1.0, z],
                    [x, y, z],
                    [x, y, z + 1.0],
                    [x, y + 1.0, z + 1.0],
                ],
                [-1.0, 0.0, 0.0],
            ),
            Face::East => (
                [
                    [x + 1.0, y + 1.0, z + 1.0],
                    [x + 1.0, y, z + 1.0],
                    [x + 1.0, y, z],
                    [x + 1.0, y + 1.0, z],
                ],
                [1.0, 0.0, 0.0],
            ),
        };

        let uvs = [
            [0.0, 0.0],
            [0.0, 1.0],
            [1.0, 1.0],
            [1.0, 0.0],
        ];

        for i in 0..4 {
            mesh.vertices.push(Vertex {
                position: positions[i],
                normal,
                uv: uvs[i],
                light,
            });
        }

        // Two triangles for the quad (0, 1, 2) and (0, 2, 3)
        mesh.indices.push(base_idx);
        mesh.indices.push(base_idx + 1);
        mesh.indices.push(base_idx + 2);
        mesh.indices.push(base_idx);
        mesh.indices.push(base_idx + 2);
        mesh.indices.push(base_idx + 3);
    }
}

enum Face {
    Top,
    Bottom,
    North,
    South,
    West,
    East,
}
