#![allow(non_snake_case)]

use std::{
    io::Result,
    ptr::null_mut,
    sync::{Arc, Mutex},
};
use windows::{
    Win32::{
        Foundation::{BOOL, HINSTANCE, HWND, LPARAM, LRESULT, TRUE, WPARAM},
        Graphics::{
            Direct3D::D3D_DRIVER_TYPE_HARDWARE,
            Direct3D11::{
                D3D11_CREATE_DEVICE_FLAG, D3D11_SDK_VERSION, D3D11_VIEWPORT,
                D3D11CreateDeviceAndSwapChain, ID3D11Device, ID3D11DeviceContext,
                ID3D11RenderTargetView, ID3D11Texture2D,
            },
            Dxgi::{
                Common::DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_PRESENT, DXGI_SWAP_CHAIN_DESC,
                DXGI_USAGE_RENDER_TARGET_OUTPUT, IDXGISwapChain,
            },
            Gdi::HBRUSH,
        },
        UI::WindowsAndMessaging::{
            CS_HREDRAW, CS_VREDRAW, CreateWindowExW, DefWindowProcW, DispatchMessageW, HICON,
            IDC_ARROW, LoadCursorW, MSG, PM_REMOVE, PeekMessageW, PostQuitMessage,
            RegisterClassExW, SW_SHOWDEFAULT, ShowWindow, TranslateMessage, WINDOW_EX_STYLE,
            WM_DESTROY, WM_QUIT, WNDCLASSEXW, WS_OVERLAPPEDWINDOW,
        },
    },
    core::{Interface, PCWSTR},
};

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

fn main() -> Result<()> {
    unsafe {
        let h_instance: HINSTANCE = HINSTANCE(null_mut());
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: h_instance,
            hIcon: HICON(null_mut()),
            hCursor: LoadCursorW(h_instance, IDC_ARROW)?,
            hIconSm: HICON(null_mut()),
            hbrBackground: HBRUSH(null_mut()),
            lpszClassName: PCWSTR::from_raw(
                "WindowClass1\0"
                    .encode_utf16()
                    .collect::<Vec<u16>>()
                    .as_ptr(),
            ),
            lpszMenuName: PCWSTR(null_mut()),
        };

        RegisterClassExW(&wc);

        let hWnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR::from_raw(
                "WindowClass1\0"
                    .encode_utf16()
                    .collect::<Vec<u16>>()
                    .as_ptr(),
            ),
            PCWSTR::from_raw(
                "Our First Windowed Program\0"
                    .encode_utf16()
                    .collect::<Vec<u16>>()
                    .as_ptr(),
            ),
            WS_OVERLAPPEDWINDOW,
            100,
            100,
            500,
            500,
            None,
            None,
            h_instance,
            None,
        )?;

        ShowWindow(hWnd, SW_SHOWDEFAULT);
        //UpdateWindow(hWnd);
        InitD3D(hWnd);
        let mut msg = MSG::default();

        loop {
            if PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE) != BOOL(0) {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
                if msg.message == WM_QUIT {
                    break;
                }
            }
            RenderFrame();
        }
        Ok(())
    }
}

static swapchain: Mutex<Option<IDXGISwapChain>> = Mutex::new(None);
static dev: Mutex<Option<ID3D11Device>> = Mutex::new(None);
static devcon: Mutex<Option<ID3D11DeviceContext>> = Mutex::new(None);
static backbuffer: Mutex<Option<ID3D11RenderTargetView>> = Mutex::new(None);
fn InitD3D(hWnd: HWND) {
    unsafe {
        let mut scd = DXGI_SWAP_CHAIN_DESC::default();
        scd.BufferCount = 1;
        scd.BufferDesc.Format = DXGI_FORMAT_R8G8B8A8_UNORM;
        scd.BufferUsage = DXGI_USAGE_RENDER_TARGET_OUTPUT;
        scd.OutputWindow = hWnd;
        scd.SampleDesc.Count = 4;
        scd.Windowed = TRUE;

        let _ = D3D11CreateDeviceAndSwapChain(
            None,
            D3D_DRIVER_TYPE_HARDWARE,
            None,
            D3D11_CREATE_DEVICE_FLAG(0),
            None,
            D3D11_SDK_VERSION,
            Some(&scd),
            Some(&mut *swapchain.lock().unwrap()),
            Some(&mut *dev.lock().unwrap()),
            None,
            Some(&mut *devcon.lock().unwrap()),
        );

        let pBackBuffer = swapchain
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .GetBuffer::<ID3D11Texture2D>(0)
            .unwrap();
        dev.lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .CreateRenderTargetView(&pBackBuffer, None, Some(&mut *backbuffer.lock().unwrap()))
            .unwrap();
        devcon
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .OMSetRenderTargets(Some(&[backbuffer.lock().unwrap().as_ref().cloned()]), None);
        let viewport = D3D11_VIEWPORT {
            TopLeftX: 0.0,
            TopLeftY: 0.0,
            Width: 800.0,
            Height: 600.0,
            MinDepth: 0.0,
            MaxDepth: 1.0,
        };
        devcon
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .RSSetViewports(Some(&[viewport]));
    }
}

fn RenderFrame() {
    unsafe {
        let blue: &[f32; 4] = &[0.0, 0.2, 0.4, 1.0];
        devcon
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .ClearRenderTargetView(backbuffer.lock().unwrap().as_ref().unwrap(), blue);
        let _ = swapchain
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .Present(0, DXGI_PRESENT(0));
    }
}
