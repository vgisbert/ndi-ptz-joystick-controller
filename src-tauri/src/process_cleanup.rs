use std::ffi::c_void;
use std::os::windows::process::CommandExt;
use std::process::Command;

const CREATE_NO_WINDOW: u32 = 0x08000000;

#[cfg(target_os = "windows")]
mod win_job {
    use super::*;

    #[link(name = "kernel32")]
    extern "system" {
        fn CreateJobObjectW(lpJobAttributes: *mut c_void, lpName: *const u16) -> *mut c_void;
        fn SetInformationJobObject(
            hJob: *mut c_void,
            JobObjectInformationClass: u32,
            lpJobObjectInformation: *const c_void,
            cbJobObjectInformationLength: u32,
        ) -> i32;
        fn AssignProcessToJobObject(hJob: *mut c_void, hProcess: *mut c_void) -> i32;
        fn GetCurrentProcess() -> *mut c_void;
    }

    const JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE: u32 = 0x2000;
    const JOB_OBJECT_EXTENDED_LIMIT_INFORMATION: u32 = 9;

    #[repr(C)]
    struct JobObjectBasicLimitInformation {
        per_process_user_time_limit: i64,
        per_job_user_time_limit: i64,
        limit_flags: u32,
        minimum_working_set_size: usize,
        maximum_working_set_size: usize,
        active_process_limit: u32,
        affinity: usize,
        priority_class: u32,
        scheduling_class: u32,
    }

    #[repr(C)]
    struct IoCounters {
        read_operation_count: u64,
        write_operation_count: u64,
        other_operation_count: u64,
        read_transfer_count: u64,
        write_transfer_count: u64,
        other_transfer_count: u64,
    }

    #[repr(C)]
    struct JobObjectExtendedLimitInformation {
        basic_limit_information: JobObjectBasicLimitInformation,
        io_info: IoCounters,
        process_memory_limit: usize,
        job_memory_limit: usize,
        peak_process_memory_used: usize,
        peak_job_memory_used: usize,
    }

    pub fn assign_current_process_to_kill_on_close_job() {
        unsafe {
            let job = CreateJobObjectW(std::ptr::null_mut(), std::ptr::null());
            if job.is_null() {
                return;
            }

            let mut info: JobObjectExtendedLimitInformation = std::mem::zeroed();
            info.basic_limit_information.limit_flags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

            let res = SetInformationJobObject(
                job,
                JOB_OBJECT_EXTENDED_LIMIT_INFORMATION,
                &info as *const _ as *const c_void,
                std::mem::size_of::<JobObjectExtendedLimitInformation>() as u32,
            );

            if res != 0 {
                let current_proc = GetCurrentProcess();
                let _ = AssignProcessToJobObject(job, current_proc);
            }
        }
    }
}

/// Executes pre-application cleanup before initializing GUI windows or network sockets.
/// 1. Registers process into a Windows Job Object with KILL_ON_JOB_CLOSE.
/// 2. Cleans up orphaned zombie instances from crashed previous sessions.
pub fn pre_application_cleanup() {
    #[cfg(target_os = "windows")]
    {
        win_job::assign_current_process_to_kill_on_close_job();
    }

    let my_pid = std::process::id();
    let targets = ["NDI-PTZ-Controller.exe", "ndi-ptz-joystick-controller.exe"];

    for exe_name in &targets {
        let filter_pid = format!("PID ne {}", my_pid);
        let _ = Command::new("taskkill")
            .args(["/F", "/T", "/FI", &filter_pid, "/IM", exe_name])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
    }
}

/// Post-application cleanup on clean shutdown.
pub fn post_application_cleanup() {
    // Job Object guarantees all children terminate when this process exits.
}
