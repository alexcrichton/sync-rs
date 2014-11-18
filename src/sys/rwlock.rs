pub struct RWLock(imp::RWLock);

pub const RWLOCK_INIT: RWLock = RWLock(imp::RWLOCK_INIT);

impl RWLock {
    pub unsafe fn new() -> RWLock { RWLock(imp::RWLock::new()) }
    pub unsafe fn read(&self) { self.0.read() }
    pub unsafe fn try_read(&self) -> bool { self.0.try_read() }
    pub unsafe fn write(&self) { self.0.write() }
    pub unsafe fn try_write(&self) -> bool { self.0.try_write() }
    pub unsafe fn read_unlock(&self) { self.0.read_unlock() }
    pub unsafe fn write_unlock(&self) { self.0.write_unlock() }
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
        pub unsafe fn new() -> RWLock {
            // Might be moved and address is changing it is better to avoid
            // initialization of potentially opaque OS data before it landed
            RWLOCK_INIT
        }
        pub unsafe fn read(&self) {
            let r = ffi::pthread_rwlock_rdlock(self.inner.get());
            debug_assert_eq!(r, 0);
        }
        pub unsafe fn try_read(&self) -> bool {
            ffi::pthread_rwlock_tryrdlock(self.inner.get()) == 0
        }
        pub unsafe fn write(&self) {
            let r = ffi::pthread_rwlock_wrlock(self.inner.get());
            debug_assert_eq!(r, 0);
        }
        pub unsafe fn try_write(&self) -> bool {
            ffi::pthread_rwlock_trywrlock(self.inner.get()) == 0
        }
        pub unsafe fn read_unlock(&self) {
            let r = ffi::pthread_rwlock_unlock(self.inner.get());
            debug_assert_eq!(r, 0);
        }
        pub unsafe fn write_unlock(&self) { self.read_unlock() }
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
        pub unsafe fn new() -> RWLock { RWLOCK_INIT }

        pub unsafe fn read(&self) {
            ffi::AcquireSRWLockShared(self.inner.get())
        }
        pub unsafe fn try_read(&self) -> bool {
            ffi::TryAcquireSRWLockShared(self.inner.get()) != 0
        }
        pub unsafe fn write(&self) {
            ffi::AcquireSRWLockExclusive(self.inner.get())
        }
        pub unsafe fn try_write(&self) -> bool {
            ffi::TryAcquireSRWLockExclusive(self.inner.get()) != 0
        }
        pub unsafe fn read_unlock(&self) {
            ffi::ReleaseSRWLockShared(self.inner.get())
        }
        pub unsafe fn write_unlock(&self) {
            ffi::ReleaseSRWLockExclusive(self.inner.get())
        }

        pub unsafe fn destroy(&self) {
            // ...
        }
    }
}

