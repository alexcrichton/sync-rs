#![allow(non_camel_case_types)]

use std::cell::UnsafeCell;
use std::os;
use std::sync::atomic;
use std::time::Duration;
use alloc::{mod, heap};

use libc::{BOOL, c_void, DWORD};
use libc;

type LPCRITICAL_SECTION = *mut c_void;
type LPCONDITION_VARIABLE = *mut CONDITION_VARIABLE;

const SPIN_COUNT: DWORD = 4000;

#[cfg(target_arch = "x86")]
const CRITICAL_SECTION_SIZE: uint = 24;
#[cfg(target_arch = "x86_64")]
const CRITICAL_SECTION_SIZE: uint = 40;

#[repr(C)]
struct CONDITION_VARIABLE { ptr: libc::LPVOID }

pub struct Mutex { inner: atomic::AtomicUint }

pub struct Condvar { inner: UnsafeCell<CONDITION_VARIABLE> }

pub const MUTEX_INIT: Mutex = Mutex { inner: atomic::INIT_ATOMIC_UINT };

pub const CONDVAR_INIT: Condvar = Condvar {
    inner: UnsafeCell { value: CONDITION_VARIABLE { ptr: 0 as *mut _ } }
};

impl Mutex {
    pub unsafe fn new() -> Mutex {
        Mutex { inner: atomic::AtomicUint::new(init_lock() as uint) }
    }
    pub unsafe fn lock(&self) {
        EnterCriticalSection(self.get())
    }
    pub unsafe fn trylock(&self) -> bool {
        TryEnterCriticalSection(self.get()) != 0
    }
    pub unsafe fn unlock(&self) {
        LeaveCriticalSection(self.get())
    }
    pub unsafe fn destroy(&self) {
        let lock = self.inner.swap(0, atomic::SeqCst);
        if lock != 0 { free_lock(lock as LPCRITICAL_SECTION) }
    }

    unsafe fn get(&self) -> LPCRITICAL_SECTION {
        match self.inner.load(atomic::SeqCst) {
            0 => {}
            n => return n as LPCRITICAL_SECTION
        }
        let lock = init_lock();
        match self.inner.compare_and_swap(0, lock as uint, atomic::SeqCst) {
            0 => return lock as LPCRITICAL_SECTION,
            _ => {}
        }
        free_lock(lock);
        return self.inner.load(atomic::SeqCst) as LPCRITICAL_SECTION;
    }
}

impl Condvar {
    pub unsafe fn new() -> Condvar { CONDVAR_INIT }

    pub unsafe fn wait(&self, mutex: &Mutex) {
        let r = SleepConditionVariableCS(self.inner.get(),
                                         mutex.get(),
                                         libc::INFINITE);
        debug_assert!(r != 0);
    }

    pub unsafe fn wait_timeout(&self, mutex: &Mutex, dur: Duration) -> bool {
        let r = SleepConditionVariableCS(self.inner.get(),
                                         mutex.get(),
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
        WakeConditionVariable(self.inner.get())
    }

    pub unsafe fn broadcast(&self) {
        WakeAllConditionVariable(self.inner.get())
    }

    pub unsafe fn destroy(&self) {
        // ...
    }
}

unsafe fn init_lock() -> LPCRITICAL_SECTION {
    let block = heap::allocate(CRITICAL_SECTION_SIZE, 8) as LPCRITICAL_SECTION;
    if block.is_null() { alloc::oom() }
    InitializeCriticalSectionAndSpinCount(block, SPIN_COUNT);
    return block;
}

unsafe fn free_lock(h: LPCRITICAL_SECTION) {
    DeleteCriticalSection(h);
    heap::deallocate(h as *mut _, CRITICAL_SECTION_SIZE, 8);
}

extern "system" {
    fn InitializeCriticalSectionAndSpinCount(
                    lpCriticalSection: LPCRITICAL_SECTION,
                    dwSpinCount: DWORD) -> BOOL;
    fn DeleteCriticalSection(lpCriticalSection: LPCRITICAL_SECTION);
    fn EnterCriticalSection(lpCriticalSection: LPCRITICAL_SECTION);
    fn LeaveCriticalSection(lpCriticalSection: LPCRITICAL_SECTION);
    fn TryEnterCriticalSection(lpCriticalSection: LPCRITICAL_SECTION) -> BOOL;

    fn SleepConditionVariableCS(ConditionVariable: LPCONDITION_VARIABLE,
                                CriticalSection: LPCRITICAL_SECTION,
                                dwMilliseconds: DWORD) -> BOOL;
    fn WakeConditionVariable(ConditionVariable: LPCONDITION_VARIABLE);
    fn WakeAllConditionVariable(ConditionVariable: LPCONDITION_VARIABLE);
}
