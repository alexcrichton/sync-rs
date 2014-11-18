/// An OS-based reader-writer lock.
///
/// This structure is entirely unsafe and serves as the lowest layer of a
/// cross-platform binding of system rwlocks. It is recommended to use the
/// safer types at the top level of this crate instead of this type.
pub struct RWLock(imp::RWLock);

/// Constant initializer for static RWLocks.
pub const RWLOCK_INIT: RWLock = RWLock(imp::RWLOCK_INIT);

impl RWLock {
    /// Creates a new instance of an RWLock.
    ///
    /// Usage of an RWLock is undefined if it is moved after its first use (any
    /// function calls below).
    #[inline]
    pub unsafe fn new() -> RWLock { RWLock(imp::RWLock::new()) }

    /// Acquire shared access to the underlying lock, blocking the current
    /// thread to do so.
    ///
    /// Behavior is undefined if the rwlock has been moved between this and any
    /// previous methodo call.
    #[inline]
    pub unsafe fn read(&self) { self.0.read() }

    /// Attempt to acquire shared access to this lock, returning whether it
    /// succeeded or not.
    ///
    /// This function does not block the current thread.
    ///
    /// Behavior is undefined if the rwlock has been moved between this and any
    /// previous methodo call.
    #[inline]
    pub unsafe fn try_read(&self) -> bool { self.0.try_read() }

    /// Acquire write access to the underlying lock, blocking the current thread
    /// to do so.
    ///
    /// Behavior is undefined if the rwlock has been moved between this and any
    /// previous methodo call.
    #[inline]
    pub unsafe fn write(&self) { self.0.write() }

    /// Attempt to acquire exclusive access to this lock, returning whether it
    /// succeeded or not.
    ///
    /// This function does not block the current thread.
    ///
    /// Behavior is undefined if the rwlock has been moved between this and any
    /// previous methodo call.
    #[inline]
    pub unsafe fn try_write(&self) -> bool { self.0.try_write() }

    /// Unlock previously acquired shared access to this lock.
    ///
    /// Behavior is undefined if the current thread does not have shared access.
    #[inline]
    pub unsafe fn read_unlock(&self) { self.0.read_unlock() }

    /// Unlock previously acquired exclusive access to this lock.
    ///
    /// Behavior is undefined if the current thread does not currently have
    /// exclusive access.
    #[inline]
    pub unsafe fn write_unlock(&self) { self.0.write_unlock() }

    /// Destroy OS-related resources with this RWLock.
    ///
    /// Behavior is undefined if there are any currently active users of this
    /// lock.
    #[inline]
    pub unsafe fn destroy(&self) { self.0.destroy() }
}

#[cfg(unix)]
mod imp {
    use std::cell::UnsafeCell;
    use sys::ffi;

    pub struct RWLock { inner: UnsafeCell<ffi::pthread_rwlock_t> }

    pub const RWLOCK_INIT: RWLock = RWLock {
        inner: UnsafeCell { value: ffi::PTHREAD_RWLOCK_INITIALIZER },
    };

    impl RWLock {
        #[inline]
        pub unsafe fn new() -> RWLock {
            // Might be moved and address is changing it is better to avoid
            // initialization of potentially opaque OS data before it landed
            RWLOCK_INIT
        }
        #[inline]
        pub unsafe fn read(&self) {
            let r = ffi::pthread_rwlock_rdlock(self.inner.get());
            debug_assert_eq!(r, 0);
        }
        #[inline]
        pub unsafe fn try_read(&self) -> bool {
            ffi::pthread_rwlock_tryrdlock(self.inner.get()) == 0
        }
        #[inline]
        pub unsafe fn write(&self) {
            let r = ffi::pthread_rwlock_wrlock(self.inner.get());
            debug_assert_eq!(r, 0);
        }
        #[inline]
        pub unsafe fn try_write(&self) -> bool {
            ffi::pthread_rwlock_trywrlock(self.inner.get()) == 0
        }
        #[inline]
        pub unsafe fn read_unlock(&self) {
            let r = ffi::pthread_rwlock_unlock(self.inner.get());
            debug_assert_eq!(r, 0);
        }
        #[inline]
        pub unsafe fn write_unlock(&self) { self.read_unlock() }
        #[inline]
        pub unsafe fn destroy(&self) {
            let r = ffi::pthread_rwlock_destroy(self.inner.get());
            debug_assert_eq!(r, 0);
        }
    }
}

#[cfg(windows)]
mod imp {
    use std::cell::UnsafeCell;

    use sys::ffi;

    pub struct RWLock { inner: UnsafeCell<ffi::SRWLOCK> }

    pub const RWLOCK_INIT: RWLock = RWLock {
        inner: UnsafeCell { value: ffi::SRWLOCK_INIT }
    };

    impl RWLock {
        #[inline]
        pub unsafe fn new() -> RWLock { RWLOCK_INIT }

        #[inline]
        pub unsafe fn read(&self) {
            ffi::AcquireSRWLockShared(self.inner.get())
        }
        #[inline]
        pub unsafe fn try_read(&self) -> bool {
            ffi::TryAcquireSRWLockShared(self.inner.get()) != 0
        }
        #[inline]
        pub unsafe fn write(&self) {
            ffi::AcquireSRWLockExclusive(self.inner.get())
        }
        #[inline]
        pub unsafe fn try_write(&self) -> bool {
            ffi::TryAcquireSRWLockExclusive(self.inner.get()) != 0
        }
        #[inline]
        pub unsafe fn read_unlock(&self) {
            ffi::ReleaseSRWLockShared(self.inner.get())
        }
        #[inline]
        pub unsafe fn write_unlock(&self) {
            ffi::ReleaseSRWLockExclusive(self.inner.get())
        }

        #[inline]
        pub unsafe fn destroy(&self) {
            // ...
        }
    }
}

