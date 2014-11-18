#![feature(unsafe_destructor, tuple_indexing)]

extern crate libc;
extern crate alloc;

pub use mutex::{Mutex, StaticMutex, MUTEX_INIT};
pub use mutex::Guard as MutexGuard;

pub use condvar::{Condvar, StaticCondvar, CONDVAR_INIT};

pub mod sys;

mod mutex;
mod condvar;
