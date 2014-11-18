use std::time::Duration;

use sys::{mutex, Mutex};

pub struct Condvar(imp::Condvar);

pub const CONDVAR_INIT: Condvar = Condvar(imp::CONDVAR_INIT);

impl Condvar {
    pub unsafe fn new() -> Condvar { Condvar(imp::Condvar::new()) }
    pub unsafe fn signal(&self) { self.0.signal() }
    pub unsafe fn broadcast(&self) { self.0.broadcast() }
    pub unsafe fn wait(&self, mutex: &Mutex) { self.0.wait(mutex::raw(mutex)) }
    pub unsafe fn wait_timeout(&self, mutex: &Mutex, dur: Duration) -> bool {
        self.0.wait_timeout(mutex::raw(mutex), dur)
    }
    pub unsafe fn destroy(&self) { self.0.destroy() }
}

#[cfg(unix)]
mod imp {
    use std::cell::UnsafeCell;
    use std::time::Duration;
    use libc;

    use sys::ffi;

    pub struct Condvar { inner: UnsafeCell<ffi::pthread_cond_t> }

    pub const CONDVAR_INIT: Condvar = Condvar {
        inner: UnsafeCell { value: ffi::PTHREAD_COND_INITIALIZER },
    };

    impl Condvar {
        pub unsafe fn new() -> Condvar {
            // Might be moved and address is changing it is better to avoid
            // initialization of potentially opaque OS data before it landed
            Condvar { inner: UnsafeCell::new(ffi::PTHREAD_COND_INITIALIZER) }
        }
        pub unsafe fn signal(&self) {
            let r = ffi::pthread_cond_signal(self.inner.get());
            debug_assert_eq!(r, 0);
        }
        pub unsafe fn broadcast(&self) {
            let r = ffi::pthread_cond_broadcast(self.inner.get());
            debug_assert_eq!(r, 0);
        }
        pub unsafe fn wait(&self, mutex: *mut ffi::pthread_mutex_t) {
            let r = ffi::pthread_cond_wait(self.inner.get(), mutex);
            debug_assert_eq!(r, 0);
        }
        pub unsafe fn wait_timeout(&self, mutex: *mut ffi::pthread_mutex_t,
                                   dur: Duration) -> bool {
            assert!(dur >= Duration::nanoseconds(0));

            // First, figure out what time it currently is
            let mut tv = libc::timeval { tv_sec: 0, tv_usec: 0 };
            let r = ffi::gettimeofday(&mut tv, 0 as *mut _);
            debug_assert_eq!(r, 0);

            // Offset that time with the specified duration
            let abs = Duration::seconds(tv.tv_sec as i64) +
                      Duration::microseconds(tv.tv_usec as i64) +
                      dur;
            let ns = abs.num_nanoseconds().unwrap() as u64;
            let timeout = libc::timespec {
                tv_sec: (ns / 1000000000) as libc::time_t,
                tv_nsec: (ns % 1000000000) as libc::c_long,
            };

            // And wait!
            let r = ffi::pthread_cond_timedwait(self.inner.get(), mutex,
                                                &timeout);
            if r != 0 {
                debug_assert_eq!(r as int, libc::ETIMEDOUT as int);
                false
            } else {
                true
            }
        }
        pub unsafe fn destroy(&self) {
            let r = ffi::pthread_cond_destroy(self.inner.get());
            debug_assert_eq!(r, 0);
        }
    }
}

#[cfg(windows)]
mod imp {
    use std::cell::UnsafeCell;
    use std::os;
    use std::time::Duration;

    use libc::DWORD;
    use libc;
    use sys::ffi;

    pub struct Condvar { inner: UnsafeCell<ffi::CONDITION_VARIABLE> }

    pub const CONDVAR_INIT: Condvar = Condvar {
        inner: UnsafeCell { value: ffi::CONDITION_VARIABLE { ptr: 0 as *mut _ } }
    };

    impl Condvar {
        pub unsafe fn new() -> Condvar { CONDVAR_INIT }

        pub unsafe fn wait(&self, mutex: ffi::LPCRITICAL_SECTION) {
            let r = ffi::SleepConditionVariableCS(self.inner.get(),
                                                  mutex,
                                                  libc::INFINITE);
            debug_assert!(r != 0);
        }

        pub unsafe fn wait_timeout(&self, mutex: ffi::LPCRITICAL_SECTION,
                                   dur: Duration) -> bool {
            let r = ffi::SleepConditionVariableCS(self.inner.get(),
                                                  mutex,
                                                  dur.num_milliseconds() as DWORD);
            if r == 0 {
                const ERROR_TIMEOUT: DWORD = 0x5B4;
                debug_assert_eq!(os::errno() as uint, ERROR_TIMEOUT as uint);
                false
            } else {
                true
            }
        }

        pub unsafe fn signal(&self) {
            ffi::WakeConditionVariable(self.inner.get())
        }

        pub unsafe fn broadcast(&self) {
            ffi::WakeAllConditionVariable(self.inner.get())
        }

        pub unsafe fn destroy(&self) {
            // ...
        }
    }
}

