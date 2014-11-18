use std::kinds::marker;

use sys;

/// A mutual exclusion primitive useful for protecting shared data
///
/// This mutex will properly block tasks waiting for the lock to become
/// available. The mutex can also be statically initialized or created via a
/// `new` constructor.
///
/// # Example
///
/// ```rust
/// use sync::Mutex;
///
/// let m = Mutex::new();
/// let guard = m.lock();
/// // do some work
/// drop(guard); // unlock the lock
/// ```
pub struct Mutex {
    // Note that this static mutex is in a *box*, not inlined into the struct
    // itself. This is done for memory safety reasons with the usage of a
    // StaticNativeMutex inside the static mutex above. Once a native mutex has
    // been used once, its address can never change (it can't be moved). This
    // mutex type can be safely moved at any time, so to ensure that the native
    // mutex is used correctly we box the inner lock to give it a constant
    // address.
    lock: Box<sys::Mutex>,
}

/// The static mutex type is provided to allow for static allocation of mutexes.
///
/// Note that this is a separate type because using a Mutex correctly means that
/// it needs to have a destructor run. In Rust, statics are not allowed to have
/// destructors. As a result, a `StaticMutex` has one extra method when compared
/// to a `Mutex`, a `destroy` method. This method is unsafe to call, and
/// documentation can be found directly on the method.
///
/// # Example
///
/// ```rust
/// use sync::{StaticMutex, MUTEX_INIT};
///
/// static LOCK: StaticMutex = MUTEX_INIT;
///
/// {
///     let _g = LOCK.lock();
///     // do some productive work
/// }
/// // lock is unlocked here.
/// ```
pub struct StaticMutex {
    lock: sys::Mutex,
}

/// An RAII implementation of a "scoped lock" of a mutex. When this structure is
/// dropped (falls out of scope), the lock will be unlocked.
#[must_use]
pub struct MutexGuard<'a> {
    lock: &'a sys::Mutex,
    marker: marker::NoSend,
}

/// Static initialization of a mutex. This constant can be used to initialize
/// other mutex constants.
pub const MUTEX_INIT: StaticMutex = StaticMutex { lock: sys::MUTEX_INIT };

impl Mutex {
    /// Creates a new mutex in an unlocked state ready for use.
    pub fn new() -> Mutex {
        Mutex { lock: box unsafe { sys::Mutex::new() } }
    }

    /// Acquires a mutex, blocking the current task until it is able to do so.
    ///
    /// This function will block the local task until it is available to acquire
    /// the mutex. Upon returning, the task is the only task with the mutex
    /// held. An RAII guard is returned to allow scoped unlock of the lock. When
    /// the guard goes out of scope, the mutex will be unlocked.
    pub fn lock(&self) -> MutexGuard {
        unsafe { self.lock.lock() }
        MutexGuard::new(&*self.lock)
    }

    /// Attempts to acquire this lock.
    ///
    /// If the lock could not be acquired at this time, then `None` is returned.
    /// Otherwise, an RAII guard is returned. The lock will be unlocked when the
    /// guard is dropped.
    ///
    /// This function does not block.
    pub fn try_lock<'a>(&'a self) -> Option<MutexGuard<'a>> {
        if unsafe { self.lock.try_lock() } {
            Some(MutexGuard::new(&*self.lock))
        } else {
            None
        }
    }
}

impl Drop for Mutex {
    fn drop(&mut self) {
        // This is actually safe b/c we know that there is no further usage of
        // this mutex (it's up to the user to arrange for a mutex to get
        // dropped, that's not our job)
        unsafe { self.lock.destroy() }
    }
}

impl StaticMutex {
    /// Acquires this lock, see `Mutex::lock`
    pub fn lock(&'static self) -> MutexGuard<'static> {
        unsafe { self.lock.lock() }
        MutexGuard::new(&self.lock)
    }

    /// Attempts to grab this lock, see `Mutex::try_lock`
    pub fn try_lock(&'static self) -> Option<MutexGuard<'static>> {
        if unsafe { self.lock.try_lock() } {
            Some(MutexGuard::new(&self.lock))
        } else {
            None
        }
    }

    /// Deallocates resources associated with this static mutex.
    ///
    /// This method is unsafe because it provides no guarantees that there are
    /// no active users of this mutex, and safety is not guaranteed if there are
    /// active users of this mutex.
    ///
    /// This method is required to ensure that there are no memory leaks on
    /// *all* platforms. It may be the case that some platforms do not leak
    /// memory if this method is not called, but this is not guaranteed to be
    /// true on all platforms.
    pub unsafe fn destroy(&'static self) {
        self.lock.destroy()
    }
}

pub fn guard_inner<'a>(guard: &MutexGuard<'a>) -> &'a sys::Mutex { guard.lock }

impl<'mutex> MutexGuard<'mutex> {
    fn new<'a>(lock: &'a sys::Mutex) -> MutexGuard<'a> {
        MutexGuard { lock: lock, marker: marker::NoSend }
    }
}

#[unsafe_destructor]
impl<'mutex> Drop for MutexGuard<'mutex> {
    fn drop(&mut self) {
        unsafe { self.lock.unlock(); }
    }
}

#[cfg(test)]
mod test {
    use super::{Mutex, StaticMutex, MUTEX_INIT};

    #[test]
    fn smoke() {
        let m = Mutex::new();
        drop(m.lock());
        drop(m.lock());
    }

    #[test]
    fn smoke_static() {
        static M: StaticMutex = MUTEX_INIT;
        unsafe {
            drop(M.lock());
            drop(M.lock());
            M.destroy();
        }
    }

    #[test]
    fn lots_and_lots() {
        static M: StaticMutex = MUTEX_INIT;
        static mut CNT: uint = 0;
        static J: uint = 1000;
        static K: uint = 3;

        fn inc() {
            for _ in range(0, J) {
                unsafe {
                    let _g = M.lock();
                    CNT += 1;
                }
            }
        }

        let (tx, rx) = channel();
        for _ in range(0, K) {
            let tx2 = tx.clone();
            spawn(proc() { inc(); tx2.send(()); });
            let tx2 = tx.clone();
            spawn(proc() { inc(); tx2.send(()); });
        }

        drop(tx);
        for _ in range(0, 2 * K) {
            rx.recv();
        }
        assert_eq!(unsafe {CNT}, J * K * 2);
        unsafe {
            M.destroy();
        }
    }

    #[test]
    fn try_lock() {
        let m = Mutex::new();
        assert!(m.try_lock().is_some());
    }
}

