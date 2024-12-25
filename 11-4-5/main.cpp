#include <Windows.h>
#include <dxgi.h>
#include <d3d11.h>
#include <d3dcompiler.h>
#pragma comment(lib, "d3d11.lib")
#pragma comment(lib, "d3dcompiler.lib")

#define SCREEN_WIDTH 800
#define SCREEN_HEIGHT 600


IDXGISwapChain* swapchain;
ID3D11Device* dev;
ID3D11DeviceContext* devcon; 
ID3D11RenderTargetView* backbuffer;
ID3D11VertexShader *pVS;
ID3D11PixelShader* pPS;
ID3D11InputLayout* pLayout;
ID3D11Buffer* pVBuffer;
void InitD3D(HWND hWnd); 
void RenderFrame(void); 
void CleanD3D(void); 
void InitGraphics(void); 
void InitPipeline(void); 

LRESULT CALLBACK WindowProc(HWND hWnd, UINT message, WPARAM wParam, LPARAM lParam);

struct VERTEX {
    FLOAT X, Y, Z; // position
    FLOAT Color[4]; // color
};


int WINAPI WinMain(HINSTANCE hInstance,
    HINSTANCE hPrevInstance,
    LPSTR lpCmdLine,
    int nShowCmd)
{
    HWND hWnd;
    WNDCLASSEXW wc;
    ZeroMemory(&wc, sizeof(WNDCLASSEXW));
    wc.cbSize = sizeof(WNDCLASSEXW);
    wc.style = CS_HREDRAW | CS_VREDRAW;
    wc.lpfnWndProc = WindowProc;
    wc.hCursor = LoadCursorW(NULL, IDC_ARROW);
    wc.lpszClassName = L"WinClass";
    RegisterClassExW(&wc);
    RECT wr = { 0, 0, SCREEN_WIDTH, SCREEN_HEIGHT };
    AdjustWindowRect(&wr, WS_OVERLAPPEDWINDOW, FALSE);
    hWnd = CreateWindowExW(
        NULL,
        L"WinClass",
        L"Window Name",
        WS_OVERLAPPEDWINDOW,
        300, 300,
        wr.right - wr.left,
        wr.bottom - wr.top,
        NULL,
        NULL,
        hInstance,
        NULL);
    ShowWindow(hWnd, nShowCmd);
    InitD3D(hWnd);

    MSG msg;

    while (TRUE) {
        if (PeekMessage(&msg, NULL, 0, 0, PM_REMOVE)) {
            TranslateMessage(&msg);
            DispatchMessage(&msg);

            if (msg.message == WM_QUIT)
                break;
        }

        RenderFrame();
    }

    // clean up DirectX and COM
    CleanD3D();

    return msg.wParam;
}

LRESULT CALLBACK WindowProc(HWND hWnd, UINT message, WPARAM wParam, LPARAM lParam)
{
    switch (message) {
    case WM_DESTROY: {
        PostQuitMessage(0);
        return 0;
    } break;
    }

    return DefWindowProc(hWnd, message, wParam, lParam);
}

void InitD3D(HWND hWnd) {
    DXGI_SWAP_CHAIN_DESC scd;
    ZeroMemory(&scd, sizeof(DXGI_SWAP_CHAIN_DESC));
    scd.BufferCount = 1; // one back buffer
    scd.BufferDesc.Format = DXGI_FORMAT_R8G8B8A8_UNORM; // use 32-bit color
    scd.BufferDesc.Width = SCREEN_WIDTH; // set the back buffer width
    scd.BufferDesc.Height = SCREEN_HEIGHT; // set the back buffer height
    scd.BufferUsage = DXGI_USAGE_RENDER_TARGET_OUTPUT; // how swap chain is to be used
    scd.OutputWindow = hWnd; // the window to be used
    scd.SampleDesc.Count = 4; // how many multisamples
    scd.Windowed = TRUE; // windowed/full-screen mode
    scd.Flags = DXGI_SWAP_CHAIN_FLAG_ALLOW_MODE_SWITCH; // allow full-screen switching
    D3D11CreateDeviceAndSwapChain(NULL,
        D3D_DRIVER_TYPE_HARDWARE,
        NULL,
        NULL,
        NULL,
        NULL,
        D3D11_SDK_VERSION,
        &scd,
        &swapchain,
        &dev,
        NULL,
        &devcon);
    ID3D11Texture2D* pBackBuffer;
    swapchain->GetBuffer(0, __uuidof(ID3D11Texture2D), (LPVOID*)&pBackBuffer);

    // use the back buffer address to create the render target
    dev->CreateRenderTargetView(pBackBuffer, NULL, &backbuffer);
    pBackBuffer->Release();

    // set the render target as the back buffer
    devcon->OMSetRenderTargets(1, &backbuffer, NULL);

    // Set the viewport
    D3D11_VIEWPORT viewport;
    ZeroMemory(&viewport, sizeof(D3D11_VIEWPORT));

    viewport.TopLeftX = 0;
    viewport.TopLeftY = 0;
    viewport.Width = SCREEN_WIDTH;
    viewport.Height = SCREEN_HEIGHT;

    devcon->RSSetViewports(1, &viewport);
    InitPipeline();
    InitGraphics();
}

void RenderFrame(void)
{
    FLOAT color[4] = { 0.0f, 0.2f, 0.4f, 1.0f };
    devcon->ClearRenderTargetView(backbuffer, color);
    UINT stride = sizeof(VERTEX);
    UINT offset = 0;

    devcon->IASetVertexBuffers(0, 1, &pVBuffer, &stride, &offset);
    devcon->IASetPrimitiveTopology(D3D10_PRIMITIVE_TOPOLOGY_TRIANGLELIST);

    devcon->Draw(3, 0);
    swapchain->Present(0, 0);
}

// this is the function that cleans up Direct3D and COM
void CleanD3D(void)
{
    swapchain->SetFullscreenState(FALSE, NULL);
    pLayout->Release();
    pVS->Release();
    pPS->Release();
    pVBuffer->Release();
    swapchain->Release();
    backbuffer->Release();
    dev->Release();
    devcon->Release();
}


void InitPipeline()
{
    ID3DBlob *VS, *PS;
    auto shadername = L"shader.hlsl";
    D3DCompileFromFile(shadername, NULL, NULL, "PShader", "ps_5_0", NULL, NULL, &PS, NULL);
    D3DCompileFromFile(shadername, NULL, NULL, "VShader", "vs_5_0", NULL, NULL, &VS, NULL);
    dev->CreateVertexShader(VS->GetBufferPointer(), VS->GetBufferSize(), NULL, &pVS);
    dev->CreatePixelShader(PS->GetBufferPointer(), PS->GetBufferSize(), NULL, &pPS);
    devcon->VSSetShader(pVS, NULL, NULL);
    devcon->PSSetShader(pPS, NULL, NULL);
    D3D11_INPUT_ELEMENT_DESC ied[] = 
    {
        { "POSITION", 0, DXGI_FORMAT_R32G32B32_FLOAT, 0, 0, D3D11_INPUT_PER_VERTEX_DATA, 0 },
        { "COLOR", 0, DXGI_FORMAT_R32G32B32A32_FLOAT, 0, 12, D3D11_INPUT_PER_VERTEX_DATA, 0 },
    };
    dev->CreateInputLayout(ied, 2, VS->GetBufferPointer(), VS->GetBufferSize(), &pLayout);
    devcon->IASetInputLayout(pLayout);
}

void InitGraphics()
{
    VERTEX OurVertices[] = {
        { 0.0f, 0.5f, 0.0f , { 1.0f, 0.0f, 0.0f, 1.0f } },
        { 0.45f, -0.5f, 0.0f , { 0.0f, 1.0f, 0.0f, 1.0f } },
        { -0.45f, -0.5f, 0.0f , { 0.0f, 0.0f, 1.0f, 1.0f } }
    };

    D3D11_BUFFER_DESC bd;
    ZeroMemory(&bd, sizeof(D3D11_BUFFER_DESC));
    bd.Usage = D3D11_USAGE_DYNAMIC;
    bd.ByteWidth = sizeof(VERTEX)*3;
    bd.BindFlags = D3D11_BIND_VERTEX_BUFFER;
    bd.CPUAccessFlags = D3D11_CPU_ACCESS_WRITE;

    dev->CreateBuffer(&bd, NULL, &pVBuffer);
    D3D11_MAPPED_SUBRESOURCE ms;
    devcon->Map(pVBuffer, NULL, D3D11_MAP_WRITE_DISCARD, NULL, &ms);
    memcpy(ms.pData, OurVertices, sizeof(OurVertices));
    devcon->Unmap(pVBuffer, NULL);
}

