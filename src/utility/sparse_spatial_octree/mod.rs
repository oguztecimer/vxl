mod tests;
use glam::IVec3;

const MAX_DEPTH: u32 = 10;

pub struct SparseSpatialOctreeNode<T> {
    center: IVec3,
    half_extent: i32,
    data: Option<T>,
    children: Option<Box<[Option<SparseSpatialOctreeNode<T>>; 8]>>,
    child_count: usize,
}

impl<T> SparseSpatialOctreeNode<T> {
    pub fn new(center: IVec3, half_extent: i32) -> Self {
        Self {
            center,
            half_extent,
            data: None,
            children: None,
            child_count: 0,
        }
    }
}

pub struct SparseSpatialOctree<T> {
    root: SparseSpatialOctreeNode<T>,
}

impl<T> SparseSpatialOctree<T> {
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
        let root = SparseSpatialOctreeNode::new(center, half_extent);
        Self { root }
    }

    pub fn add(&mut self, item: T, position: IVec3) {
        Self::add_recursive(&mut self.root, item, position);
    }

    pub fn remove(&mut self, position: IVec3) {
        Self::remove_recursive(&mut self.root, position);
    }

    pub fn get(&self, position: IVec3) -> Option<&T> {
        Self::get_recursive(&self.root, position)
    }

    fn get_recursive(node: &SparseSpatialOctreeNode<T>, position: IVec3) -> Option<&T> {
        if node.half_extent < 1 {
            return node.data.as_ref();
        }
        let Some(children) = &node.children else {
            return None;
        };
        let index = Self::get_child_index(position, node.center);
        if let Some(child) = &children[index] {
            Self::get_recursive(child, position)
        } else {
            None
        }
    }

    fn remove_recursive(node: &mut SparseSpatialOctreeNode<T>, position: IVec3) -> bool {
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

    fn add_recursive(node: &mut SparseSpatialOctreeNode<T>, item: T, position: IVec3) {
        if node.half_extent < 1 {
            node.data = Some(item);
            return;
        }
        let index = Self::get_child_index(position, node.center);
        if let Some(children) = &mut node.children {
            if let Some(next_node) = &mut children[index] {
                Self::add_recursive(next_node, item, position);
            } else {
                let mut new_node = Self::create_new_node(index, &node.center, node.half_extent);
                Self::add_recursive(&mut new_node, item, position);
                node.child_count += 1;
                children[index] = Some(new_node);
            }
        } else {
            let mut children: [Option<SparseSpatialOctreeNode<T>>; 8] = core::array::from_fn(|_| None);
            let mut new_node = Self::create_new_node(index, &node.center, node.half_extent);
            Self::add_recursive(&mut new_node, item, position);
            node.child_count += 1;
            children[index] = Some(new_node);
            node.children = Some(Box::from(children));
        }
    }

    fn create_new_node(index: usize, center: &IVec3, half_extent: i32) -> SparseSpatialOctreeNode<T> {
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
