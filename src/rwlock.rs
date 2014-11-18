use std::kinds::marker;

use sys;

/// A reader-writer lock
///
/// This type of lock allows a number of readers or at most one writer at any
/// point in time. The write portion of this lock typically allows modification
/// of the underlying data (exclusive access) and the read portion of this lock
/// typically allows for read-only access (shared access).
///
/// # Example
///
/// ```
/// use sync::RWLock;
///
/// let lock = RWLock::new();
///
/// // many reader locks can be held at once
/// {
///     let _r1 = lock.read();
///     let _r2 = lock.read();
/// } // read locks are dropped at this point
///
/// // only one write lock may be held, however
/// {
///     let _w = lock.write();
/// } // write lock is dropped here
/// ```
pub struct RWLock { inner: Box<sys::RWLock> }

/// Structure representing a staticaly allocated RWLock.
///
/// This structure is intended to be used inside of a `static` and will provide
/// automatic global access as well as lazy initialization. The internal
/// resources of this RWLock, however, must be manually deallocated.
///
/// # Example
///
/// ```
/// use sync::{StaticRWLock, RWLOCK_INIT};
///
/// static LOCK: StaticRWLock = RWLOCK_INIT;
///
/// {
///     let _g = LOCK.read();
///     // ... shared read access
/// }
/// {
///     let _g = LOCK.write();
///     // ... exclusive write access
/// }
/// unsafe { LOCK.destroy() } // free all resources
/// ```
pub struct StaticRWLock { inner: sys::RWLock }

/// Constant initialization for a statically-initialized rwlock.
pub const RWLOCK_INIT: StaticRWLock = StaticRWLock {
    inner: sys::RWLOCK_INIT
};

/// RAII structure used to release the shared read access of a lock when
/// dropped.
#[must_use]
pub struct ReadGuard<'a> {
    lock: &'a sys::RWLock,
    marker: marker::NoSend,
}

/// RAII structure used to release the exclusive write access of a lock when
/// dropped.
#[must_use]
pub struct WriteGuard<'a> {
    lock: &'a sys::RWLock,
    marker: marker::NoSend,
}

impl RWLock {
    /// Creates a new instance of an RWLock which is unlocked and read to go.
    pub fn new() -> RWLock {
        RWLock { inner: box unsafe { sys::RWLock::new() } }
    }

    /// Locks this rwlock with shared read access, blocking the current thread
    /// until it can be acquired.
    ///
    /// The calling thread will be blocked until there are no more writers
    /// available. There may be other readers currently inside the lock when
    /// this method returns.
    ///
    /// Returns an RAII guard which will release this thread's shared access
    /// once it is dropped.
    #[inline]
    pub fn read(&self) -> ReadGuard {
        unsafe { self.inner.read() }
        ReadGuard::new(&*self.inner)
    }

    /// Attempt to acquire this lock with shared read access.
    ///
    /// This function will never block and will return immediately if `read`
    /// would otherwise succeed. Returns `Some` of an RAII guard which will
    /// release the shared access of this thread when dropped, or `None` if the
    /// access could not be granted.
    #[inline]
    pub fn try_read(&self) -> Option<ReadGuard> {
        if unsafe { self.inner.try_read() } {
            Some(ReadGuard::new(&*self.inner))
        } else {
            None
        }
    }

    /// Lock this rwlock with exclusive write access, blocking the current
    /// thread until it can be acquired.
    ///
    /// This function will not return while other writers or other readers
    /// currently have access to the lock.
    ///
    /// Returns an RAII guard which will drop the write access of this rwlock
    /// when dropped.
    #[inline]
    pub fn write(&self) -> WriteGuard {
        unsafe { self.inner.write() }
        WriteGuard::new(&*self.inner)
    }

    /// Attempt to lock this rwlock with exclusive write access.
    ///
    /// This function does not ever block, and it will return `None` if a call
    /// to `write` would otherwise block. If successful, an RAII guard is
    /// returned.
    #[inline]
    pub fn try_write(&self) -> Option<WriteGuard> {
        if unsafe { self.inner.try_write() } {
            Some(WriteGuard::new(&*self.inner))
        } else {
            None
        }
    }
}

impl Drop for RWLock {
    fn drop(&mut self) {
        unsafe { self.inner.destroy() }
    }
}

impl StaticRWLock {
    /// Locks this rwlock with shared read access, blocking the current thread
    /// until it can be acquired.
    ///
    /// The calling thread will be blocked until there are no more writers
    /// available. There may be other readers currently inside the lock when
    /// this method returns.
    ///
    /// Returns an RAII guard which will release this thread's shared access
    /// once it is dropped.
    #[inline]
    pub fn read(&'static self) -> ReadGuard {
        unsafe { self.inner.read() }
        ReadGuard::new(&self.inner)
    }

    /// Attempt to acquire this lock with shared read access.
    ///
    /// This function will never block and will return immediately if `read`
    /// would otherwise succeed. Returns `Some` of an RAII guard which will
    /// release the shared access of this thread when dropped, or `None` if the
    /// access could not be granted.
    #[inline]
    pub fn try_read(&'static self) -> Option<ReadGuard> {
        if unsafe { self.inner.try_read() } {
            Some(ReadGuard::new(&self.inner))
        } else {
            None
        }
    }

    /// Lock this rwlock with exclusive write access, blocking the current
    /// thread until it can be acquired.
    ///
    /// This function will not return while other writers or other readers
    /// currently have access to the lock.
    ///
    /// Returns an RAII guard which will drop the write access of this rwlock
    /// when dropped.
    #[inline]
    pub fn write(&'static self) -> WriteGuard {
        unsafe { self.inner.write() }
        WriteGuard::new(&self.inner)
    }

    /// Attempt to lock this rwlock with exclusive write access.
    ///
    /// This function does not ever block, and it will return `None` if a call
    /// to `write` would otherwise block. If successful, an RAII guard is
    /// returned.
    #[inline]
    pub fn try_write(&'static self) -> Option<WriteGuard> {
        if unsafe { self.inner.try_write() } {
            Some(WriteGuard::new(&self.inner))
        } else {
            None
        }
    }

    /// Deallocate all resources associated with this static lock.
    ///
    /// This method is unsafe to call as there is no guarantee that there are no
    /// active users of the lock, and this also doesn't prevent any future users
    /// of this lock. This method is required to be called to not leak memory on
    /// all platforms.
    pub unsafe fn destroy(&'static self) {
        self.inner.destroy()
    }
}

impl<'rwlock> ReadGuard<'rwlock> {
    fn new<'a>(lock: &'a sys::RWLock) -> ReadGuard<'a> {
        ReadGuard { lock: lock, marker: marker::NoSend }
    }
}
impl<'rwlock> WriteGuard<'rwlock> {
    fn new<'a>(lock: &'a sys::RWLock) -> WriteGuard<'a> {
        WriteGuard { lock: lock, marker: marker::NoSend }
    }
}

#[unsafe_destructor]
impl<'rwlock> Drop for ReadGuard<'rwlock> {
    fn drop(&mut self) {
        unsafe { self.lock.read_unlock(); }
    }
}

#[unsafe_destructor]
impl<'rwlock> Drop for WriteGuard<'rwlock> {
    fn drop(&mut self) {
        unsafe { self.lock.write_unlock(); }
    }
}

#[cfg(test)]
mod tests {
    use std::rand::{mod, Rng};
    use super::{RWLock, StaticRWLock, RWLOCK_INIT};

    #[test]
    fn smoke() {
        let l = RWLock::new();
        drop(l.read());
        drop(l.write());
        drop((l.read(), l.read()));
        drop(l.write());
    }

    #[test]
    fn static_smoke() {
        static R: StaticRWLock = RWLOCK_INIT;
        drop(R.read());
        drop(R.write());
        drop((R.read(), R.read()));
        drop(R.write());
        unsafe { R.destroy(); }
    }

    #[test]
    fn frob() {
        static R: StaticRWLock = RWLOCK_INIT;
        static N: uint = 10;
        static M: uint = 1000;

        let (tx, rx) = channel::<()>();
        for _ in range(0, N) {
            let tx = tx.clone();
            spawn(proc() {
                let mut rng = rand::task_rng();
                for _ in range(0, M) {
                    if rng.gen_weighted_bool(N) {
                        drop(R.write());
                    } else {
                        drop(R.read());
                    }
                }
                drop(tx);
            });
        }
        drop(tx);
        let _ = rx.recv_opt();
        unsafe { R.destroy(); }
    }
}
