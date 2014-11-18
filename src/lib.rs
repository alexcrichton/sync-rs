#![feature(unsafe_destructor)]

extern crate libc;
extern crate alloc;

pub use mutex::{Mutex, StaticMutex, MUTEX_INIT};
pub use mutex::Guard as MutexGuard;

pub use condvar::{Condvar, StaticCondvar, CONDVAR_INIT};

#[cfg(unix)] #[path = "unix.rs"] pub mod sys;
#[cfg(windows)] #[path = "windows.rs"] pub mod sys;

mod mutex;
mod condvar;
