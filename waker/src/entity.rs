use ach_option::AchOption;
use core::ops::Deref;
use core::task::Waker;

pub struct WakerEntity<T> {
    waker: AchOption<Waker>,
    val: T,
}
impl<T> WakerEntity<T> {
    pub fn new(waker: Waker, val: T) -> WakerEntity<T> {
        let waker = AchOption::new_with(waker);
        WakerEntity { waker, val }
    }
    pub fn set_waker(&self, waker: Waker) {
        self.waker.replace(waker);
    }
    /// return true if wake it success.
    pub fn wake(&self) -> bool {
        if let Some(waker) = self.waker.take() {
            waker.wake();
            true
        } else {
            false
        }
    }
    pub fn is_waked(&self) -> bool {
        self.waker.is_none()
    }
}
impl<T> Deref for WakerEntity<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.val
    }
}
