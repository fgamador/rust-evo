use std::convert::TryInto;
use std::fmt;
use std::fmt::{Error, Formatter};
use std::u32;

#[derive(Debug)]
pub struct NodeVec<N: Node> {
    nodes: Vec<N>,
}

impl<N: Node> NodeVec<N> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        NodeVec { nodes: vec![] }
    }

    pub fn add_node(&mut self, mut node: N) -> NodeHandle {
        let handle = self.next_handle();
        node.node_data_mut().handle = handle;
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
            self.node_mut(handle).node_data_mut().handle = handle;
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

pub trait Node {
    fn node_handle(&self) -> NodeHandle;

    fn node_data(&self) -> &NodeData;

    fn node_data_mut(&mut self) -> &mut NodeData;
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
        N: Node,
    {
        &nodes[self.index()]
    }

    pub fn resolve_mut<'a, N>(&self, nodes: &'a mut [N]) -> &'a mut N
    where
        N: Node,
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

#[derive(Clone, Debug, PartialEq)]
pub struct NodeData {
    handle: NodeHandle,
}

impl NodeData {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        NodeData {
            handle: NodeHandle::unset(),
        }
    }

    pub fn handle(&self) -> NodeHandle {
        self.handle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn added_node_has_correct_handle() {
        let mut nodes = NodeVec::new();

        let handle = nodes.add_node(SimpleNode::new(0));

        let node = &nodes.nodes()[0];
        assert_eq!(node.node_handle(), handle);
    }

    #[test]
    fn can_fetch_node_by_handle() {
        let mut nodes = NodeVec::new();

        let node_handle = nodes.add_node(SimpleNode::new(0));

        let node = &nodes.nodes()[0];
        assert_eq!(*nodes.node(node_handle), *node);
    }

    #[test]
    fn can_remove_last_and_non_last_nodes() {
        let mut nodes = NodeVec::new();
        let node0_handle = nodes.add_node(SimpleNode::new(0));
        let _node1_handle = nodes.add_node(SimpleNode::new(1));
        let node2_handle = nodes.add_node(SimpleNode::new(2));

        nodes.remove_nodes(&vec![node0_handle, node2_handle]);

        assert_eq!(nodes.nodes.len(), 1);
        let node = &nodes.nodes()[0];
        assert_eq!(node.id, 1);
        assert_eq!(node.node_handle().index, 0);
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct SimpleNode {
        node_data: NodeData,
        pub id: i32,
    }

    impl SimpleNode {
        pub fn new(id: i32) -> Self {
            SimpleNode {
                node_data: NodeData::new(),
                id,
            }
        }
    }

    impl Node for SimpleNode {
        fn node_handle(&self) -> NodeHandle {
            self.node_data.handle
        }

        fn node_data(&self) -> &NodeData {
            &self.node_data
        }

        fn node_data_mut(&mut self) -> &mut NodeData {
            &mut self.node_data
        }
    }
}
