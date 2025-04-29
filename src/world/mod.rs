use crate::utility::sparse_spatial_octree::SparseSpatialOctree;
use crate::world::chunk::Chunk;
use glam::{IVec3, Vec3, ivec3};
use std::collections::HashMap;

pub(crate) mod chunk;

pub struct World {
    pub loaded_chunks: HashMap<IVec3, Option<Box<Chunk>>>,
    visible_map: SparseSpatialOctree,
    last_map_center: IVec3,
    last_player_pos: IVec3,
}

impl World {
    pub fn new(radius: i32) -> Self {
        let last_map_center = IVec3::ZERO;
        let last_player_pos = IVec3::ZERO;
        let loaded_chunks: HashMap<IVec3, Option<Box<Chunk>>> = HashMap::new();
        let visible_map = SparseSpatialOctree::new(last_map_center, radius);
        let mut chunk = Self {
            loaded_chunks,
            visible_map,
            last_map_center,
            last_player_pos,
        };
        chunk.initialize_map(radius);
        chunk
    }

    #[inline]
    fn initialize_map(&mut self, radius: i32) {
        for x in -radius..radius + 1 {
            for y in -radius..radius + 1 {
                for z in -radius..radius + 1 {
                    let key = ivec3(x, y, z);
                    let chunk = self.load_chunk(key);
                    if self.visible_map.is_in_sphere(&key) {
                        self.visible_map.add(key, false);
                        self.loaded_chunks.insert(key, chunk);
                    }
                }
            }
        }
    }

    pub fn on_player_moved(&mut self, pos: Vec3) {
        let pos = pos.floor().as_ivec3();
        if self.last_player_pos == pos {
            return;
        };
        self.last_player_pos = pos;
        self.update_map_position(pos)
    }

    pub fn update_map_position(&mut self, new_center: IVec3) {
        if new_center == self.last_map_center {
            return;
        }
        let old_center = self.last_map_center;
        let delta_pos = new_center - old_center;
        if delta_pos.x <= 1
            && delta_pos.x > 0
            && delta_pos.y <= 1
            && delta_pos.y > 0
            && delta_pos.z <= 1
            && delta_pos.z > 0
        {
            return;
        }
        self.visible_map = self.visible_map.copy_base(new_center);
        let mut new_chunk_positions = Vec::new();
        self.loaded_chunks.retain(|key, value| {
            let local_pos = key - old_center;
            if self.visible_map.is_in_sphere(&local_pos) {
                self.visible_map.add(*key, false);
                true
            } else {
                let local_add_pos = -IVec3::new(local_pos.x - 1, local_pos.y - 1, local_pos.z - 1);
                new_chunk_positions.push(local_add_pos);
                false
            }
        });
        for local_pos in new_chunk_positions {
            let world_pos = new_center + local_pos;
            let chunk = self.load_chunk(world_pos);
            if chunk.is_some() {
                self.visible_map.add(local_pos, true);
            }
            self.loaded_chunks.insert(world_pos, chunk);
        }
        self.last_map_center = new_center;
    }

    fn load_chunk(&mut self, pos: IVec3) -> Option<Box<Chunk>> {
        Chunk::new(pos)
    }
}
