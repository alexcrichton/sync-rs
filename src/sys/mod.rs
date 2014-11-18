#![allow(non_camel_case_types)]

pub use self::mutex::{Mutex, MUTEX_INIT};
pub use self::condvar::{Condvar, CONDVAR_INIT};

mod mutex;
mod condvar;

#[cfg(unix)] #[path = "unix.rs"] mod ffi;
#[cfg(windows)] #[path = "windows.rs"] mod ffi;
