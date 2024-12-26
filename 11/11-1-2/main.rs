use std::iter::once;
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::HWND,
        UI::WindowsAndMessaging::{MessageBoxW, MB_OK},
    },
};

fn main() {
    let hwnd = HWND::default();
    let lptext: Vec<u16> = "lptext".encode_utf16().chain(once(0)).collect::<Vec<u16>>();
    let lpcaption: Vec<u16> = "lpcaption"
        .encode_utf16()
        .chain(once(0))
        .collect::<Vec<u16>>();
    let utype = MB_OK;
    unsafe {
        MessageBoxW(
            hwnd,
            PCWSTR::from_raw(lptext.as_ptr()),
            PCWSTR::from_raw(lpcaption.as_ptr()),
            utype,
        );
    }
}
