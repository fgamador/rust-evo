use std::cmp::Ordering;
use std::convert::TryInto;
use std::fmt;
use std::fmt::{Error, Formatter};
use std::marker::PhantomData;
use std::u32;

#[derive(Debug)]
pub struct NodesWithHandles<N: WithHandle<N>> {
    nodes: Vec<N>,
}

impl<N: WithHandle<N>> NodesWithHandles<N> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        NodesWithHandles { nodes: vec![] }
    }

    pub fn add_node(&mut self, mut node: N) -> Handle<N> {
        let handle = self.next_handle();
        *node.handle_mut() = handle;
        self.nodes.push(node);
        handle
    }

    fn next_handle(&self) -> Handle<N> {
        Handle::new(self.nodes.len().try_into().unwrap())
    }

    pub fn is_valid_handle(&self, handle: Handle<N>) -> bool {
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
    pub fn remove_nodes<F>(&mut self, handles: &[Handle<N>], mut on_handle_change: F)
    where
        F: FnMut(&N, Handle<N>),
    {
        for &handle in handles.iter().rev() {
            self.remove_node(handle, &mut on_handle_change);
        }
    }

    /// Warning: invalidates handles to the last node in self.nodes.
    fn remove_node<F>(&mut self, handle: Handle<N>, on_handle_change: &mut F)
    where
        F: FnMut(&N, Handle<N>),
    {
        self.nodes.swap_remove(handle.index());
        if self.is_valid_handle(handle) {
            *self.node_mut(handle).handle_mut() = handle;
            on_handle_change(self.node(handle), self.next_handle());
        }
    }

    pub fn with_nodes<F>(&mut self, handle1: Handle<N>, handle2: Handle<N>, mut f: F)
    where
        F: FnMut(&mut N, &mut N),
    {
        let node1;
        let node2;
        if handle1.index() < handle2.index() {
            let slices = self.nodes.split_at_mut(handle2.index());
            node1 = &mut slices.0[handle1.index()];
            node2 = &mut slices.1[0];
        } else {
            let slices = self.nodes.split_at_mut(handle1.index());
            node2 = &mut slices.0[handle2.index()];
            node1 = &mut slices.1[0];
        }

        f(node1, node2);
    }

    pub fn nodes(&self) -> &[N] {
        &self.nodes
    }

    pub fn nodes_mut(&mut self) -> &mut [N] {
        &mut self.nodes
    }

    pub fn node(&self, handle: Handle<N>) -> &N {
        &self.nodes[handle.index()]
    }

    pub fn node_mut(&mut self, handle: Handle<N>) -> &mut N {
        &mut self.nodes[handle.index()]
    }
}

pub trait WithHandle<N: WithHandle<N>> {
    fn handle(&self) -> Handle<N>;

    fn handle_mut(&mut self) -> &mut Handle<N>;
}

pub struct Handle<N: WithHandle<N>> {
    index: u32,
    _phantom: PhantomData<N>,
}

impl<N: WithHandle<N>> Handle<N> {
    pub fn new(index: u32) -> Self {
        Handle {
            index,
            _phantom: PhantomData,
        }
    }

    pub fn unset() -> Self {
        Handle {
            index: u32::MAX,
            _phantom: PhantomData,
        }
    }

    fn index(self) -> usize {
        self.index as usize
    }
}

impl<N: WithHandle<N>> Clone for Handle<N> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<N: WithHandle<N>> Copy for Handle<N> {}

impl<N: WithHandle<N>> fmt::Debug for Handle<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NodeHandle")
            .field("index", &self.index)
            .finish()
    }
}

impl<N: WithHandle<N>> fmt::Display for Handle<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.index)
    }
}

impl<N: WithHandle<N>> Eq for Handle<N> {}

impl<N: WithHandle<N>> Ord for Handle<N> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.index.cmp(&other.index)
    }
}

impl<N: WithHandle<N>> PartialEq for Handle<N> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<N: WithHandle<N>> PartialOrd for Handle<N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn added_node_has_correct_handle() {
        let mut nodes = NodesWithHandles::new();

        let handle = nodes.add_node(SimpleNodeWithHandle::new(0));
        println!("{:?}", handle);

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

        nodes.remove_nodes(&vec![node0_handle, node2_handle], |_, _| {});

        assert_eq!(nodes.nodes.len(), 1);
        let node = &nodes.nodes()[0];
        assert_eq!(node.id, 1);
        assert_eq!(node.handle().index, 0);
    }

    #[test]
    fn gets_callback_for_swapped_node() {
        let mut nodes = NodesWithHandles::new();
        let node0_handle = nodes.add_node(SimpleNodeWithHandle::new(0));
        nodes.add_node(SimpleNodeWithHandle::new(1));
        let mut num = 0;

        nodes.remove_nodes(&vec![node0_handle], |node, prev_handle| {
            assert_eq!(node.handle.index, 0);
            assert_eq!(prev_handle.index, 1);
            num = 42;
        });

        assert_eq!(num, 42);
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct SimpleNodeWithHandle {
        handle: Handle<SimpleNodeWithHandle>,
        pub id: i32,
    }

    impl SimpleNodeWithHandle {
        pub fn new(id: i32) -> Self {
            SimpleNodeWithHandle {
                handle: Handle::unset(),
                id,
            }
        }
    }

    impl WithHandle<SimpleNodeWithHandle> for SimpleNodeWithHandle {
        fn handle(&self) -> Handle<SimpleNodeWithHandle> {
            self.handle
        }

        fn handle_mut(&mut self) -> &mut Handle<SimpleNodeWithHandle> {
            &mut self.handle
        }
    }
}
