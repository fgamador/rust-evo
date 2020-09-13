use std::convert::TryInto;
use std::fmt;
use std::fmt::{Error, Formatter};
use std::u32;

#[derive(Debug)]
pub struct NodesWithHandles<N: NodeWithHandle> {
    nodes: Vec<N>,
}

impl<N: NodeWithHandle> NodesWithHandles<N> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        NodesWithHandles { nodes: vec![] }
    }

    pub fn add_node(&mut self, mut node: N) -> NodeHandle {
        let handle = self.next_handle();
        node.handle_mut().index = handle.index;
        self.nodes.push(node);
        handle
    }

    fn next_handle(&self) -> NodeHandle {
        NodeHandle::new(self.nodes.len().try_into().unwrap())
    }

    pub fn is_valid_handle(&self, handle: NodeHandle) -> bool {
        (handle.index as usize) < self.nodes.len()
    }

    /// Removes the nodes referenced by `handles`.
    ///
    /// Warning: this function has two big gotchas:
    ///
    /// 1) `handles` should be in ascending order of `index`. If not, the function will
    /// panic on index out-of-bounds if we're removing nodes at the end of self.nodes.
    ///
    /// 2) Worse, this function changes the nodes referenced by some of the remaining handles.
    /// Never retain handles across a call to this function.
    pub fn remove_nodes(&mut self, handles: &[NodeHandle]) {
        for handle in handles.iter().rev() {
            self.remove_node(*handle);
        }
    }

    /// Warning: invalidates handle to the last node in self.nodes.
    fn remove_node(&mut self, handle: NodeHandle) {
        self.nodes.swap_remove(handle.index());
        if handle != self.next_handle() {
            self.node_mut(handle).handle_mut().index = handle.index;
        }
    }

    pub fn nodes(&self) -> &[N] {
        &self.nodes
    }

    pub fn nodes_mut(&mut self) -> &mut [N] {
        &mut self.nodes
    }

    pub fn node(&self, handle: NodeHandle) -> &N {
        &self.nodes[handle.index()]
    }

    pub fn node_mut(&mut self, handle: NodeHandle) -> &mut N {
        &mut self.nodes[handle.index()]
    }
}

pub trait NodeWithHandle {
    fn handle(&self) -> NodeHandle;

    fn handle_mut(&mut self) -> &mut NodeHandle;
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NodeHandle {
    index: u32,
}

impl NodeHandle {
    fn new(index: u32) -> Self {
        NodeHandle { index }
    }

    pub fn unset() -> Self {
        NodeHandle { index: u32::MAX }
    }

    pub fn resolve<'a, N>(&self, nodes: &'a mut [N]) -> &'a N
    where
        N: NodeWithHandle,
    {
        &nodes[self.index()]
    }

    pub fn resolve_mut<'a, N>(&self, nodes: &'a mut [N]) -> &'a mut N
    where
        N: NodeWithHandle,
    {
        &mut nodes[self.index()]
    }

    fn index(self) -> usize {
        self.index as usize
    }
}

impl fmt::Display for NodeHandle {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn added_node_has_correct_handle() {
        let mut nodes = NodesWithHandles::new();

        let handle = nodes.add_node(SimpleNodeWithHandle::new(0));

        let node = &nodes.nodes()[0];
        assert_eq!(node.handle(), handle);
    }

    #[test]
    fn can_fetch_node_by_handle() {
        let mut nodes = NodesWithHandles::new();

        let node_handle = nodes.add_node(SimpleNodeWithHandle::new(0));

        let node = &nodes.nodes()[0];
        assert_eq!(*nodes.node(node_handle), *node);
    }

    #[test]
    fn can_remove_last_and_non_last_nodes() {
        let mut nodes = NodesWithHandles::new();
        let node0_handle = nodes.add_node(SimpleNodeWithHandle::new(0));
        let _node1_handle = nodes.add_node(SimpleNodeWithHandle::new(1));
        let node2_handle = nodes.add_node(SimpleNodeWithHandle::new(2));

        nodes.remove_nodes(&vec![node0_handle, node2_handle]);

        assert_eq!(nodes.nodes.len(), 1);
        let node = &nodes.nodes()[0];
        assert_eq!(node.id, 1);
        assert_eq!(node.handle().index, 0);
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct SimpleNodeWithHandle {
        handle: NodeHandle,
        pub id: i32,
    }

    impl SimpleNodeWithHandle {
        pub fn new(id: i32) -> Self {
            SimpleNodeWithHandle {
                handle: NodeHandle::unset(),
                id,
            }
        }
    }

    impl NodeWithHandle for SimpleNodeWithHandle {
        fn handle(&self) -> NodeHandle {
            self.handle
        }

        fn handle_mut(&mut self) -> &mut NodeHandle {
            &mut self.handle
        }
    }
}
