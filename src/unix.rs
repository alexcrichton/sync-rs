#![allow(non_camel_case_types)]

use std::cell::UnsafeCell;
use std::time::Duration;
use libc;

use self::os::{PTHREAD_MUTEX_INITIALIZER, PTHREAD_COND_INITIALIZER,
               pthread_mutex_t, pthread_cond_t};

type pthread_mutexattr_t = libc::c_void;
type pthread_condattr_t = libc::c_void;

pub struct Mutex { inner: UnsafeCell<pthread_mutex_t> }

pub const MUTEX_INIT: Mutex = Mutex {
    inner: UnsafeCell { value: PTHREAD_MUTEX_INITIALIZER },
};

impl Mutex {
    pub unsafe fn new() -> Mutex {
        // Might be moved and address is changing it is better to avoid
        // initialization of potentially opaque OS data before it landed
        Mutex { inner: UnsafeCell::new(PTHREAD_MUTEX_INITIALIZER) }
    }
    pub unsafe fn lock(&self) {
        let r = pthread_mutex_lock(self.inner.get());
        debug_assert_eq!(r, 0);
    }
    pub unsafe fn unlock(&self) {
        let r = pthread_mutex_unlock(self.inner.get());
        debug_assert_eq!(r, 0);
    }
    pub unsafe fn trylock(&self) -> bool {
        pthread_mutex_trylock(self.inner.get()) == 0
    }
    pub unsafe fn destroy(&self) {
        let r = pthread_mutex_destroy(self.inner.get());
        debug_assert_eq!(r, 0);
    }
}

pub struct Condvar { inner: UnsafeCell<pthread_cond_t> }

pub const CONDVAR_INIT: Condvar = Condvar {
    inner: UnsafeCell { value: PTHREAD_COND_INITIALIZER },
};

impl Condvar {
    pub unsafe fn new() -> Condvar {
        // Might be moved and address is changing it is better to avoid
        // initialization of potentially opaque OS data before it landed
        Condvar { inner: UnsafeCell::new(PTHREAD_COND_INITIALIZER) }
    }
    pub unsafe fn signal(&self) {
        let r = pthread_cond_signal(self.inner.get());
        debug_assert_eq!(r, 0);
    }
    pub unsafe fn broadcast(&self) {
        let r = pthread_cond_broadcast(self.inner.get());
        debug_assert_eq!(r, 0);
    }
    pub unsafe fn wait(&self, mutex: &Mutex) {
        let r = pthread_cond_wait(self.inner.get(), mutex.inner.get());
        debug_assert_eq!(r, 0);
    }
    pub unsafe fn wait_timeout(&self, mutex: &Mutex, dur: Duration) -> bool {
        assert!(dur >= Duration::nanoseconds(0));

        // First, figure out what time it currently is
        let mut tv = libc::timeval { tv_sec: 0, tv_usec: 0 };
        let r = gettimeofday(&mut tv, 0 as *mut _);
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
        let r = pthread_cond_timedwait(self.inner.get(), mutex.inner.get(),
                                       &timeout);
        if r != 0 {
            debug_assert_eq!(r as int, libc::ETIMEDOUT as int);
            false
        } else {
            true
        }
    }
    pub unsafe fn destroy(&self) {
        debug_assert_eq!(pthread_cond_destroy(self.inner.get()), 0);
    }
}

extern {
    fn pthread_mutex_destroy(lock: *mut pthread_mutex_t) -> libc::c_int;
    fn pthread_mutex_lock(lock: *mut pthread_mutex_t) -> libc::c_int;
    fn pthread_mutex_trylock(lock: *mut pthread_mutex_t) -> libc::c_int;
    fn pthread_mutex_unlock(lock: *mut pthread_mutex_t) -> libc::c_int;

    fn pthread_cond_wait(cond: *mut pthread_cond_t,
                         lock: *mut pthread_mutex_t) -> libc::c_int;
    fn pthread_cond_timedwait(cond: *mut pthread_cond_t,
                              lock: *mut pthread_mutex_t,
                              abstime: *const libc::timespec) -> libc::c_int;
    fn pthread_cond_signal(cond: *mut pthread_cond_t) -> libc::c_int;
    fn pthread_cond_broadcast(cond: *mut pthread_cond_t) -> libc::c_int;
    fn pthread_cond_destroy(cond: *mut pthread_cond_t) -> libc::c_int;
    fn gettimeofday(tp: *mut libc::timeval,
                    tz: *mut libc::c_void) -> libc::c_int;
}

#[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
mod os {
    use libc;

    pub type pthread_mutex_t = *mut libc::c_void;
    pub type pthread_cond_t = *mut libc::c_void;

    pub const PTHREAD_MUTEX_INITIALIZER: pthread_mutex_t =
        0 as pthread_mutex_t;
    pub const PTHREAD_COND_INITIALIZER: pthread_cond_t =
        0 as pthread_cond_t;
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
mod os {
    use libc;

    #[cfg(target_arch = "x86_64")]
    const __PTHREAD_MUTEX_SIZE__: uint = 56;
    #[cfg(target_arch = "x86_64")]
    const __PTHREAD_COND_SIZE__: uint = 40;
    #[cfg(target_arch = "x86")]
    const __PTHREAD_MUTEX_SIZE__: uint = 40;
    #[cfg(target_arch = "x86")]
    const __PTHREAD_COND_SIZE__: uint = 24;
    #[cfg(target_arch = "arm")]
    const __PTHREAD_MUTEX_SIZE__: uint = 40;
    #[cfg(target_arch = "arm")]
    const __PTHREAD_COND_SIZE__: uint = 24;

    const _PTHREAD_MUTEX_SIG_INIT: libc::c_long = 0x32AAABA7;
    const _PTHREAD_COND_SIG_INIT: libc::c_long = 0x3CB0B1BB;

    #[repr(C)]
    pub struct pthread_mutex_t {
        __sig: libc::c_long,
        __opaque: [u8, ..__PTHREAD_MUTEX_SIZE__],
    }
    #[repr(C)]
    pub struct pthread_cond_t {
        __sig: libc::c_long,
        __opaque: [u8, ..__PTHREAD_COND_SIZE__],
    }

    pub const PTHREAD_MUTEX_INITIALIZER: pthread_mutex_t = pthread_mutex_t {
        __sig: _PTHREAD_MUTEX_SIG_INIT,
        __opaque: [0, ..__PTHREAD_MUTEX_SIZE__],
    };
    pub const PTHREAD_COND_INITIALIZER: pthread_cond_t = pthread_cond_t {
        __sig: _PTHREAD_COND_SIG_INIT,
        __opaque: [0, ..__PTHREAD_COND_SIZE__],
    };
}

#[cfg(target_os = "linux")]
mod os {
    use libc;

    // minus 8 because we have an 'align' field
    #[cfg(target_arch = "x86_64")]
    const __SIZEOF_PTHREAD_MUTEX_T: uint = 40 - 8;
    #[cfg(target_arch = "x86")]
    const __SIZEOF_PTHREAD_MUTEX_T: uint = 24 - 8;
    #[cfg(target_arch = "arm")]
    const __SIZEOF_PTHREAD_MUTEX_T: uint = 24 - 8;
    #[cfg(target_arch = "mips")]
    const __SIZEOF_PTHREAD_MUTEX_T: uint = 24 - 8;
    #[cfg(target_arch = "mipsel")]
    const __SIZEOF_PTHREAD_MUTEX_T: uint = 24 - 8;
    #[cfg(target_arch = "x86_64")]
    const __SIZEOF_PTHREAD_COND_T: uint = 48 - 8;
    #[cfg(target_arch = "x86")]
    const __SIZEOF_PTHREAD_COND_T: uint = 48 - 8;
    #[cfg(target_arch = "arm")]
    const __SIZEOF_PTHREAD_COND_T: uint = 48 - 8;
    #[cfg(target_arch = "mips")]
    const __SIZEOF_PTHREAD_COND_T: uint = 48 - 8;
    #[cfg(target_arch = "mipsel")]
    const __SIZEOF_PTHREAD_COND_T: uint = 48 - 8;

    #[repr(C)]
    pub struct pthread_mutex_t {
        __align: libc::c_longlong,
        size: [u8, ..__SIZEOF_PTHREAD_MUTEX_T],
    }
    #[repr(C)]
    pub struct pthread_cond_t {
        __align: libc::c_longlong,
        size: [u8, ..__SIZEOF_PTHREAD_COND_T],
    }

    pub const PTHREAD_MUTEX_INITIALIZER: pthread_mutex_t = pthread_mutex_t {
        __align: 0,
        size: [0, ..__SIZEOF_PTHREAD_MUTEX_T],
    };
    pub const PTHREAD_COND_INITIALIZER: pthread_cond_t = pthread_cond_t {
        __align: 0,
        size: [0, ..__SIZEOF_PTHREAD_COND_T],
    };
}
#[cfg(target_os = "android")]
mod os {
    use libc;

    #[repr(C)]
    pub struct pthread_mutex_t { value: libc::c_int }
    #[repr(C)]
    pub struct pthread_cond_t { value: libc::c_int }

    pub const PTHREAD_MUTEX_INITIALIZER: pthread_mutex_t = pthread_mutex_t {
        value: 0,
    };
    pub const PTHREAD_COND_INITIALIZER: pthread_cond_t = pthread_cond_t {
        value: 0,
    };
}
