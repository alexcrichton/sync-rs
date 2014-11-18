#![allow(non_camel_case_types)]

pub use self::mutex::{Mutex, MUTEX_INIT};
pub use self::condvar::{Condvar, CONDVAR_INIT};
pub use self::rwlock::{RWLock, RWLOCK_INIT};

mod mutex;
mod condvar;
mod rwlock;

#[cfg(unix)] #[path = "unix.rs"] mod ffi;
#[cfg(windows)] #[path = "windows.rs"] mod ffi;
