//! Bindings to system primitives
//!
//! This module contains bindings to the OS-provided synchronization primitives.
//! All functions and methods in this module are unsafe as the goal is to
//! provide a 0-cost abstraction and cross-platform abstraction in this module,
//! not to provide a set of safe primitives to use.
//!
//! Normal usage should favor the top-level types of this crate instead.

#![allow(non_camel_case_types)]

pub use self::mutex::{Mutex, MUTEX_INIT};
pub use self::condvar::{Condvar, CONDVAR_INIT};
pub use self::rwlock::{RWLock, RWLOCK_INIT};

mod mutex;
mod condvar;
mod rwlock;

#[cfg(unix)] #[path = "unix.rs"] mod ffi;
#[cfg(windows)] #[path = "windows.rs"] mod ffi;
