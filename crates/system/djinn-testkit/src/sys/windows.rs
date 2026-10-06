// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Win32: process identity, pipe enumeration, the kill-on-close job, load.
//! Nothing here connects to a pipe or acts on a process it did not spawn.

#![allow(unsafe_code)]

use std::os::windows::io::AsRawHandle;
use std::path::PathBuf;
use std::process::Child;
use std::time::Duration;

use windows_sys::Win32::Foundation::{
    CloseHandle, FILETIME, HANDLE, INVALID_HANDLE_VALUE, STILL_ACTIVE,
};
use windows_sys::Win32::Storage::FileSystem::{
    FindClose, FindFirstFileW, FindNextFileW, WIN32_FIND_DATAW,
};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
    SetInformationJobObject,
};
use windows_sys::Win32::System::Performance::{
    PDH_FMT_COUNTERVALUE, PDH_FMT_LONG, PdhAddEnglishCounterW, PdhCloseQuery, PdhCollectQueryData,
    PdhGetFormattedCounterValue, PdhOpenQueryW,
};
use windows_sys::Win32::System::Threading::{
    GetExitCodeProcess, GetProcessTimes, GetSystemTimes, OpenProcess, PROCESS_NAME_WIN32,
    PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};

use super::ProcessEntry;

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}

fn from_wide(buffer: &[u16]) -> String {
    let end = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
    String::from_utf16_lossy(&buffer[..end])
}

fn filetime(time: FILETIME) -> u64 {
    (u64::from(time.dwHighDateTime) << 32) | u64::from(time.dwLowDateTime)
}

struct Owned(HANDLE);

impl Drop for Owned {
    fn drop(&mut self) {
        if !self.0.is_null() && self.0 != INVALID_HANDLE_VALUE {
            unsafe { CloseHandle(self.0) };
        }
    }
}

/// Every process: id, parent, image name, and where it can be read, its full
/// path and creation time.
pub fn processes() -> Vec<ProcessEntry> {
    let snapshot = Owned(unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) });
    if snapshot.0 == INVALID_HANDLE_VALUE {
        return Vec::new();
    }
    let mut entry = PROCESSENTRY32W {
        dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };
    let mut found = Vec::new();
    let mut more = unsafe { Process32FirstW(snapshot.0, &mut entry) } != 0;
    while more {
        let pid = entry.th32ProcessID;
        let (path, started) = identity(pid).unwrap_or((None, None));
        found.push(ProcessEntry {
            pid,
            parent: entry.th32ParentProcessID,
            name: from_wide(&entry.szExeFile),
            path,
            started,
        });
        more = unsafe { Process32NextW(snapshot.0, &mut entry) } != 0;
    }
    found
}

fn identity(pid: u32) -> Option<(Option<PathBuf>, Option<u64>)> {
    let process = Owned(unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) });
    if process.0.is_null() {
        return None;
    }
    let mut buffer = [0u16; 1024];
    let mut size = buffer.len() as u32;
    let path = (unsafe {
        QueryFullProcessImageNameW(
            process.0,
            PROCESS_NAME_WIN32,
            buffer.as_mut_ptr(),
            &mut size,
        )
    } != 0)
        .then(|| PathBuf::from(from_wide(&buffer[..size as usize])));
    let zero = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    let (mut created, mut exited, mut kernel, mut user) = (zero, zero, zero, zero);
    let started =
        (unsafe { GetProcessTimes(process.0, &mut created, &mut exited, &mut kernel, &mut user) }
            != 0)
            .then(|| filetime(created));
    Some((path, started))
}

/// The creation time of `pid`, when it is still running.
pub fn started(pid: u32) -> Option<u64> {
    let process = Owned(unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) });
    if process.0.is_null() {
        return None;
    }
    let mut code = 0u32;
    if unsafe { GetExitCodeProcess(process.0, &mut code) } == 0 || code != STILL_ACTIVE as u32 {
        return None;
    }
    identity(pid).and_then(|(_, started)| started)
}

/// The names under `\\.\pipe\`, enumerated. No pipe is opened.
pub fn pipe_names() -> Vec<String> {
    let pattern = wide(r"\\.\pipe\*");
    let mut data: WIN32_FIND_DATAW = unsafe { std::mem::zeroed() };
    let find = unsafe { FindFirstFileW(pattern.as_ptr(), &mut data) };
    if find == INVALID_HANDLE_VALUE {
        return Vec::new();
    }
    let mut names = vec![from_wide(&data.cFileName)];
    while unsafe { FindNextFileW(find, &mut data) } != 0 {
        names.push(from_wide(&data.cFileName));
    }
    unsafe { FindClose(find) };
    names
}

/// A job whose processes die when its last handle closes, including when the
/// process holding it is killed.
pub struct Job(HANDLE);

// The handle is a kernel object usable from any thread.
unsafe impl Send for Job {}
unsafe impl Sync for Job {}

impl Job {
    pub fn new() -> std::io::Result<Self> {
        let handle = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
        if handle.is_null() {
            return Err(std::io::Error::last_os_error());
        }
        let job = Self(handle);
        let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        let set = unsafe {
            SetInformationJobObject(
                job.0,
                JobObjectExtendedLimitInformation,
                (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        };
        if set == 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(job)
    }

    pub fn adopt(&self, child: &Child) -> std::io::Result<()> {
        let process = child.as_raw_handle() as HANDLE;
        if unsafe { AssignProcessToJobObject(self.0, process) } == 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(())
    }
}

impl Drop for Job {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.0) };
    }
}

/// `\System\Processor Queue Length`, one sample.
pub fn cpu_queue() -> Option<f64> {
    let mut query = std::ptr::null_mut();
    if unsafe { PdhOpenQueryW(std::ptr::null(), 0, &mut query) } != 0 {
        return None;
    }
    let path = wide(r"\System\Processor Queue Length");
    let mut counter = std::ptr::null_mut();
    let mut value: PDH_FMT_COUNTERVALUE = unsafe { std::mem::zeroed() };
    let read = unsafe {
        PdhAddEnglishCounterW(query, path.as_ptr(), 0, &mut counter) == 0
            && PdhCollectQueryData(query) == 0
            && PdhGetFormattedCounterValue(counter, PDH_FMT_LONG, std::ptr::null_mut(), &mut value)
                == 0
    };
    unsafe { PdhCloseQuery(query) };
    read.then(|| f64::from(unsafe { value.Anonymous.longValue }))
}

/// Busy share of all processors over `window`, from `GetSystemTimes`.
pub fn cpu_busy(window: Duration) -> Option<f64> {
    let sample = || {
        let zero = FILETIME {
            dwLowDateTime: 0,
            dwHighDateTime: 0,
        };
        let (mut idle, mut kernel, mut user) = (zero, zero, zero);
        (unsafe { GetSystemTimes(&mut idle, &mut kernel, &mut user) } != 0)
            .then(|| (filetime(idle), filetime(kernel) + filetime(user)))
    };
    let (idle0, total0) = sample()?;
    std::thread::sleep(window);
    let (idle1, total1) = sample()?;
    let total = total1.saturating_sub(total0);
    (total > 0).then(|| 1.0 - idle1.saturating_sub(idle0) as f64 / total as f64)
}
