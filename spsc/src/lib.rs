#![no_std]
#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
mod heap;
mod heapless;

#[cfg(feature = "alloc")]
pub use heap::*;
#[cfg(not(feature = "alloc"))]
pub use heapless::*;
