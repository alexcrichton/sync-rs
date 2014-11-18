//! # sync-rs: revamping std::sync
//!
//! The purpose of this crate is to provide a prototype implementation of a
//! revamp'd std::sync API. It strives to maintain the high-level types while
//! bringing their implementations much closer to the corresponding system
//! primitives where available.
//!
//! The crate id structured into a few primary modules:
//!
//! * The `sys` module contains 0-cost, very unsafe, raw bindings to the system
//!   primitives. The behavior of these primitives may vary slightly across
//!   platforms and are generally considered too unsafe to use. It is highly
//!   recommended to use the safe primitives at the top level instead.
//!
//! * The crate root has a number of types exported which are all safe to use
//!   and provide alternatives to the `sys` module. The types provided are not
//!   high-level abstractions but rather the thinnest layer on top of the `sys`
//!   apis to ensure that usage is safe across all platforms.
//!
//! TBD: cells and such

#![feature(unsafe_destructor, tuple_indexing)]
#![deny(missing_docs)]

extern crate libc;
extern crate alloc;

pub use mutex::{Mutex, StaticMutex, MUTEX_INIT};
pub use mutex::Guard as MutexGuard;
pub use rwlock::{RWLock, StaticRWLock, RWLOCK_INIT};
pub use rwlock::ReadGuard as RWLockReadGuard;
pub use rwlock::WriteGuard as RWLockWriteGuard;
pub use condvar::{Condvar, StaticCondvar, CONDVAR_INIT};

pub mod sys;

mod mutex;
mod condvar;
mod rwlock;
