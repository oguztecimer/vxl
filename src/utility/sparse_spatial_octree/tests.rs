#[cfg(test)]
mod tests {
    use super::*;
    use crate::utility::sparse_spatial_octree::SpatialOctree;
    use glam::IVec3;

    // Helper function to create a default octree for tests
    fn create_test_octree() -> SpatialOctree<i32> {
        SpatialOctree::new(IVec3::new(0, 0, 0), 4096)
    }

    #[test]
    fn test_new_octree_half_extent() {
        let octree = SpatialOctree::<i32>::new(IVec3::ZERO, 1);
        assert_eq!(octree.root.half_extent, 0); // Leaf node

        let octree = SpatialOctree::<i32>::new(IVec3::ZERO, 8);
        assert_eq!(octree.root.half_extent, 1); // 8^1 -> half_extent = 2^0

        let octree = SpatialOctree::<i32>::new(IVec3::ZERO, 64);
        assert_eq!(octree.root.half_extent, 2); // 8^2 -> half_extent = 2^1
    }

    #[test]
    #[should_panic]
    fn test_new_octree_invalid_capacity() {
        SpatialOctree::<i32>::new(IVec3::ZERO, 0);
    }

    #[test]
    fn test_new_octree() {
        let octree = create_test_octree();
        assert_eq!(octree.root.center, IVec3::new(0, 0, 0));
        assert_eq!(octree.root.half_extent, 8);
        assert!(octree.root.data.is_none());
        assert!(octree.root.children.is_none());
        assert_eq!(octree.root.child_count, 0);
    }

    #[test]
    fn test_add_single_item() {
        let mut octree = create_test_octree();
        let position = IVec3::new(2, 2, 2);
        octree.add(42, position);

        // Check that children were created
        assert!(octree.root.children.is_some());
        assert_eq!(octree.root.child_count, 1);

        // Navigate to the node containing the item
        let children = octree.root.children.unwrap();
        let child_index = SpatialOctree::<i32>::get_child_index(position, octree.root.center);
        let child = children[child_index].as_ref().unwrap();

        // Check child node properties
        assert_eq!(child.center, IVec3::new(4, 4, 4));
        assert_eq!(child.half_extent, 4);
        assert!(child.data.is_none());
        assert!(child.children.is_some());

        // Further navigate to the leaf node
        let children = child.children.as_ref().unwrap();
        let child_index = SpatialOctree::<i32>::get_child_index(position, child.center);
        let child = children[child_index].as_ref().unwrap();

        //assert_eq!(child.center, IVec3::new(4, 4, 4));
        assert_eq!(child.half_extent, 2);
        assert!(child.data.is_none());
        assert!(child.children.is_some());

        // Further navigate to the leaf node
        let children = child.children.as_ref().unwrap();
        let child_index = SpatialOctree::<i32>::get_child_index(position, child.center);
        let child = children[child_index].as_ref().unwrap();

        //assert_eq!(child.center, IVec3::new(4, 4, 4));
        assert_eq!(child.half_extent, 1);
        assert!(child.data.is_none());
        assert!(child.children.is_some());

        // Further navigate to the leaf node
        let children = child.children.as_ref().unwrap();
        let child_index = SpatialOctree::<i32>::get_child_index(position, child.center);
        let child = children[child_index].as_ref().unwrap();

        //assert_eq!(child.center, IVec3::new(4, 4, 4));
        assert_eq!(child.half_extent, 0);
        assert!(child.data.is_some());
        assert!(child.children.is_none());

        // Check leaf node contains the item
        assert_eq!(child.data, Some(42));
    }

    #[test]
    fn test_add_multiple_items() {
        let mut octree = create_test_octree();
        let positions = [
            IVec3::new(2, 2, 2),
            IVec3::new(-2, -2, -2),
            IVec3::new(3, 1, 2),
        ];

        for (i, &pos) in positions.iter().enumerate() {
            octree.add(i as i32, pos);
        }

        assert!(octree.root.children.is_some());
        assert_eq!(octree.root.child_count, 2); // Should have two child nodes

        // Verify each item exists
        let children = octree.root.children.as_ref().unwrap();
        for (i, &pos) in positions.iter().enumerate() {
            let mut current_node = &octree.root;
            let mut current_pos = pos;

            // Navigate to leaf node
            while current_node.children.is_some() {
                let index = SpatialOctree::<i32>::get_child_index(current_pos, current_node.center);
                current_node = current_node.children.as_ref().unwrap()[index]
                    .as_ref()
                    .unwrap();
                current_pos = pos;
            }

            assert_eq!(current_node.data, Some(i as i32));
        }
    }

    #[test]
    fn test_remove_item() {
        let mut octree = create_test_octree();
        let position = IVec3::new(2, 2, 2);

        // Add and remove item
        octree.add(42, position);
        octree.remove(position);

        // Check that the octree is empty
        assert!(octree.root.children.is_none());
        assert_eq!(octree.root.child_count, 0);
        assert!(octree.root.data.is_none());
    }

    #[test]
    fn test_remove_nonexistent_item() {
        let mut octree = create_test_octree();
        let position = IVec3::new(2, 2, 2);

        // Try to remove from empty octree
        octree.remove(position);
        assert!(octree.root.children.is_none());
        assert_eq!(octree.root.child_count, 0);

        // Add item, then try to remove from wrong position
        octree.add(42, position);
        octree.remove(IVec3::new(-2, -2, -2));

        // Verify item still exists
        assert!(octree.root.children.is_some());
        assert_eq!(octree.root.child_count, 1);
    }

    #[test]
    fn test_child_index_calculation() {
        let center = IVec3::new(0, 0, 0);
        let positions = [
            (IVec3::new(1, 1, 1), 7),    // +x,+y,+z
            (IVec3::new(-1, -1, -1), 0), // -x,-y,-z
            (IVec3::new(1, -1, 1), 5),   // +x,-y,+z
            (IVec3::new(-1, 1, -1), 2),  // -x,+y,-z
        ];

        for (pos, expected_index) in positions {
            let index = SpatialOctree::<i32>::get_child_index(pos, center);
            assert_eq!(index, expected_index);
        }
    }

    #[test]
    fn test_create_new_node() {
        let center = IVec3::new(0, 0, 0);
        let half_extent = 8;

        // Test creating node in index 7 (+x,+y,+z)
        let new_node = SpatialOctree::<i32>::create_new_node(7, &center, half_extent);
        assert_eq!(new_node.center, IVec3::new(4, 4, 4));
        assert_eq!(new_node.half_extent, 4);

        // Test creating node in index 0 (-x,-y,-z)
        let new_node = SpatialOctree::<i32>::create_new_node(0, &center, half_extent);
        assert_eq!(new_node.center, IVec3::new(-4, -4, -4));
        assert_eq!(new_node.half_extent, 4);
    }

    #[test]
    fn test_add_at_minimum_half_extent() {
        let mut octree = SpatialOctree::new(IVec3::new(0, 0, 0), 1);
        let position = IVec3::new(0, 0, 0);

        octree.add(42, position);
        assert_eq!(octree.root.data, Some(42));
        assert!(octree.root.children.is_none());
        assert_eq!(octree.root.child_count, 0);
    }

    #[test]
    fn test_get() {
        let mut octree: SpatialOctree<i32> = SpatialOctree::new(IVec3::ZERO, 8);

        // Test empty octree
        assert_eq!(octree.get(IVec3::new(1, 1, 1)), None);

        // Test adding and retrieving an item
        octree.add(42, IVec3::new(1, 1, 1));
        assert_eq!(octree.get(IVec3::new(1, 1, 1)), Some(&42));
        assert_eq!(octree.get(IVec3::new(0, 0, 0)), None);

        // Test adding another item
        octree.add(99, IVec3::new(-1, -1, -1));
        assert_eq!(octree.get(IVec3::new(-1, -1, -1)), Some(&99));

        // Test removing an item
        octree.remove(IVec3::new(1, 1, 1));
        assert_eq!(octree.get(IVec3::new(1, 1, 1)), None);
        assert_eq!(octree.get(IVec3::new(-1, -1, -1)), Some(&99));
    }
    #[test]
    fn test_add_child_count() {
        let mut octree = SpatialOctree::new(IVec3::ZERO, 8);
        octree.add(42, IVec3::new(1, 1, 1));
        octree.add(43, IVec3::new(-1, -1, -1));
        // Verify child_count is correct at root
        assert_eq!(octree.root.child_count, 2);
    }
}
