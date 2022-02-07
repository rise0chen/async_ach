use ach_linked::{LinkedList, Node};
use core::task::Waker;

pub struct WakerNode(Node<Waker>);
impl WakerNode {
    pub const fn new() -> Self {
        Self(Node::new())
    }
    pub fn register(&self, waker: &Waker) {
        if let Err(err) = self.0.try_replace(waker.clone()) {
            err.input.wake()
        }
    }
}

pub struct WakerLinked {
    linked: LinkedList<Waker>,
}
impl WakerLinked {
    pub const fn new() -> Self {
        Self {
            linked: LinkedList::new(),
        }
    }
    /// Adds a node to the LinkedList.
    ///
    /// # Safety
    ///
    /// This function is only safe as long as `node` is guaranteed to
    /// get removed from the list before it gets moved or dropped.
    ///
    /// In addition to this `node` may not be added to another other list before
    /// it is removed from the current one.
    pub unsafe fn push(&self, node: &mut WakerNode) {
        self.linked.push(&mut node.0)
    }
    /// Wake a waiter, and remove it.
    ///
    /// Returns false if the pool is empty.
    pub fn wake_one(&self) -> bool {
        let mut success = false;
        for node in self.linked.iter() {
            if let Ok(Some(waker)) = node.try_take() {
                waker.wake();
                success = true;
                break;
            }
        }
        success
    }
    /// Wake all waiter, and remove it.
    ///
    /// returns the number of had waked
    pub fn wake_all(&self) -> usize {
        let mut num = 0;
        for node in self.linked.iter() {
            if let Ok(Some(waker)) = node.try_take() {
                waker.wake();
                num += 1;
            }
        }
        num
    }
}
