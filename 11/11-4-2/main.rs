#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
use std::{
    ffi::c_void,
    iter::once,
    ptr::{self, null_mut},
    sync::Mutex,
};
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{BOOL, FALSE, HINSTANCE, HWND, LPARAM, LRESULT, RECT, TRUE, WPARAM},
        Graphics::{
            Direct3D::D3D_DRIVER_TYPE_HARDWARE,
            Direct3D11::{
                D3D11CreateDeviceAndSwapChain, ID3D11Device, ID3D11DeviceContext,
                ID3D11RenderTargetView, ID3D11Texture2D, D3D11_CREATE_DEVICE_FLAG,
                D3D11_SDK_VERSION, D3D11_VIEWPORT,
            },
            Dxgi::{
                Common::{DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_MODE_DESC, DXGI_SAMPLE_DESC},
                IDXGISwapChain, DXGI_SWAP_CHAIN_DESC, DXGI_USAGE_RENDER_TARGET_OUTPUT,
            },
            Gdi::{COLOR_WINDOW, HBRUSH},
        },
        UI::WindowsAndMessaging::{
            AdjustWindowRect, CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW,
            LoadCursorW, MessageBoxW, PeekMessageW, PostQuitMessage, RegisterClassExW, ShowWindow,
            TranslateMessage, CS_HREDRAW, CS_VREDRAW, HMENU, IDC_ARROW, MB_OK, MSG, PM_REMOVE,
            SHOW_WINDOW_CMD, WINDOW_EX_STYLE, WM_DESTROY, WM_QUIT, WNDCLASSEXW,
            WS_OVERLAPPEDWINDOW,
        },
    },
};

static swapchain: Mutex<Option<IDXGISwapChain>> = Mutex::new(None);
static dev: Mutex<Option<ID3D11Device>> = Mutex::new(None);
static devcon: Mutex<Option<ID3D11DeviceContext>> = Mutex::new(None);

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
        let mut wr = RECT {
            left: 0,
            top: 0,
            right: 500,
            bottom: 400,
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
            wr.bottom - wr.top,
            HWND(null_mut()), //why we can't just type NULL
            HMENU(null_mut()),
            hinstance,
            None,
        )?;
        let _ = ShowWindow(hwnd, ncmdshow);
        InitD3D(hwnd)?;
        let mut msg = MSG::default();
        loop {
            if PeekMessageW(&mut msg, HWND(null_mut()), 0, 0, PM_REMOVE) == TRUE {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
                if msg.message == WM_QUIT {
                    break;
                }
            }
        }
        CleanD3D();
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
        if msg == WM_DESTROY {
            PostQuitMessage(0);
        }
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}


fn CleanD3D() {
    println!("well,we use Rust!");
}

fn InitD3D(hwnd: HWND) -> std::io::Result<()> {
    unsafe {
        let scd = DXGI_SWAP_CHAIN_DESC {
            BufferCount: 1,
            BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
            OutputWindow: hwnd,
            Windowed: TRUE,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 4,
                ..Default::default()
            },
            BufferDesc: DXGI_MODE_DESC {
                Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                ..Default::default()
            },
            ..Default::default()
        };

        D3D11CreateDeviceAndSwapChain(
            None,
            D3D_DRIVER_TYPE_HARDWARE,
            None,
            D3D11_CREATE_DEVICE_FLAG(0),
            None,
            D3D11_SDK_VERSION,
            Some(&scd),
            Some(&mut *swapchain.lock().unwrap()),
            Some(&mut *dev.lock().unwrap()), //wait wtf is this
            None,
            Some(&mut *devcon.lock().unwrap()),
        )?;
    }
    Ok(())
}
