use std::time::Duration;

use sys::{mutex, Mutex};

/// An OS-based condition variable.
///
/// This structure is the lowest layer possible on top of the OS-provided
/// condition variables. It is consequently entirely unsafe to use. It is
/// recommended to use the safer types at the top level of this crate instead of
/// this type.
pub struct Condvar(imp::Condvar);

/// Static initializer for condition variables.
pub const CONDVAR_INIT: Condvar = Condvar(imp::CONDVAR_INIT);

impl Condvar {
    /// Creates a new condition variable for use.
    ///
    /// Behavior is undefined if the condition variable is moved after it is
    /// first used with any of the functions below.
    #[inline]
    pub unsafe fn new() -> Condvar { Condvar(imp::Condvar::new()) }

    /// Signal one waiter on this condition variable to wake up.
    #[inline]
    pub unsafe fn notify_one(&self) { self.0.notify_one() }

    /// Awaken all current waiters on this condition variable.
    #[inline]
    pub unsafe fn notify_all(&self) { self.0.notify_all() }

    /// Wait for a signal on the specified mutex.
    ///
    /// Behavior is undefined if the mutex is not locked by the current thread.
    /// Behavior is also undefined if more than one mutex is used concurrently
    /// on this condition variable.
    #[inline]
    pub unsafe fn wait(&self, mutex: &Mutex) { self.0.wait(mutex::raw(mutex)) }

    /// Wait for a signal on the specified mutex with a timeout duration
    /// specified by `dur` (a relative time into the future).
    ///
    /// Behavior is undefined if the mutex is not locked by the current thread.
    /// Behavior is also undefined if more than one mutex is used concurrently
    /// on this condition variable.
    #[inline]
    pub unsafe fn wait_timeout(&self, mutex: &Mutex, dur: Duration) -> bool {
        self.0.wait_timeout(mutex::raw(mutex), dur)
    }

    /// Deallocate all resources associated with this condition variable.
    ///
    /// Behavior is undefined if there are current or will be future users of
    /// this condition variable.
    #[inline]
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
        #[inline]
        pub unsafe fn new() -> Condvar {
            // Might be moved and address is changing it is better to avoid
            // initialization of potentially opaque OS data before it landed
            Condvar { inner: UnsafeCell::new(ffi::PTHREAD_COND_INITIALIZER) }
        }

        #[inline]
        pub unsafe fn notify_one(&self) {
            let r = ffi::pthread_cond_signal(self.inner.get());
            debug_assert_eq!(r, 0);
        }

        #[inline]
        pub unsafe fn notify_all(&self) {
            let r = ffi::pthread_cond_broadcast(self.inner.get());
            debug_assert_eq!(r, 0);
        }

        #[inline]
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

        #[inline]
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
        inner: UnsafeCell { value: ffi::CONDITION_VARIABLE_INIT }
    };

    impl Condvar {
        #[inline]
        pub unsafe fn new() -> Condvar { CONDVAR_INIT }

        #[inline]
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

        #[inline]
        pub unsafe fn notify_one(&self) {
            ffi::WakeConditionVariable(self.inner.get())
        }

        #[inline]
        pub unsafe fn notify_all(&self) {
            ffi::WakeAllConditionVariable(self.inner.get())
        }

        pub unsafe fn destroy(&self) {
            // ...
        }
    }
}

