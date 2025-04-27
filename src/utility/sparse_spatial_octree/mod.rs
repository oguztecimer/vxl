mod tests;
use crate::world::World;
use glam::IVec3;

const MAX_RADIUS: i32 = 512;

pub struct SparseSpatialOctreeNode {
    center: IVec3,
    half_extent: i32,
    children: Option<[Option<Box<SparseSpatialOctreeNode>>; 8]>,
    child_count: usize,
}

impl SparseSpatialOctreeNode {
    pub fn new(center: IVec3, half_extent: i32) -> Self {
        Self {
            center,
            half_extent,
            children: None,
            child_count: 0,
        }
    }
}

pub struct SparseSpatialOctree {
    root: SparseSpatialOctreeNode,
    center: IVec3,
    radius_sqr: i32,
}

impl SparseSpatialOctree {
    pub fn new(center: IVec3, radius: i32) -> Self {
        //depth = log2(radius)+1
        if radius > MAX_RADIUS {
            panic!(
                "Cannot create a tree with radius larger than {}",
                MAX_RADIUS
            );
        }
        if radius <= 0 || (radius & (radius - 1)) > 0 {
            panic!("Radius must be a multiplier of 2");
        }
        let radius_sqr = radius * radius;
        let root = SparseSpatialOctreeNode::new(IVec3::ZERO, radius);
        Self {
            root,
            center,
            radius_sqr,
        }
    }

    pub fn add(&mut self, position: IVec3) {
        let relative_position = position - self.center;
        if relative_position.length_squared() > self.radius_sqr {return;}
        Self::add_recursive(&mut self.root, relative_position);
    }

    pub fn remove(&mut self, position: IVec3) {
        let relative_position = position - self.center;
        if relative_position.length_squared() > self.radius_sqr {return;}
        Self::remove_recursive(&mut self.root, relative_position);
    }

    pub fn exists(&self, position: IVec3) -> bool {
        let relative_position = position - self.center;
        if relative_position.length_squared() > self.radius_sqr {return false;}
        Self::exists_recursive(&self.root, relative_position)
    }

    fn exists_recursive(node: &SparseSpatialOctreeNode, position: IVec3) -> bool {
        if node.half_extent < 1 {
            return true;
        }
        let Some(children) = &node.children else {
            return false;
        };
        let index = Self::get_child_index(position, node.center);
        if let Some(child) = &children[index] {
            Self::exists_recursive(child, position)
        } else {
            false
        }
    }

    fn remove_recursive(node: &mut SparseSpatialOctreeNode, position: IVec3) -> bool {
        if node.half_extent < 1 {
            return true;
        }
        let Some(children) = &mut node.children else {
            return false;
        };
        let index = Self::get_child_index(position, node.center);
        let result = if let Some(child) = &mut children[index] {
            Self::remove_recursive(child, position)
        } else {
            return false;
        };
        if result {
            children[index] = None;
            node.child_count -= 1;
            if node.child_count == 0 {
                node.children = None;
                return true;
            }
        }
        false
    }

    fn add_recursive(node: &mut SparseSpatialOctreeNode, position: IVec3) {
        if node.half_extent < 1 {
            return;
        }
        let index = Self::get_child_index(position, node.center);
        if let Some(children) = &mut node.children {
            if let Some(next_node) = &mut children[index] {
                Self::add_recursive(next_node, position);
            } else {
                let mut new_node = Self::create_new_node(index, &node.center, node.half_extent);
                Self::add_recursive(&mut new_node, position);
                node.child_count += 1;
                children[index] = Some(Box::from(new_node));
            }
        } else {
            let mut children: [Option<Box<SparseSpatialOctreeNode>>; 8] =
                core::array::from_fn(|_| None);
            let mut new_node = Self::create_new_node(index, &node.center, node.half_extent);
            Self::add_recursive(&mut new_node, position);
            node.child_count += 1;
            children[index] = Some(Box::from(new_node));
            node.children = Some(children);
        }
    }

    fn create_new_node(index: usize, center: &IVec3, half_extent: i32) -> SparseSpatialOctreeNode {
        let offset = half_extent / 2;
        let center = IVec3::new(
            center.x + if index & 1 != 0 { offset } else { -offset },
            center.y + if index & 2 != 0 { offset } else { -offset },
            center.z + if index & 4 != 0 { offset } else { -offset },
        );
        SparseSpatialOctreeNode::new(center, offset)
    }

    fn get_child_index(pos: IVec3, center: IVec3) -> usize {
        ((pos.x > center.x) as usize)
            | ((pos.y > center.y) as usize) << 1
            | ((pos.z > center.z) as usize) << 2
    }

    fn recenter(&mut self, new_center: IVec3, world: &mut World) {}
}
