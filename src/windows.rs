
use alloc::heap;
use core::atomic;
use core::ptr;
use core::ptr::RawPtr;
use libc::{HANDLE, BOOL, LPSECURITY_ATTRIBUTES, c_void, DWORD, LPCSTR};
use libc;

type LPCRITICAL_SECTION = *mut c_void;
const SPIN_COUNT: DWORD = 4000;
#[cfg(target_arch = "x86")]
const CRIT_SECTION_SIZE: uint = 24;
#[cfg(target_arch = "x86_64")]
const CRIT_SECTION_SIZE: uint = 40;

pub struct Mutex {
    // pointers for the lock/cond handles, atomically updated
    lock: atomic::AtomicUint,
    cond: atomic::AtomicUint,
}

pub const MUTEX_INIT: Mutex = Mutex {
    lock: atomic::INIT_ATOMIC_UINT,
    cond: atomic::INIT_ATOMIC_UINT,
};

impl Mutex {
    pub unsafe fn new() -> Mutex {
        Mutex {
            lock: atomic::AtomicUint::new(init_lock()),
            cond: atomic::AtomicUint::new(init_cond()),
        }
    }
    pub unsafe fn lock(&self) {
        EnterCriticalSection(self.getlock() as LPCRITICAL_SECTION)
    }
    pub unsafe fn trylock(&self) -> bool {
        TryEnterCriticalSection(self.getlock() as LPCRITICAL_SECTION) != 0
    }
    pub unsafe fn unlock(&self) {
        LeaveCriticalSection(self.getlock() as LPCRITICAL_SECTION)
    }

    pub unsafe fn wait(&self) {
        self.unlock();
        WaitForSingleObject(self.getcond() as HANDLE, libc::INFINITE);
        self.lock();
    }

    pub unsafe fn signal(&self) {
        assert!(SetEvent(self.getcond() as HANDLE) != 0);
    }

    /// This function is especially unsafe because there are no guarantees made
    /// that no other thread is currently holding the lock or waiting on the
    /// condition variable contained inside.
    pub unsafe fn destroy(&self) {
        let lock = self.lock.swap(0, atomic::SeqCst);
        let cond = self.cond.swap(0, atomic::SeqCst);
        if lock != 0 { free_lock(lock) }
        if cond != 0 { free_cond(cond) }
    }

    unsafe fn getlock(&self) -> *mut c_void {
        match self.lock.load(atomic::SeqCst) {
            0 => {}
            n => return n as *mut c_void
        }
        let lock = init_lock();
        match self.lock.compare_and_swap(0, lock, atomic::SeqCst) {
            0 => return lock as *mut c_void,
            _ => {}
        }
        free_lock(lock);
        return self.lock.load(atomic::SeqCst) as *mut c_void;
    }

    unsafe fn getcond(&self) -> *mut c_void {
        match self.cond.load(atomic::SeqCst) {
            0 => {}
            n => return n as *mut c_void
        }
        let cond = init_cond();
        match self.cond.compare_and_swap(0, cond, atomic::SeqCst) {
            0 => return cond as *mut c_void,
            _ => {}
        }
        free_cond(cond);
        return self.cond.load(atomic::SeqCst) as *mut c_void;
    }
}

pub unsafe fn init_lock() -> uint {
    let block = heap::allocate(CRIT_SECTION_SIZE, 8) as *mut c_void;
    if block.is_null() { ::alloc::oom() }
    InitializeCriticalSectionAndSpinCount(block, SPIN_COUNT);
    return block as uint;
}

pub unsafe fn init_cond() -> uint {
    return CreateEventA(ptr::null_mut(), libc::FALSE, libc::FALSE,
                        ptr::null()) as uint;
}

pub unsafe fn free_lock(h: uint) {
    DeleteCriticalSection(h as LPCRITICAL_SECTION);
    heap::deallocate(h as *mut u8, CRIT_SECTION_SIZE, 8);
}

pub unsafe fn free_cond(h: uint) {
    let block = h as HANDLE;
    libc::CloseHandle(block);
}

#[allow(non_snake_case)]
extern "system" {
    fn CreateEventA(lpSecurityAttributes: LPSECURITY_ATTRIBUTES,
                    bManualReset: BOOL,
                    bInitialState: BOOL,
                    lpName: LPCSTR) -> HANDLE;
    fn InitializeCriticalSectionAndSpinCount(
                    lpCriticalSection: LPCRITICAL_SECTION,
                    dwSpinCount: DWORD) -> BOOL;
    fn DeleteCriticalSection(lpCriticalSection: LPCRITICAL_SECTION);
    fn EnterCriticalSection(lpCriticalSection: LPCRITICAL_SECTION);
    fn LeaveCriticalSection(lpCriticalSection: LPCRITICAL_SECTION);
    fn TryEnterCriticalSection(lpCriticalSection: LPCRITICAL_SECTION) -> BOOL;
    fn SetEvent(hEvent: HANDLE) -> BOOL;
    fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD;
}
