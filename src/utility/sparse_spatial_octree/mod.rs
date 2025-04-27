mod tests;
use glam::IVec3;

const MAX_DEPTH: u32 = 10;

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
    center: IVec3
}

impl SparseSpatialOctree {
    pub fn new(center: IVec3, capacity: i32) -> Self {
        if capacity <= 0 {
            panic!("Can not create a tree with non-positive capacity.");
        }
        if capacity > 8i32.pow(MAX_DEPTH) {
            panic!(
                "Cannot create a tree with capacity larger than 8^MAX_DEPTH({})",
                MAX_DEPTH
            );
        }
        let mut half_extent = 0;
        for i in 0..MAX_DEPTH {
            let real_capacity = 8i32.pow(i);
            if real_capacity >= capacity {
                half_extent = if i == 0 { 0 } else { 1 << (i - 1) };
                break;
            }
        }
        let root = SparseSpatialOctreeNode::new(IVec3::ZERO, half_extent);
        Self { root,center }
    }

    pub fn add(&mut self, position: IVec3) {
        let relative_position = position - self.center;
        Self::add_recursive(&mut self.root, relative_position);
    }

    pub fn remove(&mut self, position: IVec3) {
        let relative_position = position - self.center;
        Self::remove_recursive(&mut self.root, relative_position);
    }

    pub fn exists(&self, position: IVec3) -> bool {
        let relative_position = position - self.center;
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
}
