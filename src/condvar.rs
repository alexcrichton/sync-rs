use std::sync::atomic::{mod, AtomicUint};
use std::time::Duration;

use mutex;
use sys;

pub struct Condvar { inner: Box<StaticCondvar> }

pub struct StaticCondvar {
    inner: sys::Condvar,
    mutex: AtomicUint,
}

pub const CONDVAR_INIT: StaticCondvar = StaticCondvar {
    inner: sys::CONDVAR_INIT,
    mutex: atomic::INIT_ATOMIC_UINT,
};

impl Condvar {
    pub fn new() -> Condvar {
        Condvar {
            inner: box StaticCondvar {
                inner: unsafe { sys::Condvar::new() },
                mutex: AtomicUint::new(0),
            }
        }
    }

    pub fn wait(&self, guard: &mutex::Guard) {
        unsafe {
            let me: &'static Condvar = &*(self as *const _);
            me.inner.wait(guard)
        }
    }

    pub fn wait_timeout(&self, guard: &mutex::Guard,
                               dur: Duration) -> bool {
        unsafe {
            let me: &'static Condvar = &*(self as *const _);
            me.inner.wait_timeout(guard, dur)
        }
    }

    pub fn signal(&self) { unsafe { self.inner.inner.signal() } }

    pub fn broadcast(&self) { unsafe { self.inner.inner.broadcast() } }
}

impl Drop for Condvar {
    fn drop(&mut self) {
        unsafe { self.inner.inner.destroy() }
    }
}

impl StaticCondvar {
    pub fn wait(&'static self, guard: &mutex::Guard) {
        unsafe {
            self.verify(guard);
            self.inner.wait(mutex::guard_inner(guard))
        }
    }

    pub fn wait_timeout(&self, guard: &mutex::Guard,
                               dur: Duration) -> bool {
        unsafe {
            self.verify(guard);
            self.inner.wait_timeout(mutex::guard_inner(guard), dur)
        }
    }

    pub fn signal(&'static self) { unsafe { self.inner.signal() } }

    pub fn broadcast(&'static self) { unsafe { self.inner.broadcast() } }

    pub unsafe fn destroy(&'static self) {
        self.inner.destroy()
    }

    fn verify(&self, guard: &mutex::Guard) {
        let addr = guard as *const _ as uint;
        if self.mutex.load(atomic::SeqCst) != addr {
            match self.mutex.compare_and_swap(0, addr, atomic::SeqCst) {
                0 => {}
                _ => panic!("attempted to use a condition variable with two \
                             mutexes"),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use super::{Condvar, StaticCondvar, CONDVAR_INIT};
    use mutex::{StaticMutex, MUTEX_INIT};

    #[test]
    fn smoke() {
        let c = Condvar::new();
        c.signal();
        c.broadcast();
    }

    #[test]
    fn static_smoke() {
        static C: StaticCondvar = CONDVAR_INIT;
        C.signal();
        C.broadcast();
        unsafe { C.destroy(); }
    }

    #[test]
    fn signal() {
        static C: StaticCondvar = CONDVAR_INIT;
        static M: StaticMutex = MUTEX_INIT;

        let g = M.lock();
        spawn(proc() {
            let _g = M.lock();
            C.signal();
        });
        C.wait(&g);
        drop(g);
        unsafe { C.destroy(); M.destroy(); }
    }

    #[test]
    fn broadcast() {
        static C: StaticCondvar = CONDVAR_INIT;
        static M: StaticMutex = MUTEX_INIT;

        let g = M.lock();
        spawn(proc() {
            let _g = M.lock();
            C.broadcast();
        });
        C.wait(&g);
        drop(g);
        unsafe { C.destroy(); M.destroy(); }
    }

    #[test]
    fn wait_timeout() {
        static C: StaticCondvar = CONDVAR_INIT;
        static M: StaticMutex = MUTEX_INIT;

        let g = M.lock();
        assert!(!C.wait_timeout(&g, Duration::nanoseconds(1000)));
        spawn(proc() {
            let _g = M.lock();
            C.signal();
        });
        assert!(C.wait_timeout(&g, Duration::days(1)));
        drop(g);
        unsafe { C.destroy(); M.destroy(); }
    }

    #[test]
    #[should_fail]
    fn two_mutexes() {
        static M1: StaticMutex = MUTEX_INIT;
        static M2: StaticMutex = MUTEX_INIT;
        static C: StaticCondvar = CONDVAR_INIT;

        let g = M1.lock();
        spawn(proc() {
            let _g = M1.lock();
            C.signal();
        });
        C.wait(&g);
        drop(g);

        C.wait(&M2.lock());

    }
}

