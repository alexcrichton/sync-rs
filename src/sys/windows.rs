use libc::{BOOL, DWORD, c_void, LPVOID};

pub type LPCRITICAL_SECTION = *mut c_void;
pub type LPCONDITION_VARIABLE = *mut CONDITION_VARIABLE;

#[cfg(target_arch = "x86")]
pub const CRITICAL_SECTION_SIZE: uint = 24;
#[cfg(target_arch = "x86_64")]
pub const CRITICAL_SECTION_SIZE: uint = 40;

#[repr(C)]
pub struct CONDITION_VARIABLE { pub ptr: LPVOID }

extern "system" {
    pub fn InitializeCriticalSectionAndSpinCount(
                    lpCriticalSection: LPCRITICAL_SECTION,
                    dwSpinCount: DWORD) -> BOOL;
    pub fn DeleteCriticalSection(lpCriticalSection: LPCRITICAL_SECTION);
    pub fn EnterCriticalSection(lpCriticalSection: LPCRITICAL_SECTION);
    pub fn LeaveCriticalSection(lpCriticalSection: LPCRITICAL_SECTION);
    pub fn TryEnterCriticalSection(lpCriticalSection: LPCRITICAL_SECTION) -> BOOL;

    pub fn SleepConditionVariableCS(ConditionVariable: LPCONDITION_VARIABLE,
                                    CriticalSection: LPCRITICAL_SECTION,
                                    dwMilliseconds: DWORD) -> BOOL;
    pub fn WakeConditionVariable(ConditionVariable: LPCONDITION_VARIABLE);
    pub fn WakeAllConditionVariable(ConditionVariable: LPCONDITION_VARIABLE);
}
