use glam::{IVec3};

#[derive(Default)]
pub struct SpatialOctreeNode{
    is_leaf: bool,
    depth: u32,
    center: IVec3,
    half_size: i32,
}

impl SpatialOctreeNode{
    fn center(mut self,center:IVec3) -> Self{ self.center = center; self }
    fn half_size(mut self,half_size:i32) -> Self{ self.half_size = half_size; self }
}

pub struct SpatialOctree {
    max_depth: u32,
    root:SpatialOctreeNode
}

impl SpatialOctree {
    pub fn new(
        max_depth: u32,
        center: IVec3,
        half_size: i32
    ) -> Self {
        let root = SpatialOctreeNode::default()
            .center(center)
            .half_size(half_size);
        Self{
            max_depth,
            root
        }
    }

    pub fn add(&mut self, x: i32, y: i32) {

    }

    fn get_child_index(pos:IVec3, center: IVec3) -> usize{
        let mut index = 0;
        if center.x < pos.x {index |= 1}
        if center.y < pos.y {index |= 2}
        if center.z < pos.z {index |= 4}
        index
    }
}