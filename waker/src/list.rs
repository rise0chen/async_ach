//! Only work on single thread

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
        if let Some(list) = self.list.take_all() {
            let mut waker = unsafe { &mut *(list as *mut Node<WakerEntity<T>>) };
            let ret = loop {
                if waker.wake() {
                    break true;
                } else {
                    if let Some(list) = waker.next() {
                        waker = list
                    } else {
                        break false;
                    }
                }
            };
            unsafe { self.list.push_list(list) };
            ret
        } else {
            false
        }
    }
    /// Wake all waiter, and remove it.
    ///
    /// returns the number of had waked
    pub fn wake_all(&self) -> usize {
        let mut num = 0;
        if let Some(list) = self.list.take_all() {
            let mut waker = unsafe { &mut *(list as *mut Node<WakerEntity<T>>) };
            loop {
                if waker.wake() {
                    num += 1;
                }
                if let Some(list) = waker.next() {
                    waker = list
                } else {
                    break;
                }
            }
            unsafe { self.list.push_list(list) };
        }
        num
    }
    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all elements e such that f(&e) returns false.
    /// This method operates in place, visiting each element exactly once in the original order,
    /// but not preserves the order of the retained elements.
    pub fn retain(&self, mut f: impl FnMut(&WakerEntity<T>) -> bool) {
        let list = self.list.take_all();
        if let Some(list) = list {
            let mut remain: Option<&mut Node<WakerEntity<T>>> = None;
            for waker in list.into_iter() {
                if f(&*waker) {
                    if let Some(remain) = &mut remain {
                        unsafe { remain.push(waker) };
                    } else {
                        remain = Some(waker);
                    }
                }
            }
            if let Some(remain) = remain {
                unsafe { self.list.push_list(remain) };
            }
        }
    }
}
