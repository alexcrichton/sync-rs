//! # sync-rs: revamping std::sync
//!
//! The purpose of this crate is to provide a prototype implementation of a
//! revamp'd std::sync API. It strives to maintain the high-level types while
//! bringing their implementations much closer to the corresponding system
//! primitives where available.
//!
//! # Abstraction layer
//!
//! The crate id structured into a two primary modes:
//!
//! * The `sys` module contains 0-cost, very unsafe, raw bindings to the system
//!   primitives. The behavior of these primitives may vary slightly across
//!   platforms and are generally considered too unsafe to use. It is highly
//!   recommended to use the safe primitives at the top level instead.
//!
//! * The crate root has a number of types exported which are all safe to use
//!   and provide alternatives to the `sys` module. These primitives all provide
//!   safety features such as poisoning and RAII guards. Types like `Mutex` and
//!   `RWLock` also provide the ability to contain the data they are protecting.
//!
//! # Poisoning
//!
//! The `Mutex` and `RWLock` types in this module implement a strategy referred
//! to as poisoning in order to prevent access to possibly invalid data. If a
//! thread panics with write-access to one of these two locks. then all future
//! accesses to the lock will panic immediately.
//!
//! # Static initialization
//!
//! This crate supports a number of statically initialized primitives for use in
//! setting up C libraries, for example. Types which are normally not statically
//! initialized have a `Static`-prefix type to use (`StaticMutex`,
//! `StaticCondvar`, `StaticRWLock`). A form of one-time initialization (`Once`)
//! is also provided. All types have a `*_INIT` constant which may be used to
//! initialize the primitive.
//!
//! # Custom primitives
//!
//! Using the system-provided mutexes, condition variables, and rwlocks, this
//! crate also builds abstractions such as `Once`, `Semaphore`, and `Barrier`
//! which do not bind to the corresponding system abstraction if one is
//! available.

#![feature(unsafe_destructor, tuple_indexing)]
#![deny(missing_docs)]

extern crate libc;
extern crate alloc;

pub use mutex::{Mutex, MutexGuard, StaticMutex, StaticMutexGuard, MUTEX_INIT};
pub use rwlock::{RWLock, StaticRWLock, RWLOCK_INIT};
pub use rwlock::{RWLockReadGuard, RWLockWriteGuard};
pub use rwlock::{StaticRWLockReadGuard, StaticRWLockWriteGuard};
pub use condvar::{Condvar, StaticCondvar, CONDVAR_INIT, AsMutexGuard};
pub use one::{Once, ONCE_INIT};
pub use semaphore::{Semaphore, SemaphoreGuard};
pub use barrier::Barrier;
pub use std::sync::{Arc, Weak, TaskPool, Future, atomic};

pub mod sys;

mod condvar;
mod mutex;
mod one;
mod rwlock;
mod semaphore;
mod barrier;

mod poison;
