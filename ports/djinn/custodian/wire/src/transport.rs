// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The transport trait the contract runs over, and its two OS transports:
//! a Unix-domain socket (0600, or a systemd-activated socket owned by the
//! desktop user at 0600) and a Windows named pipe whose DACL admits only
//! the vault owner and whose peers are checked by token SID. Each accepted
//! or connected stream carries its OS-established [`Peer`]. Blocking I/O
//! with a watchdog that shuts the stream down replaces read timeouts, so
//! one deadline mechanism serves both platforms.

use std::io::{Read, Write};

use crate::peer::Peer;

/// One full-duplex local stream.
pub trait Duplex: Read + Write + Send + Sync {
    /// A second handle on the same stream, for a reader thread.
    fn try_clone_duplex(&self) -> std::io::Result<Box<dyn Duplex>>;
    /// End both directions now; a blocked read on any handle returns.
    fn shutdown_both(&self);
}

/// An accepted or connected stream and who is on the other end.
pub struct Connection {
    pub stream: Box<dyn Duplex>,
    pub peer: Peer,
}

/// The server side: accept one connection at a time.
pub trait Listener: Send {
    fn accept(&self) -> std::io::Result<Connection>;
}

/// The client side: connect, reporting the server's OS identity.
pub trait Connector: Send + Sync {
    fn connect(&self) -> std::io::Result<Connection>;
}

#[cfg(unix)]
pub use unix::{UnixSocketConnector, UnixSocketListener};

#[cfg(unix)]
mod unix {
    use std::os::fd::{AsRawFd, FromRawFd};
    use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};
    use std::os::unix::net::{UnixListener, UnixStream};
    use std::path::{Path, PathBuf};

    use super::{Connection, Connector, Duplex, Listener};
    use crate::peer::{effective_uid, unix_peer};

    impl Duplex for UnixStream {
        fn try_clone_duplex(&self) -> std::io::Result<Box<dyn Duplex>> {
            Ok(Box::new(self.try_clone()?))
        }

        fn shutdown_both(&self) {
            let _ = self.shutdown(std::net::Shutdown::Both);
        }
    }

    /// A Unix-domain listener at an owner-only path.
    pub struct UnixSocketListener {
        listener: UnixListener,
        path: Option<PathBuf>,
    }

    impl UnixSocketListener {
        /// Bind `path` at 0600. A stale socket owned by this user with no
        /// listener is replaced; anything else at the path is refused, so a
        /// second custodian fails loudly instead of orphaning the first.
        pub fn bind(path: &Path) -> std::io::Result<Self> {
            if let Some(parent) = path.parent() {
                let meta = std::fs::metadata(parent)?;
                if meta.uid() != effective_uid() || meta.mode() & 0o022 != 0 {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        "the socket directory must be owned by the custodian and not group- or world-writable",
                    ));
                }
            }
            match std::fs::symlink_metadata(path) {
                Ok(meta) => {
                    if !meta.file_type().is_socket() || meta.uid() != effective_uid() {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::AlreadyExists,
                            "the endpoint path holds something other than this user's socket",
                        ));
                    }
                    match UnixStream::connect(path) {
                        Ok(_) => {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::AddrInUse,
                                "another custodian serves this endpoint",
                            ));
                        },
                        Err(error) if error.kind() == std::io::ErrorKind::ConnectionRefused => {
                            std::fs::remove_file(path)?;
                        },
                        Err(error) => return Err(error),
                    }
                },
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {},
                Err(error) => return Err(error),
            }
            let listener = UnixListener::bind(path)?;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
            Ok(Self {
                listener,
                path: Some(path.to_path_buf()),
            })
        }

        /// The socket systemd passed (`LISTEN_PID`/`LISTEN_FDS`, fd 3), if
        /// this process was socket-activated. The unit owns its path and
        /// mode; the custodian does not need to reach the path at all.
        pub fn from_systemd() -> std::io::Result<Option<Self>> {
            let pid_matches = std::env::var("LISTEN_PID")
                .ok()
                .and_then(|pid| pid.parse::<u32>().ok())
                == Some(std::process::id());
            let one_fd = std::env::var("LISTEN_FDS").ok().as_deref() == Some("1");
            if !pid_matches || !one_fd {
                return Ok(None);
            }
            const SD_LISTEN_FDS_START: i32 = 3;
            let mut kind: libc::c_int = 0;
            let mut accepting: libc::c_int = 0;
            let mut len = std::mem::size_of::<libc::c_int>() as libc::socklen_t;
            // SAFETY: valid out-pointers of the stated sizes on an fd systemd
            // promised to this pid.
            let ok = unsafe {
                libc::getsockopt(SD_LISTEN_FDS_START, libc::SOL_SOCKET, libc::SO_TYPE, (&mut kind as *mut libc::c_int).cast(), &mut len) == 0
                    && libc::getsockopt(SD_LISTEN_FDS_START, libc::SOL_SOCKET, libc::SO_ACCEPTCONN, (&mut accepting as *mut libc::c_int).cast(), &mut len) == 0
                    && libc::fcntl(SD_LISTEN_FDS_START, libc::F_SETFD, libc::FD_CLOEXEC) == 0
            };
            if !ok || kind != libc::SOCK_STREAM || accepting == 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "the activated descriptor is not a listening stream socket",
                ));
            }
            // SAFETY: systemd transferred ownership of fd 3 to this process.
            let listener = unsafe { UnixListener::from_raw_fd(SD_LISTEN_FDS_START) };
            Ok(Some(Self {
                listener,
                path: None,
            }))
        }

        /// Wrap an already-bound listener (tests).
        pub fn from_listener(listener: UnixListener) -> Self {
            Self {
                listener,
                path: None,
            }
        }
    }

    impl Drop for UnixSocketListener {
        fn drop(&mut self) {
            if let Some(path) = &self.path {
                let _ = std::fs::remove_file(path);
            }
        }
    }

    impl Listener for UnixSocketListener {
        fn accept(&self) -> std::io::Result<Connection> {
            let (stream, _) = self.listener.accept()?;
            let peer = unix_peer(stream.as_raw_fd())?;
            Ok(Connection {
                stream: Box::new(stream),
                peer,
            })
        }
    }

    /// Connect to a custodian's Unix-domain socket.
    pub struct UnixSocketConnector {
        path: PathBuf,
    }

    impl UnixSocketConnector {
        pub fn new(path: impl Into<PathBuf>) -> Self {
            Self { path: path.into() }
        }
    }

    impl Connector for UnixSocketConnector {
        fn connect(&self) -> std::io::Result<Connection> {
            let stream = UnixStream::connect(&self.path)?;
            let peer = unix_peer(stream.as_raw_fd())?;
            Ok(Connection {
                stream: Box::new(stream),
                peer,
            })
        }
    }
}

#[cfg(windows)]
pub use windows::{NamedPipeConnector, NamedPipeListener};

#[cfg(windows)]
pub(crate) mod windows {
    use std::fs::File;
    use std::os::windows::io::{AsRawHandle, FromRawHandle, RawHandle};

    use windows_sys::Win32::Foundation::{
        CloseHandle, ERROR_INSUFFICIENT_BUFFER, ERROR_PIPE_BUSY, ERROR_PIPE_CONNECTED,
        GENERIC_READ, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE, LocalFree,
    };
    use windows_sys::Win32::Security::Authorization::{
        ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW,
        SDDL_REVISION_1,
    };
    use windows_sys::Win32::Security::{
        GetTokenInformation, LookupAccountNameW, PSID, SECURITY_ATTRIBUTES, SID_NAME_USE,
        TOKEN_QUERY, TOKEN_USER, TokenUser,
    };
    use windows_sys::Win32::Storage::FileSystem::{
        CreateFileW, FILE_FLAG_FIRST_PIPE_INSTANCE, OPEN_EXISTING, PIPE_ACCESS_DUPLEX,
        SECURITY_IDENTIFICATION, SECURITY_SQOS_PRESENT,
    };
    use windows_sys::Win32::System::Pipes::{
        ConnectNamedPipe, CreateNamedPipeW, DisconnectNamedPipe, GetNamedPipeClientProcessId,
        GetNamedPipeServerProcessId, PIPE_READMODE_BYTE, PIPE_REJECT_REMOTE_CLIENTS,
        PIPE_TYPE_BYTE, PIPE_UNLIMITED_INSTANCES, PIPE_WAIT, WaitNamedPipeW,
    };
    use windows_sys::Win32::System::Threading::{
        GetCurrentProcess, OpenProcess, OpenProcessToken, PROCESS_QUERY_LIMITED_INFORMATION,
    };

    use super::{Connection, Connector, Duplex, Listener};
    use crate::peer::Peer;

    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().chain(std::iter::once(0)).collect()
    }

    struct Owned(HANDLE);

    impl Drop for Owned {
        fn drop(&mut self) {
            if !self.0.is_null() && self.0 != INVALID_HANDLE_VALUE {
                // SAFETY: this wrapper owns the handle.
                unsafe { CloseHandle(self.0) };
            }
        }
    }

    /// A connected pipe end. Byte-mode synchronous pipe handles read and
    /// write like files.
    struct PipeStream {
        file: File,
        server: bool,
    }

    impl std::io::Read for PipeStream {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            match self.file.read(buf) {
                // A closed pipe is the end of the stream.
                Err(error) if error.raw_os_error() == Some(109) => Ok(0),
                other => other,
            }
        }
    }

    impl std::io::Write for PipeStream {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.file.write(buf)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            self.file.flush()
        }
    }

    impl Duplex for PipeStream {
        fn try_clone_duplex(&self) -> std::io::Result<Box<dyn Duplex>> {
            Ok(Box::new(PipeStream {
                file: self.file.try_clone()?,
                server: self.server,
            }))
        }

        fn shutdown_both(&self) {
            let handle = self.file.as_raw_handle() as HANDLE;
            // SAFETY: the handle is live for this stream's lifetime.
            unsafe {
                windows_sys::Win32::System::IO::CancelIoEx(handle, std::ptr::null());
                if self.server {
                    DisconnectNamedPipe(handle);
                }
            }
        }
    }

    /// The SID of a process's token user, as an SDDL string.
    fn process_sid(process: HANDLE) -> std::io::Result<String> {
        let mut token: HANDLE = std::ptr::null_mut();
        // SAFETY: valid out-pointer; the token is closed by `Owned`.
        if unsafe { OpenProcessToken(process, TOKEN_QUERY, &mut token) } == 0 {
            return Err(std::io::Error::last_os_error());
        }
        let token = Owned(token);
        let mut len = 0u32;
        // SAFETY: a size query with a null buffer.
        unsafe { GetTokenInformation(token.0, TokenUser, std::ptr::null_mut(), 0, &mut len) };
        if len == 0 {
            return Err(std::io::Error::last_os_error());
        }
        let mut buffer = vec![0usize; (len as usize).div_ceil(std::mem::size_of::<usize>())];
        // SAFETY: `buffer` holds at least `len` bytes, suitably aligned.
        if unsafe { GetTokenInformation(token.0, TokenUser, buffer.as_mut_ptr().cast(), len, &mut len) } == 0 {
            return Err(std::io::Error::last_os_error());
        }
        // SAFETY: TokenUser fills a TOKEN_USER at the start of the buffer.
        let user = unsafe { &*(buffer.as_ptr().cast::<TOKEN_USER>()) };
        sid_string(user.User.Sid)
    }

    fn sid_string(sid: PSID) -> std::io::Result<String> {
        let mut text: *mut u16 = std::ptr::null_mut();
        // SAFETY: `sid` is valid; the returned string is freed below.
        if unsafe { ConvertSidToStringSidW(sid, &mut text) } == 0 {
            return Err(std::io::Error::last_os_error());
        }
        // SAFETY: a NUL-terminated wide string allocated by the system.
        let len = (0..).take_while(|&i| unsafe { *text.add(i) } != 0).count();
        let value = String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(text, len) });
        unsafe { LocalFree(text.cast()) };
        Ok(value)
    }

    fn peer_sid(pid: u32) -> std::io::Result<String> {
        // SAFETY: an ordinary query-only open; closed by `Owned`.
        let process = Owned(unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) });
        if process.0.is_null() {
            return Err(std::io::Error::last_os_error());
        }
        process_sid(process.0)
    }

    /// This process's token user SID.
    pub fn current_user_sid() -> std::io::Result<String> {
        // SAFETY: the pseudo-handle needs no closing.
        process_sid(unsafe { GetCurrentProcess() })
    }

    /// The SID of a service's virtual account (`NT SERVICE\<name>`).
    pub fn service_sid(service: &str) -> std::io::Result<String> {
        let account = wide(&format!("NT SERVICE\\{service}"));
        let mut sid = vec![0u8; 68];
        let mut sid_len = sid.len() as u32;
        let mut domain = vec![0u16; 64];
        let mut domain_len = domain.len() as u32;
        let mut use_kind: SID_NAME_USE = 0;
        // SAFETY: every buffer is valid for the lengths passed.
        let ok = unsafe {
            LookupAccountNameW(std::ptr::null(), account.as_ptr(), sid.as_mut_ptr().cast(), &mut sid_len, domain.as_mut_ptr(), &mut domain_len, &mut use_kind)
        };
        if ok == 0 {
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() == Some(ERROR_INSUFFICIENT_BUFFER as i32) {
                return Err(std::io::Error::other("service SID buffer too small"));
            }
            return Err(error);
        }
        sid_string(sid.as_mut_ptr().cast())
    }

    /// A named pipe that admits only `owner_sid` (plus SYSTEM and the
    /// service itself) and rejects remote clients.
    pub struct NamedPipeListener {
        name: Vec<u16>,
        sddl: Vec<u16>,
        first: std::sync::Mutex<bool>,
    }

    impl NamedPipeListener {
        /// `owner_sid` is the desktop user whose vault this custodian holds.
        pub fn new(name: &str, owner_sid: &str) -> Self {
            // Protected DACL: full control for SYSTEM and the creator (the
            // service), read/write for the owner; no inheritance.
            let sddl = format!("D:P(A;;GA;;;SY)(A;;GA;;;OW)(A;;GRGW;;;{owner_sid})");
            Self {
                name: wide(name),
                sddl: wide(&sddl),
                first: std::sync::Mutex::new(true),
            }
        }

        fn create(&self) -> std::io::Result<File> {
            let mut descriptor: *mut core::ffi::c_void = std::ptr::null_mut();
            // SAFETY: a NUL-terminated SDDL string; the descriptor is freed below.
            if unsafe { ConvertStringSecurityDescriptorToSecurityDescriptorW(self.sddl.as_ptr(), SDDL_REVISION_1, &mut descriptor, std::ptr::null_mut()) } == 0 {
                return Err(std::io::Error::last_os_error());
            }
            let attributes = SECURITY_ATTRIBUTES {
                nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
                lpSecurityDescriptor: descriptor,
                bInheritHandle: 0,
            };
            let mut first = self.first.lock().unwrap_or_else(|e| e.into_inner());
            let open_mode = PIPE_ACCESS_DUPLEX | if *first { FILE_FLAG_FIRST_PIPE_INSTANCE } else { 0 };
            // SAFETY: valid name and attributes for the duration of the call.
            let handle = unsafe {
                CreateNamedPipeW(self.name.as_ptr(), open_mode, PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS, PIPE_UNLIMITED_INSTANCES, 64 * 1024, 64 * 1024, 0, &attributes)
            };
            unsafe { LocalFree(descriptor) };
            if handle == INVALID_HANDLE_VALUE {
                return Err(std::io::Error::last_os_error());
            }
            *first = false;
            // SAFETY: a fresh owned pipe handle.
            Ok(unsafe { File::from_raw_handle(handle as RawHandle) })
        }
    }

    impl Listener for NamedPipeListener {
        fn accept(&self) -> std::io::Result<Connection> {
            let file = self.create()?;
            let handle = file.as_raw_handle() as HANDLE;
            // SAFETY: a synchronous connect on our own pipe instance.
            if unsafe { ConnectNamedPipe(handle, std::ptr::null_mut()) } == 0 {
                let error = std::io::Error::last_os_error();
                if error.raw_os_error() != Some(ERROR_PIPE_CONNECTED as i32) {
                    return Err(error);
                }
            }
            let mut pid = 0u32;
            // SAFETY: valid out-pointer on a connected pipe.
            if unsafe { GetNamedPipeClientProcessId(handle, &mut pid) } == 0 {
                return Err(std::io::Error::last_os_error());
            }
            // The DACL admits only the owner; the custodian's policy still
            // selects a principal from this SID and refuses anything else.
            let sid = peer_sid(pid)?;
            Ok(Connection {
                stream: Box::new(PipeStream { file, server: true }),
                peer: Peer::Windows { sid, pid },
            })
        }
    }

    /// Connect to the custodian's pipe at identification-only impersonation.
    pub struct NamedPipeConnector {
        name: Vec<u16>,
    }

    impl NamedPipeConnector {
        pub fn new(name: &str) -> Self {
            Self { name: wide(name) }
        }
    }

    impl Connector for NamedPipeConnector {
        fn connect(&self) -> std::io::Result<Connection> {
            for _ in 0..50 {
                // SAFETY: valid NUL-terminated name; the handle is owned below.
                let handle = unsafe {
                    CreateFileW(self.name.as_ptr(), GENERIC_READ | GENERIC_WRITE, 0, std::ptr::null(), OPEN_EXISTING, SECURITY_SQOS_PRESENT | SECURITY_IDENTIFICATION, std::ptr::null_mut())
                };
                if handle == INVALID_HANDLE_VALUE {
                    let error = std::io::Error::last_os_error();
                    if error.raw_os_error() == Some(ERROR_PIPE_BUSY as i32) {
                        unsafe { WaitNamedPipeW(self.name.as_ptr(), 100) };
                        continue;
                    }
                    return Err(error);
                }
                // SAFETY: a fresh owned handle.
                let file = unsafe { File::from_raw_handle(handle as RawHandle) };
                let mut pid = 0u32;
                if unsafe { GetNamedPipeServerProcessId(handle, &mut pid) } == 0 {
                    return Err(std::io::Error::last_os_error());
                }
                let sid = peer_sid(pid)?;
                return Ok(Connection {
                    stream: Box::new(PipeStream { file, server: false }),
                    peer: Peer::Windows { sid, pid },
                });
            }
            Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "the custodian pipe stayed busy"))
        }
    }
}
