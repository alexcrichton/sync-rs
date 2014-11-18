use libc;

pub use self::os::{PTHREAD_MUTEX_INITIALIZER, PTHREAD_COND_INITIALIZER,
                   pthread_mutex_t, pthread_cond_t};

pub type pthread_mutexattr_t = libc::c_void;
pub type pthread_condattr_t = libc::c_void;

extern {
    pub fn pthread_mutex_destroy(lock: *mut pthread_mutex_t) -> libc::c_int;
    pub fn pthread_mutex_lock(lock: *mut pthread_mutex_t) -> libc::c_int;
    pub fn pthread_mutex_trylock(lock: *mut pthread_mutex_t) -> libc::c_int;
    pub fn pthread_mutex_unlock(lock: *mut pthread_mutex_t) -> libc::c_int;

    pub fn pthread_cond_wait(cond: *mut pthread_cond_t,
                             lock: *mut pthread_mutex_t) -> libc::c_int;
    pub fn pthread_cond_timedwait(cond: *mut pthread_cond_t,
                              lock: *mut pthread_mutex_t,
                              abstime: *const libc::timespec) -> libc::c_int;
    pub fn pthread_cond_signal(cond: *mut pthread_cond_t) -> libc::c_int;
    pub fn pthread_cond_broadcast(cond: *mut pthread_cond_t) -> libc::c_int;
    pub fn pthread_cond_destroy(cond: *mut pthread_cond_t) -> libc::c_int;
    pub fn gettimeofday(tp: *mut libc::timeval,
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
