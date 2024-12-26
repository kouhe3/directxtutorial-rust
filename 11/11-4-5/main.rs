#![allow(non_snake_case)]

use std::{
    env, io::{Read, Result}, iter::once, ptr::null_mut, slice, sync::{Arc, Mutex}
};
use windows::{
    core::{Interface, PCSTR, PCWSTR}, Win32::{
        Foundation::{BOOL, HINSTANCE, HWND, LPARAM, LRESULT, TRUE, WPARAM},
        Graphics::{
            Direct3D::{Fxc::D3DCompileFromFile, ID3DBlob, D3D_DRIVER_TYPE_HARDWARE},
            Direct3D11::{
                D3D11CreateDeviceAndSwapChain, ID3D11Buffer, ID3D11ClassLinkage, ID3D11Device, ID3D11DeviceContext, ID3D11InputLayout, ID3D11PixelShader, ID3D11RenderTargetView, ID3D11Texture2D, ID3D11VertexShader, D3D11_BIND_VERTEX_BUFFER, D3D11_BUFFER_DESC, D3D11_CPU_ACCESS_WRITE, D3D11_CREATE_DEVICE_FLAG, D3D11_INPUT_ELEMENT_DESC, D3D11_INPUT_PER_VERTEX_DATA, D3D11_MAPPED_SUBRESOURCE, D3D11_MAP_WRITE_DISCARD, D3D11_SDK_VERSION, D3D11_USAGE_DYNAMIC, D3D11_VIEWPORT
            },
            Dxgi::{
                Common::{DXGI_FORMAT_R32G32B32A32_FLOAT, DXGI_FORMAT_R32G32B32_FLOAT, DXGI_FORMAT_R8G8B8A8_UNORM}, IDXGISwapChain, DXGI_PRESENT, DXGI_SWAP_CHAIN_DESC, DXGI_USAGE_RENDER_TARGET_OUTPUT
            },
            Gdi::HBRUSH,
        },
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DispatchMessageW, LoadCursorW, PeekMessageW, PostQuitMessage, RegisterClassExW, ShowWindow, TranslateMessage, CS_HREDRAW, CS_VREDRAW, HICON, IDC_ARROW, MSG, PM_REMOVE, SW_SHOWDEFAULT, WINDOW_EX_STYLE, WM_DESTROY, WM_QUIT, WNDCLASSEXW, WS_OVERLAPPEDWINDOW
        },
    }
};

#[derive(Clone, Copy)]
struct VERTEX {
    X: f32,
    Y: f32,
    Z: f32,
    Color: [f32; 4],
}

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
static pVBuffer: Mutex<Option<ID3D11Buffer>> = Mutex::new(None);
static pPS: Mutex<Option<ID3D11PixelShader>> = Mutex::new(None);
static pVS: Mutex<Option<ID3D11VertexShader>> = Mutex::new(None);
static pLayout: Mutex<Option<ID3D11InputLayout>> = Mutex::new(None);
fn InitD3D(hWnd: HWND)->std::io::Result<()> {
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

        //draw angle here
        InitPipeline().expect("init pip err");
    }
    Ok(())
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

fn InitGraphics() {
    
    const OUR_VERTICES: [VERTEX; 3] = [
        VERTEX {
            X: 0.0,
            Y: 0.5,
            Z: 0.0,
            Color: [1.0, 0.0, 0.0, 1.0],
        },
        VERTEX {
            X: 0.45,
            Y: -0.5,
            Z: 0.0,
            Color: [0.0, 1.0, 0.0, 1.0],
        },
        VERTEX {
            X: -0.45,
            Y: -0.5,
            Z: 0.0,
            Color: [0.0, 0.0, 1.0, 1.0],
        },
    ];

    let bd = D3D11_BUFFER_DESC {
        Usage: D3D11_USAGE_DYNAMIC,
        ByteWidth: (std::mem::size_of::<VERTEX>() * 3) as u32,
        BindFlags: D3D11_BIND_VERTEX_BUFFER.0 as u32,
        CPUAccessFlags: D3D11_CPU_ACCESS_WRITE.0 as u32,
        ..Default::default()
    };
    unsafe {
        dev.lock().unwrap().as_ref().unwrap().CreateBuffer(
            &bd,
            None,
            Some(&mut *pVBuffer.lock().unwrap()),
        );
        let mut ms: D3D11_MAPPED_SUBRESOURCE = D3D11_MAPPED_SUBRESOURCE::default();
        let _pVBuffer = &pVBuffer.lock().unwrap().clone().unwrap();
        devcon.lock().unwrap().as_ref().unwrap().Map(
            _pVBuffer,
            0,
            D3D11_MAP_WRITE_DISCARD,
            0,
            Some(&mut ms),
        );
        //memcpy(ms.pData, OurVertices, sizeof(OurVertices)); 
        devcon.lock().unwrap().as_ref().unwrap().Unmap(_pVBuffer, 0);
    }
}



fn InitPipeline()->std::io::Result<()>
{
    unsafe{
        let utf16_vec = "shader.hlsl".encode_utf16().chain(once(0)).collect::<Vec<u16>>();
        let shadername = PCWSTR::from_raw(utf16_vec.as_ptr());

        let mut PS:Option<ID3DBlob> = None;
        let mut VS:Option<ID3DBlob> = None;
        println!("cwd {}",env::current_dir().unwrap().to_str().unwrap());
        D3DCompileFromFile(shadername, None, None, PCSTR::from_raw("PShader\0".as_ptr()), PCSTR::from_raw("ps_5_0\0".as_ptr()), 0, 0, &mut PS, None).expect("compile failed");
        D3DCompileFromFile(shadername, None, None, PCSTR::from_raw("VShader\0".as_ptr()), PCSTR::from_raw("vs_5_0\0".as_ptr()), 0, 0, &mut VS, None).expect("compile failed");
        let _dev = dev.lock().unwrap();
        let _devcon = devcon.lock().unwrap();
        
        let VS_slice = slice::from_raw_parts(VS.clone().unwrap().GetBufferPointer() as *const u8, VS.unwrap().GetBufferSize());
        let PS_slice = slice::from_raw_parts(PS.clone().unwrap().GetBufferPointer() as *const u8, PS.unwrap().GetBufferSize());
        _dev.as_ref().unwrap().CreateVertexShader(VS_slice, None, Some(&mut *pVS.lock().unwrap())).expect("CreateVertexShader err");
        _dev.as_ref().unwrap().CreatePixelShader(PS_slice, None, Some(&mut *pPS.lock().unwrap())).expect("CreatePixelShader err");
        
        _devcon.as_ref().unwrap().VSSetShader(&pVS.lock().unwrap().clone().unwrap(), None);
        _devcon.as_ref().unwrap().PSSetShader(&pPS.lock().unwrap().clone().unwrap(), None);
        let ied:[D3D11_INPUT_ELEMENT_DESC;2] = [
            D3D11_INPUT_ELEMENT_DESC{
                SemanticName: PCSTR::from_raw("POSITION\0".as_ptr()),
                SemanticIndex: 0,
                Format: DXGI_FORMAT_R32G32B32_FLOAT,
                InputSlot: 0,
                AlignedByteOffset: 0,
                InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                InstanceDataStepRate: 0
            },
            D3D11_INPUT_ELEMENT_DESC{
                SemanticName: PCSTR::from_raw("COLOR\0".as_ptr()),
                SemanticIndex: 0,
                Format: DXGI_FORMAT_R32G32B32A32_FLOAT,
                InputSlot: 0,
                AlignedByteOffset: 0,
                InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                InstanceDataStepRate: 0
            }
        ];
        _dev.as_ref().unwrap().CreateInputLayout(&ied, VS_slice, Some(&mut *pLayout.lock().unwrap()))?;
        _devcon.as_ref().unwrap().IASetInputLayout(pLayout.lock().unwrap().as_ref().unwrap());
    
        }
        Ok(())
}


