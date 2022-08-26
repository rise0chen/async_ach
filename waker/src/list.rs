use crate::WakerEntity;
use ach_linked::LinkedList;
pub use ach_linked::Node;

pub struct WakerList<T> {
    list: LinkedList<WakerEntity<T>>,
}
impl<T> WakerList<T> {
    pub const fn new() -> Self {
        Self {
            list: LinkedList::new(),
        }
    }
    /// Register a waker
    ///
    /// Safety:
    /// This function is only safe as long as `node` is guaranteed to
    /// get removed from the list before it gets moved or dropped.
    pub unsafe fn register(&self, waker: &mut Node<WakerEntity<T>>) {
        self.list.push(waker)
    }
    /// Removes a node from the LinkedList.
    pub fn remove(&self, waker: &mut Node<WakerEntity<T>>) {
        self.list.remove(waker)
    }
    /// Wake a waiter, and remove it.
    ///
    /// Returns false if the list is empty.
    pub fn wake_one(&self) -> bool {
        if let Some(mut waker) = self.list.take_all() {
            loop {
                if waker.wake() {
                    if let Some(list) = waker.next() {
                        unsafe { self.list.push_list(list) };
                    }
                    return true;
                } else {
                    if let Some(list) = waker.next() {
                        waker = list
                    } else {
                        return false;
                    }
                }
            }
        } else {
            false
        }
    }
    /// Wake all waiter, and remove it.
    ///
    /// returns the number of had waked
    pub fn wake_all(&self) -> usize {
        let mut num = 0;
        let mut list = self.list.take_all();
        while let Some(waker) = list {
            waker.wake();
            list = waker.next();
            num += 1;
        }
        num
    }
}
