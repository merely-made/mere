// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Linux: a systemd system service under its own dynamic uid (the unit is
//! `packaging/linux`), socket-activated from a 0600 socket owned by the
//! desktop user, with `SO_PEERCRED` principals. Before any input the
//! process sets `no_new_privs`, turns off dumping and core files, installs
//! Landlock (ABI 6 in protected mode: the state directory read-write, the
//! configuration read-only, no execution, no TCP, abstract sockets and
//! signals scoped) and a seccomp allowlist. polkit consent runs `pkcheck`
//! from a helper forked before confinement, so the confined custodian never
//! executes anything.

use std::ffi::CString;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::os::unix::ffi::OsStrExt;

use super::{Applied, Confinement};

const PROTECTED_LANDLOCK_ABI: i64 = 6;

/// Apply the recipe.
pub fn confine(plan: &Confinement) -> Result<Applied, String> {
    let mut applied = Applied::default();
    harden()?;
    applied.notes.push("no_new_privs, not dumpable, no core files".into());
    match landlock(plan) {
        Ok(abi) => applied.notes.push(format!("landlock ABI {abi}")),
        Err(error) if plan.required => return Err(error),
        Err(error) => applied.notes.push(format!("landlock not applied: {error}")),
    }
    seccomp(plan.required)?;
    applied.notes.push(if plan.required {
        "seccomp allowlist enforcing".into()
    } else {
        "seccomp allowlist logging only (development)".into()
    });
    Ok(applied)
}

fn harden() -> Result<(), String> {
    // SAFETY: plain prctl/setrlimit calls with constant arguments.
    unsafe {
        if libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) != 0 {
            return Err(format!("no_new_privs: {}", std::io::Error::last_os_error()));
        }
        if libc::prctl(libc::PR_SET_DUMPABLE, 0, 0, 0, 0) != 0 {
            return Err(format!("not dumpable: {}", std::io::Error::last_os_error()));
        }
        let none = libc::rlimit { rlim_cur: 0, rlim_max: 0 };
        if libc::setrlimit(libc::RLIMIT_CORE, &none) != 0 {
            return Err(format!("no core files: {}", std::io::Error::last_os_error()));
        }
    }
    Ok(())
}

#[repr(C)]
struct RulesetAttr {
    handled_access_fs: u64,
    handled_access_net: u64,
    scoped: u64,
}

#[repr(C, packed)]
struct PathBeneath {
    allowed_access: u64,
    parent_fd: i32,
}

const FS_EXECUTE: u64 = 1 << 0;
const FS_WRITE_FILE: u64 = 1 << 1;
const FS_READ_FILE: u64 = 1 << 2;
const FS_READ_DIR: u64 = 1 << 3;
const FS_REMOVE_DIR: u64 = 1 << 4;
const FS_REMOVE_FILE: u64 = 1 << 5;
const FS_MAKE_DIR: u64 = 1 << 7;
const FS_MAKE_REG: u64 = 1 << 8;
const FS_REFER: u64 = 1 << 13;
const FS_TRUNCATE: u64 = 1 << 14;

fn handled_fs(abi: i64) -> u64 {
    let mut all = (1u64 << 13) - 1;
    if abi >= 2 {
        all |= FS_REFER;
    }
    if abi >= 3 {
        all |= FS_TRUNCATE;
    }
    if abi >= 5 {
        all |= 1 << 15; // IOCTL_DEV
    }
    all
}

fn landlock(plan: &Confinement) -> Result<i64, String> {
    // SAFETY: the version query takes no attribute.
    let abi = unsafe { libc::syscall(libc::SYS_landlock_create_ruleset, std::ptr::null::<RulesetAttr>(), 0usize, 1u32) };
    if abi < 1 {
        return Err("Landlock is unavailable".into());
    }
    if plan.required && abi < PROTECTED_LANDLOCK_ABI {
        return Err(format!("Landlock ABI {abi} is below the protected minimum {PROTECTED_LANDLOCK_ABI}"));
    }
    let handled = handled_fs(abi);
    let attr = RulesetAttr {
        handled_access_fs: handled,
        handled_access_net: if abi >= 4 { 0b11 } else { 0 },
        scoped: if abi >= 6 { 0b11 } else { 0 },
    };
    let size = match abi {
        1..=3 => 8,
        4 | 5 => 16,
        _ => 24,
    };
    // SAFETY: `attr` is valid for `size` bytes, which the ABI understands.
    let ruleset = unsafe { libc::syscall(libc::SYS_landlock_create_ruleset, &attr as *const RulesetAttr, size as usize, 0u32) };
    if ruleset < 0 {
        return Err(format!("create ruleset: {}", std::io::Error::last_os_error()));
    }
    // SAFETY: the kernel returned a fresh descriptor we now own.
    let ruleset = unsafe { OwnedFd::from_raw_fd(ruleset as RawFd) };
    let state = FS_READ_FILE | FS_WRITE_FILE | FS_READ_DIR | FS_REMOVE_FILE | FS_REMOVE_DIR | FS_MAKE_DIR | FS_MAKE_REG | (handled & (FS_REFER | FS_TRUNCATE));
    allow(&ruleset, &plan.state, state & handled)?;
    allow(&ruleset, &plan.config, FS_READ_FILE)?;
    let _ = FS_EXECUTE;
    // SAFETY: a valid ruleset descriptor, no flags.
    if unsafe { libc::syscall(libc::SYS_landlock_restrict_self, ruleset.as_raw_fd(), 0u32) } != 0 {
        return Err(format!("restrict self: {}", std::io::Error::last_os_error()));
    }
    Ok(abi)
}

fn allow(ruleset: &OwnedFd, path: &std::path::Path, access: u64) -> Result<(), String> {
    let name = CString::new(path.as_os_str().as_bytes()).map_err(|_| "path holds a NUL byte".to_string())?;
    // SAFETY: a NUL-terminated path; O_PATH opens without reading.
    let fd = unsafe { libc::open(name.as_ptr(), libc::O_PATH | libc::O_CLOEXEC) };
    if fd < 0 {
        return Err(format!("open {}: {}", path.display(), std::io::Error::last_os_error()));
    }
    // SAFETY: a fresh descriptor we own.
    let fd = unsafe { OwnedFd::from_raw_fd(fd) };
    let rule = PathBeneath {
        allowed_access: access,
        parent_fd: fd.as_raw_fd(),
    };
    // SAFETY: rule type 1 is LANDLOCK_RULE_PATH_BENEATH with this layout.
    if unsafe { libc::syscall(libc::SYS_landlock_add_rule, ruleset.as_raw_fd(), 1u32, &rule as *const PathBeneath, 0u32) } != 0 {
        return Err(format!("rule for {}: {}", path.display(), std::io::Error::last_os_error()));
    }
    Ok(())
}

/// The syscalls the confined custodian makes: threads, the allocator,
/// files under the state directory, accepted sockets, clocks and entropy.
/// No socket creation, connect, execve or ptrace.
fn allowed() -> Vec<libc::c_long> {
    let mut calls = vec![
        libc::SYS_read, libc::SYS_write, libc::SYS_readv, libc::SYS_writev, libc::SYS_pread64,
        libc::SYS_pwrite64, libc::SYS_close, libc::SYS_openat, libc::SYS_fstat, libc::SYS_statx,
        libc::SYS_lseek, libc::SYS_mmap, libc::SYS_munmap, libc::SYS_mprotect, libc::SYS_mremap,
        libc::SYS_madvise, libc::SYS_brk, libc::SYS_rt_sigaction, libc::SYS_rt_sigprocmask,
        libc::SYS_rt_sigreturn, libc::SYS_sigaltstack, libc::SYS_getpid, libc::SYS_gettid,
        libc::SYS_getuid, libc::SYS_geteuid, libc::SYS_getgid, libc::SYS_getegid, libc::SYS_futex,
        libc::SYS_set_robust_list, libc::SYS_rseq, libc::SYS_clone, libc::SYS_clone3, libc::SYS_exit,
        libc::SYS_exit_group, libc::SYS_sched_yield, libc::SYS_sched_getaffinity,
        libc::SYS_clock_nanosleep, libc::SYS_nanosleep, libc::SYS_clock_gettime, libc::SYS_getrandom,
        libc::SYS_accept4, libc::SYS_accept, libc::SYS_recvfrom, libc::SYS_sendto, libc::SYS_recvmsg,
        libc::SYS_sendmsg, libc::SYS_shutdown, libc::SYS_getsockopt, libc::SYS_setsockopt,
        libc::SYS_getsockname, libc::SYS_getpeername, libc::SYS_fcntl, libc::SYS_fsync,
        libc::SYS_fdatasync, libc::SYS_renameat, libc::SYS_renameat2, libc::SYS_unlinkat,
        libc::SYS_mkdirat, libc::SYS_getdents64, libc::SYS_flock, libc::SYS_ftruncate,
        libc::SYS_prctl, libc::SYS_tgkill, libc::SYS_restart_syscall, libc::SYS_newfstatat,
        libc::SYS_ppoll, libc::SYS_membarrier, libc::SYS_prlimit64, libc::SYS_clock_getres,
        // The SSH agent's single-threaded tokio runtime.
        libc::SYS_epoll_create1, libc::SYS_epoll_ctl, libc::SYS_epoll_pwait, libc::SYS_epoll_pwait2,
        libc::SYS_eventfd2,
    ];
    #[cfg(target_arch = "x86_64")]
    calls.extend([
        libc::SYS_stat, libc::SYS_lstat, libc::SYS_rename, libc::SYS_unlink, libc::SYS_mkdir,
        libc::SYS_rmdir, libc::SYS_poll, libc::SYS_arch_prctl, libc::SYS_open,
        libc::SYS_getrlimit, libc::SYS_gettimeofday, libc::SYS_time, libc::SYS_epoll_wait,
    ]);
    calls
}

#[cfg(target_arch = "x86_64")]
const AUDIT_ARCH: u32 = 0xC000_003E;
#[cfg(target_arch = "aarch64")]
const AUDIT_ARCH: u32 = 0xC000_00B7;

const RET_KILL_PROCESS: u32 = 0x8000_0000;
const RET_ERRNO: u32 = 0x0005_0000;
const RET_LOG: u32 = 0x7ffc_0000;
const RET_ALLOW: u32 = 0x7fff_0000;

fn statement(code: u16, k: u32) -> libc::sock_filter {
    libc::sock_filter { code, jt: 0, jf: 0, k }
}

fn jump(k: u32, jt: u8, jf: u8) -> libc::sock_filter {
    libc::sock_filter {
        code: (libc::BPF_JMP | libc::BPF_JEQ | libc::BPF_K) as u16,
        jt,
        jf,
        k,
    }
}

/// An allowlist: the wrong architecture or the x32 ABI kills the process;
/// an unlisted call fails with EPERM, or is only logged in development.
fn seccomp(enforce: bool) -> Result<(), String> {
    let calls = allowed();
    if calls.len() > 250 {
        return Err("seccomp allowlist too long for one jump table".into());
    }
    let load = (libc::BPF_LD | libc::BPF_W | libc::BPF_ABS) as u16;
    let ret = (libc::BPF_RET | libc::BPF_K) as u16;
    let mut program = vec![
        statement(load, 4), // seccomp_data.arch
        jump(AUDIT_ARCH, 1, 0),
        statement(ret, RET_KILL_PROCESS),
        statement(load, 0), // seccomp_data.nr
    ];
    #[cfg(target_arch = "x86_64")]
    {
        program.push(libc::sock_filter {
            code: (libc::BPF_JMP | libc::BPF_JGE | libc::BPF_K) as u16,
            jt: 0,
            jf: 1,
            k: 0x4000_0000,
        });
        program.push(statement(ret, RET_KILL_PROCESS));
    }
    let n = calls.len();
    for (i, call) in calls.iter().enumerate() {
        // Jump over the remaining comparisons and the deny to the allow.
        program.push(jump(*call as u32, (n - i) as u8, 0));
    }
    program.push(statement(ret, if enforce { RET_ERRNO | libc::EPERM as u32 } else { RET_LOG }));
    program.push(statement(ret, RET_ALLOW));
    let filter = libc::sock_fprog {
        len: program.len() as u16,
        filter: program.as_mut_ptr(),
    };
    // SAFETY: a valid filter program; no_new_privs is already set.
    let rc = unsafe { libc::prctl(libc::PR_SET_SECCOMP, libc::SECCOMP_MODE_FILTER, &filter as *const libc::sock_fprog, 0, 0) };
    if rc != 0 {
        return Err(format!("seccomp: {}", std::io::Error::last_os_error()));
    }
    Ok(())
}

/// A polkit petition broker forked before confinement. It holds nothing but
/// its end of a socket pair and knows one thing: run the installed
/// `pkcheck` for the action with the subject and details it is handed.
pub struct PolkitHelper {
    channel: std::sync::Mutex<std::os::unix::net::UnixStream>,
}

impl PolkitHelper {
    /// Fork the helper. Call while single-threaded, before confinement.
    pub fn fork(polkit: crate::consent::Polkit) -> Result<Self, String> {
        let (parent, child) = std::os::unix::net::UnixStream::pair().map_err(|e| e.to_string())?;
        // SAFETY: called before any thread exists; the child only runs code
        // that does not depend on other threads' state.
        match unsafe { libc::fork() } {
            -1 => Err(format!("fork the polkit helper: {}", std::io::Error::last_os_error())),
            0 => {
                drop(parent);
                serve_helper(child, &polkit);
                // SAFETY: leave without running the parent's destructors.
                unsafe { libc::_exit(0) }
            },
            _ => {
                drop(child);
                Ok(Self {
                    channel: std::sync::Mutex::new(parent),
                })
            },
        }
    }
}

fn serve_helper(mut channel: std::os::unix::net::UnixStream, polkit: &crate::consent::Polkit) {
    use crate::consent::Mechanism;
    // The helper keeps no secrets and accepts no other input.
    unsafe {
        libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL);
    }
    while let Ok(Some(prompt)) = custody_wire::frame::read_message::<_, HelperPrompt>(&mut channel) {
        let decided = polkit.present(&prompt.into_prompt());
        if custody_wire::frame::write_message(&mut channel, &decided).is_err() {
            break;
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct HelperPrompt {
    petition: custody_wire::Petition,
    requester: custody_wire::Peer,
}

impl HelperPrompt {
    fn into_prompt(self) -> crate::core::Prompt {
        crate::core::Prompt {
            petition: self.petition,
            requester: self.requester,
        }
    }
}

impl crate::consent::Mechanism for PolkitHelper {
    fn present(&self, prompt: &crate::core::Prompt) -> bool {
        let mut channel = self.channel.lock().unwrap_or_else(|e| e.into_inner());
        let request = HelperPrompt {
            petition: prompt.petition.clone(),
            requester: prompt.requester.clone(),
        };
        if custody_wire::frame::write_message(&mut *channel, &request).is_err() {
            return false;
        }
        matches!(custody_wire::frame::read_message::<_, bool>(&mut *channel), Ok(Some(true)))
    }
}
