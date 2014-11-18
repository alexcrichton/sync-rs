use std::time::Duration;

use mutex;
use sys;

pub struct Condvar { inner: Box<sys::Condvar> }

pub struct StaticCondvar { inner: sys::Condvar }

pub const CONDVAR_INIT: StaticCondvar = StaticCondvar {
    inner: sys::CONDVAR_INIT
};

impl Condvar {
    pub fn new() -> Condvar {
        Condvar { inner: box unsafe { sys::Condvar::new() } }
    }

    pub unsafe fn wait(&self, guard: &mutex::Guard) {
        self.inner.wait(mutex::guard_inner(guard))
    }

    pub unsafe fn wait_timeout(&self, guard: &mutex::Guard,
                               dur: Duration) -> bool {
        self.inner.wait_timeout(mutex::guard_inner(guard), dur)
    }

    pub fn signal(&self) { unsafe { self.inner.signal() } }

    pub fn broadcast(&self) { unsafe { self.inner.broadcast() } }
}

impl Drop for Condvar {
    fn drop(&mut self) {
        unsafe { self.inner.destroy() }
    }
}

impl StaticCondvar {
    pub unsafe fn wait(&'static self, guard: &mutex::Guard) {
        self.inner.wait(mutex::guard_inner(guard))
    }

    pub unsafe fn wait_timeout(&self, guard: &mutex::Guard,
                               dur: Duration) -> bool {
        self.inner.wait_timeout(mutex::guard_inner(guard), dur)
    }

    pub fn signal(&'static self) { unsafe { self.inner.signal() } }

    pub fn broadcast(&'static self) { unsafe { self.inner.broadcast() } }

    pub unsafe fn destroy(&'static self) {
        self.inner.destroy()
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
        unsafe { C.wait(&g); }
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
        unsafe { C.wait(&g); }
        drop(g);
        unsafe { C.destroy(); M.destroy(); }
    }

    #[test]
    fn wait_timeout() {
        static C: StaticCondvar = CONDVAR_INIT;
        static M: StaticMutex = MUTEX_INIT;

        let g = M.lock();
        unsafe {
            assert!(!C.wait_timeout(&g, Duration::nanoseconds(1000)));
        }
        spawn(proc() {
            let _g = M.lock();
            C.signal();
        });
        unsafe {
            assert!(C.wait_timeout(&g, Duration::days(1)));
        }
        drop(g);
        unsafe { C.destroy(); M.destroy(); }
    }
}
