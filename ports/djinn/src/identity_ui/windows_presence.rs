// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Desktop Hello requires an HWND owned by the resident. Keep its parent
//! responsive on a dedicated UI thread while Castellan verifies consent.

use std::sync::{OnceLock, mpsc};
use std::thread::JoinHandle;

use castellan::custody::OsPresence;
use personae::IdentityError;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, MSG,
    PostMessageW, PostQuitMessage, RegisterClassW, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
    SWP_SHOWWINDOW, SetForegroundWindow, SetWindowPos, TranslateMessage, WINDOW_EX_STYLE, WM_CLOSE,
    WM_DESTROY, WNDCLASSW, WS_CAPTION, WS_OVERLAPPED, WS_SYSMENU,
};
use windows::core::w;

struct Parent {
    hwnd: isize,
    thread: Option<JoinHandle<()>>,
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_DESTROY {
        unsafe { PostQuitMessage(0) };
    }
    unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
}

impl Parent {
    fn open() -> Result<Self, IdentityError> {
        let (send, receive) = mpsc::sync_channel(1);
        let thread = std::thread::Builder::new()
            .name("djinn-hello-parent".into())
            .spawn(move || {
                let window = (|| -> windows::core::Result<HWND> {
                    static CLASS: OnceLock<Result<(), String>> = OnceLock::new();
                    let registered = CLASS.get_or_init(|| {
                        let class = WNDCLASSW {
                            lpfnWndProc: Some(window_proc),
                            lpszClassName: w!("DjinnIdentityPresence"),
                            ..Default::default()
                        };
                        if unsafe { RegisterClassW(&class) } == 0 {
                            Err(windows::core::Error::from_thread().to_string())
                        } else {
                            Ok(())
                        }
                    });
                    if let Err(reason) = registered {
                        return Err(windows::core::Error::new(
                            windows::core::HRESULT(0x80004005u32 as i32),
                            reason,
                        ));
                    }
                    unsafe {
                        CreateWindowExW(
                            WINDOW_EX_STYLE::default(),
                            w!("DjinnIdentityPresence"),
                            w!("Djinn: unlock identity with Windows Hello"),
                            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU,
                            CW_USEDEFAULT,
                            CW_USEDEFAULT,
                            460,
                            120,
                            None,
                            None,
                            None,
                            None,
                        )
                    }
                })();
                let hwnd = match window {
                    Ok(hwnd) => hwnd,
                    Err(error) => {
                        let _ = send.send(Err(error.to_string()));
                        return;
                    },
                };
                unsafe {
                    // Native consent must be visible even when the resident
                    // was started with a hidden-window launch state.
                    let _ = SetWindowPos(
                        hwnd,
                        None,
                        0,
                        0,
                        0,
                        0,
                        SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_SHOWWINDOW,
                    );
                    let _ = SetForegroundWindow(hwnd);
                }
                if send.send(Ok(hwnd.0 as isize)).is_err() {
                    unsafe {
                        let _ = windows::Win32::UI::WindowsAndMessaging::DestroyWindow(hwnd);
                    }
                    return;
                }
                let mut message = MSG::default();
                loop {
                    let result = unsafe { GetMessageW(&mut message, None, 0, 0) }.0;
                    if result <= 0 {
                        break;
                    }
                    unsafe {
                        let _ = TranslateMessage(&message);
                        DispatchMessageW(&message);
                    }
                }
            })
            .map_err(|error| IdentityError::Backend(error.to_string()))?;
        match receive.recv() {
            Ok(Ok(hwnd)) => Ok(Self {
                hwnd,
                thread: Some(thread),
            }),
            other => {
                let _ = thread.join();
                Err(IdentityError::Backend(format!(
                    "Hello parent unavailable: {other:?}"
                )))
            },
        }
    }
}

impl Drop for Parent {
    fn drop(&mut self) {
        unsafe {
            let _ = PostMessageW(Some(HWND(self.hwnd as _)), WM_CLOSE, WPARAM(0), LPARAM(0));
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

pub(super) fn verify() -> Result<OsPresence, IdentityError> {
    let parent = Parent::open()?;
    castellan::custody::unlock::verify_presence_for_window(
        parent.hwnd,
        "Unlock your identity vault",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::UI::WindowsAndMessaging::{GetWindowThreadProcessId, IsWindow};

    #[test]
    fn consent_parent_belongs_to_resident_and_closes_after_use() {
        let parent = Parent::open().expect("create resident consent parent");
        let hwnd = HWND(parent.hwnd as _);
        let mut owner = 0;
        unsafe {
            GetWindowThreadProcessId(hwnd, Some(&mut owner));
        }
        assert_eq!(owner, std::process::id());
        assert!(unsafe { IsWindow(Some(hwnd)) }.as_bool());
        drop(parent);
        assert!(!unsafe { IsWindow(Some(hwnd)) }.as_bool());
    }
}
