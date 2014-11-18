use std::kinds::marker;

use sys;

pub struct RWLock { inner: Box<sys::RWLock> }

pub struct StaticRWLock { inner: sys::RWLock }

pub const RWLOCK_INIT: StaticRWLock = StaticRWLock {
    inner: sys::RWLOCK_INIT
};

#[must_use]
pub struct ReadGuard<'a> {
    lock: &'a sys::RWLock,
    marker: marker::NoSend,
}

#[must_use]
pub struct WriteGuard<'a> {
    lock: &'a sys::RWLock,
    marker: marker::NoSend,
}

impl RWLock {
    pub fn new() -> RWLock {
        RWLock { inner: box unsafe { sys::RWLock::new() } }
    }

    pub fn read(&self) -> ReadGuard {
        unsafe { self.inner.read() }
        ReadGuard::new(&*self.inner)
    }

    pub fn try_read(&self) -> Option<ReadGuard> {
        if unsafe { self.inner.try_read() } {
            Some(ReadGuard::new(&*self.inner))
        } else {
            None
        }
    }

    pub fn write(&self) -> WriteGuard {
        unsafe { self.inner.write() }
        WriteGuard::new(&*self.inner)
    }

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
    pub fn read(&'static self) -> ReadGuard {
        unsafe { self.inner.read() }
        ReadGuard::new(&self.inner)
    }

    pub fn try_read(&'static self) -> Option<ReadGuard> {
        if unsafe { self.inner.try_read() } {
            Some(ReadGuard::new(&self.inner))
        } else {
            None
        }
    }

    pub fn write(&'static self) -> WriteGuard {
        unsafe { self.inner.write() }
        WriteGuard::new(&self.inner)
    }

    pub fn try_write(&'static self) -> Option<WriteGuard> {
        if unsafe { self.inner.try_write() } {
            Some(WriteGuard::new(&self.inner))
        } else {
            None
        }
    }

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
