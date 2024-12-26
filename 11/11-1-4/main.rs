#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(non_snake_case)]

use std::{
    ffi::c_void,
    iter::once,
    ptr::{self, null_mut},
};
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{BOOL, FALSE, HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Gdi::{COLOR_WINDOW, HBRUSH},
        UI::WindowsAndMessaging::{
            AdjustWindowRect, CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, LoadCursorW, MessageBoxW, PostQuitMessage, RegisterClassExW, ShowWindow, TranslateMessage, CS_HREDRAW, CS_VREDRAW, HMENU, IDC_ARROW, MB_OK, MSG, SHOW_WINDOW_CMD, WINDOW_EX_STYLE, WM_DESTROY, WNDCLASSEXW, WS_OVERLAPPEDWINDOW
        },
    },
};

fn main() -> std::io::Result<()> {
    unsafe {
        let ncmdshow = SHOW_WINDOW_CMD(10); //or orther value https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindow
        let hinstance = HINSTANCE(null_mut());
        let wc = WNDCLASSEXW {
            cbSize: (size_of::<WNDCLASSEXW>()) as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(WindowProc),
            hInstance: hinstance,
            hCursor: LoadCursorW(hinstance, IDC_ARROW)?,
            hbrBackground: HBRUSH(null_mut()), //how to use (HBRUSH)COLOR_WINDOW;
            lpszClassName: PCWSTR::from_raw(
                "WindowClass1"
                    .encode_utf16()
                    .chain(once(0))
                    .collect::<Vec<u16>>()
                    .as_ptr(),
            ),
            ..Default::default()
        };
        RegisterClassExW(&wc);
        let mut wr = RECT{left:0,
            top:0,
            right:500,
            bottom:400,
        };
        AdjustWindowRect(&mut wr, WS_OVERLAPPEDWINDOW, FALSE)?;
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            PCWSTR::from_raw(
                "WindowClass1"
                    .encode_utf16()
                    .chain(once(0))
                    .collect::<Vec<u16>>()
                    .as_ptr(),
            ),
            PCWSTR::from_raw(
                "Window title"
                    .encode_utf16()
                    .chain(once(0))
                    .collect::<Vec<u16>>()
                    .as_ptr(),
            ),
            WS_OVERLAPPEDWINDOW,
            300,
            300,
            wr.right - wr.left,
            wr.bottom-wr.top,
            HWND(null_mut()), //why we can't just type NULL
            HMENU(null_mut()),
            hinstance,
            None,
        )?;
        ShowWindow(hwnd, ncmdshow);
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, HWND(null_mut()), 0, 0) == BOOL(1) {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        Ok(())
    }
}

unsafe extern "system" fn WindowProc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            WM_DESTROY => {
                PostQuitMessage(0);
            }
            _ => {}
        }
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}
