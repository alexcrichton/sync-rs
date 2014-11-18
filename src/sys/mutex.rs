pub use self::imp::raw;

pub struct Mutex(imp::Mutex);

pub const MUTEX_INIT: Mutex = Mutex(imp::MUTEX_INIT);

impl Mutex {
    pub unsafe fn new() -> Mutex { Mutex(imp::Mutex::new()) }
    pub unsafe fn lock(&self) { self.0.lock() }
    pub unsafe fn try_lock(&self) -> bool { self.0.try_lock() }
    pub unsafe fn unlock(&self) { self.0.unlock() }
    pub unsafe fn destroy(&self) { self.0.destroy() }
}

#[cfg(unix)]
mod imp {
    use std::cell::UnsafeCell;
    use sys::ffi;

    pub struct Mutex { inner: UnsafeCell<ffi::pthread_mutex_t> }

    pub unsafe fn raw(m: &super::Mutex) -> *mut ffi::pthread_mutex_t {
        m.0.inner.get()
    }

    pub const MUTEX_INIT: Mutex = Mutex {
        inner: UnsafeCell { value: ffi::PTHREAD_MUTEX_INITIALIZER },
    };

    impl Mutex {
        pub unsafe fn new() -> Mutex {
            // Might be moved and address is changing it is better to avoid
            // initialization of potentially opaque OS data before it landed
            MUTEX_INIT
        }
        pub unsafe fn lock(&self) {
            let r = ffi::pthread_mutex_lock(self.inner.get());
            debug_assert_eq!(r, 0);
        }
        pub unsafe fn unlock(&self) {
            let r = ffi::pthread_mutex_unlock(self.inner.get());
            debug_assert_eq!(r, 0);
        }
        pub unsafe fn try_lock(&self) -> bool {
            ffi::pthread_mutex_trylock(self.inner.get()) == 0
        }
        pub unsafe fn destroy(&self) {
            let r = ffi::pthread_mutex_destroy(self.inner.get());
            debug_assert_eq!(r, 0);
        }
    }
}

#[cfg(windows)]
mod imp {
    use std::sync::atomic;
    use alloc::{mod, heap};

    use libc::{DWORD};
    use sys::ffi;

    const SPIN_COUNT: DWORD = 4000;

    pub struct Mutex { inner: atomic::AtomicUint }

    pub const MUTEX_INIT: Mutex = Mutex { inner: atomic::INIT_ATOMIC_UINT };

    pub unsafe fn raw(m: &super::Mutex) -> ffi::LPCRITICAL_SECTION {
        m.0.get()
    }

    impl Mutex {
        pub unsafe fn new() -> Mutex {
            Mutex { inner: atomic::AtomicUint::new(init_lock() as uint) }
        }
        pub unsafe fn lock(&self) {
            ffi::EnterCriticalSection(self.get())
        }
        pub unsafe fn try_lock(&self) -> bool {
            ffi::TryEnterCriticalSection(self.get()) != 0
        }
        pub unsafe fn unlock(&self) {
            ffi::LeaveCriticalSection(self.get())
        }
        pub unsafe fn destroy(&self) {
            let lock = self.inner.swap(0, atomic::SeqCst);
            if lock != 0 { free_lock(lock as ffi::LPCRITICAL_SECTION) }
        }

        unsafe fn get(&self) -> ffi::LPCRITICAL_SECTION {
            match self.inner.load(atomic::SeqCst) {
                0 => {}
                n => return n as ffi::LPCRITICAL_SECTION
            }
            let lock = init_lock();
            match self.inner.compare_and_swap(0, lock as uint, atomic::SeqCst) {
                0 => return lock as ffi::LPCRITICAL_SECTION,
                _ => {}
            }
            free_lock(lock);
            return self.inner.load(atomic::SeqCst) as ffi::LPCRITICAL_SECTION;
        }
    }

    unsafe fn init_lock() -> ffi::LPCRITICAL_SECTION {
        let block = heap::allocate(ffi::CRITICAL_SECTION_SIZE, 8)
                            as ffi::LPCRITICAL_SECTION;
        if block.is_null() { alloc::oom() }
        ffi::InitializeCriticalSectionAndSpinCount(block, SPIN_COUNT);
        return block;
    }

    unsafe fn free_lock(h: ffi::LPCRITICAL_SECTION) {
        ffi::DeleteCriticalSection(h);
        heap::deallocate(h as *mut _, ffi::CRITICAL_SECTION_SIZE, 8);
    }
}
