#pragma comment(lib, "d3d12")
#pragma comment(lib, "dxgi")
#pragma comment(lib, "d3dcompiler")

// ----------------------------------------------------------------------------------------------------

#include <windows.h>
#include <wrl.h> // ComPtr
#include <d3d12.h>
#include <dxgi1_4.h>
#include <d3dcompiler.h>

#include <memory> // std::memcpy, std::size
#include <cstdlib> // std::exit

// ----------------------------------------------------------------------------------------------------

using Microsoft::WRL::ComPtr;

static const wchar_t* TITLE{ L"Minimal D3D12 by Secticide" };

struct float3 { float x, y, z; };

void check(HRESULT hr) {

    if (FAILED(hr)) {
        std::exit(hr);
    }
}

// ----------------------------------------------------------------------------------------------------

int APIENTRY wWinMain(_In_ HINSTANCE hInstance, _In_opt_ HINSTANCE hPrevInstance, _In_ LPWSTR lpCmdLine, _In_ int nCmdShow) {

    WNDCLASSW wnd_class{ 0, DefWindowProcW, 0, 0, 0, 0, 0, 0, 0, TITLE };

    RegisterClassW(&wnd_class);

    HWND hwnd{ CreateWindowExW(0, TITLE, TITLE, WS_POPUP | WS_MAXIMIZE | WS_VISIBLE, 0, 0, 0, 0, nullptr, nullptr, nullptr, nullptr) };

    // ----------------------------------------------------------------------------------------------------

    ComPtr<ID3D12Debug> debug_controller{};
    check(D3D12GetDebugInterface(IID_PPV_ARGS(&debug_controller)));

    debug_controller->EnableDebugLayer();

    // ----------------------------------------------------------------------------------------------------

    ComPtr<ID3D12Device> device{};
    check(D3D12CreateDevice(nullptr, D3D_FEATURE_LEVEL_12_0, IID_PPV_ARGS(&device)));

    // ----------------------------------------------------------------------------------------------------

    D3D12_COMMAND_QUEUE_DESC queue_desc{};
    queue_desc.Flags = D3D12_COMMAND_QUEUE_FLAG_NONE;
    queue_desc.Type = D3D12_COMMAND_LIST_TYPE_DIRECT;

    ComPtr<ID3D12CommandQueue> cmd_queue{};
    check(device->CreateCommandQueue(&queue_desc, IID_PPV_ARGS(&cmd_queue)));

    // ----------------------------------------------------------------------------------------------------

    ComPtr<IDXGIFactory4> factory{};
    check(CreateDXGIFactory2(DXGI_CREATE_FACTORY_DEBUG, IID_PPV_ARGS(&factory)));

    // ----------------------------------------------------------------------------------------------------

    DXGI_SWAP_CHAIN_DESC1 swapchain_desc{};
    swapchain_desc.BufferCount = 2;
    swapchain_desc.Width = 0; // use window width
    swapchain_desc.Height = 0; // use window height
    swapchain_desc.Format = DXGI_FORMAT_R8G8B8A8_UNORM;
    swapchain_desc.BufferUsage = DXGI_USAGE_RENDER_TARGET_OUTPUT;
    swapchain_desc.SwapEffect = DXGI_SWAP_EFFECT_FLIP_DISCARD;
    swapchain_desc.SampleDesc.Count = 1;

    ComPtr<IDXGISwapChain1> swapchain1{};
    check(factory->CreateSwapChainForHwnd(cmd_queue.Get(), hwnd, &swapchain_desc, nullptr, nullptr, &swapchain1));

    factory->MakeWindowAssociation(hwnd, DXGI_MWA_NO_ALT_ENTER);

    // `IDXGISwapChain3` needed for the `GetCurrentBackBufferIndex` method
    ComPtr<IDXGISwapChain3> swapchain{};
    check(swapchain1->QueryInterface(IID_PPV_ARGS(&swapchain)));

    swapchain->GetDesc1(&swapchain_desc);

    // ----------------------------------------------------------------------------------------------------

    D3D12_DESCRIPTOR_HEAP_DESC rtv_heap_desc{};
    rtv_heap_desc.NumDescriptors = 2;
    rtv_heap_desc.Type = D3D12_DESCRIPTOR_HEAP_TYPE_RTV;

    ComPtr<ID3D12DescriptorHeap> rtv_heap{};
    check(device->CreateDescriptorHeap(&rtv_heap_desc, IID_PPV_ARGS(&rtv_heap)));

    // ----------------------------------------------------------------------------------------------------

    ComPtr<ID3D12Resource> render_targets[2]{};

    UINT rtv_descriptor_size{ device->GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_RTV) };

    D3D12_CPU_DESCRIPTOR_HANDLE rtv_handle{ rtv_heap->GetCPUDescriptorHandleForHeapStart() };

    for (UINT i = 0; i < 2; ++i) {
        check(swapchain->GetBuffer(i, IID_PPV_ARGS(&render_targets[i])));
        device->CreateRenderTargetView(render_targets[i].Get(), nullptr, rtv_handle);
        rtv_handle.ptr = SIZE_T(INT64(rtv_handle.ptr) + INT64(rtv_descriptor_size));
    }

    // ----------------------------------------------------------------------------------------------------

    ComPtr<ID3D12CommandAllocator> cmd_allocator{};
    check(device->CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT, IID_PPV_ARGS(&cmd_allocator)));

    // ----------------------------------------------------------------------------------------------------

    D3D12_ROOT_SIGNATURE_DESC root_signature_desc{};
    root_signature_desc.Flags = D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT;

    ComPtr<ID3DBlob> signature{};
    check(D3D12SerializeRootSignature(&root_signature_desc, D3D_ROOT_SIGNATURE_VERSION_1, &signature, nullptr));

    ComPtr<ID3D12RootSignature> root_signature{};
    check(device->CreateRootSignature(0, signature->GetBufferPointer(), signature->GetBufferSize(), IID_PPV_ARGS(&root_signature)));

    // ----------------------------------------------------------------------------------------------------

    ComPtr<ID3DBlob> vertex_shader{};
    check(D3DCompileFromFile(L"passthrough.hlsl", nullptr, nullptr, "vert_main", "vs_5_0", 0, 0, &vertex_shader, nullptr));

    ComPtr<ID3DBlob> fragment_shader{};
    check(D3DCompileFromFile(L"passthrough.hlsl", nullptr, nullptr, "frag_main", "ps_5_0", 0, 0, &fragment_shader, nullptr));

    // ----------------------------------------------------------------------------------------------------

    D3D12_INPUT_ELEMENT_DESC input_element_descs[]{
        { "POSITION", 0, DXGI_FORMAT_R32G32B32_FLOAT, 0, 0, D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA, 0 },
    };

    D3D12_RASTERIZER_DESC rasterizer_desc{};
    rasterizer_desc.FillMode = D3D12_FILL_MODE_SOLID;
    rasterizer_desc.CullMode = D3D12_CULL_MODE_BACK;
    rasterizer_desc.FrontCounterClockwise = FALSE;
    rasterizer_desc.DepthBias = D3D12_DEFAULT_DEPTH_BIAS;
    rasterizer_desc.DepthBiasClamp = D3D12_DEFAULT_DEPTH_BIAS_CLAMP;
    rasterizer_desc.SlopeScaledDepthBias = D3D12_DEFAULT_SLOPE_SCALED_DEPTH_BIAS;
    rasterizer_desc.DepthClipEnable = TRUE;
    rasterizer_desc.MultisampleEnable = FALSE;
    rasterizer_desc.AntialiasedLineEnable = FALSE;
    rasterizer_desc.ForcedSampleCount = 0;
    rasterizer_desc.ConservativeRaster = D3D12_CONSERVATIVE_RASTERIZATION_MODE_OFF;

    D3D12_BLEND_DESC blend_desc{};
    for (UINT i = 0; i < D3D12_SIMULTANEOUS_RENDER_TARGET_COUNT; ++i) {
        D3D12_RENDER_TARGET_BLEND_DESC& rt_blend_desc = blend_desc.RenderTarget[i];
        rt_blend_desc.SrcBlend = D3D12_BLEND_ONE;
        rt_blend_desc.DestBlend = D3D12_BLEND_ZERO;
        rt_blend_desc.BlendOp = D3D12_BLEND_OP_ADD;
        rt_blend_desc.SrcBlendAlpha = D3D12_BLEND_ONE;
        rt_blend_desc.DestBlendAlpha = D3D12_BLEND_ZERO;
        rt_blend_desc.BlendOpAlpha = D3D12_BLEND_OP_ADD;
        rt_blend_desc.LogicOp = D3D12_LOGIC_OP_NOOP;
        rt_blend_desc.RenderTargetWriteMask = D3D12_COLOR_WRITE_ENABLE_ALL;
    }

    D3D12_GRAPHICS_PIPELINE_STATE_DESC pipeline_state_desc{};
    pipeline_state_desc.InputLayout = { input_element_descs, _countof(input_element_descs) };
    pipeline_state_desc.pRootSignature = root_signature.Get();
    pipeline_state_desc.VS = { vertex_shader->GetBufferPointer(), vertex_shader->GetBufferSize() };
    pipeline_state_desc.PS = { fragment_shader->GetBufferPointer(), fragment_shader ->GetBufferSize() };
    pipeline_state_desc.RasterizerState = rasterizer_desc;
    pipeline_state_desc.BlendState = blend_desc;
    pipeline_state_desc.DepthStencilState.DepthEnable = FALSE;
    pipeline_state_desc.DepthStencilState.StencilEnable = FALSE;
    pipeline_state_desc.SampleMask = UINT_MAX;
    pipeline_state_desc.PrimitiveTopologyType = D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE;
    pipeline_state_desc.NumRenderTargets = 1;
    pipeline_state_desc.RTVFormats[0] = DXGI_FORMAT_R8G8B8A8_UNORM;
    pipeline_state_desc.SampleDesc.Count = 1;

    ComPtr<ID3D12PipelineState> pipeline_state{};
    check(device->CreateGraphicsPipelineState(&pipeline_state_desc, IID_PPV_ARGS(&pipeline_state)));

    // ----------------------------------------------------------------------------------------------------

    ComPtr<ID3D12GraphicsCommandList> cmd_list{};
    check(device->CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, cmd_allocator.Get(), pipeline_state.Get(), IID_PPV_ARGS(&cmd_list)));

    cmd_list->Close();

    // ----------------------------------------------------------------------------------------------------

    ComPtr<ID3D12Fence> fence{};
    check(device->CreateFence(0, D3D12_FENCE_FLAG_NONE, IID_PPV_ARGS(&fence)));

    UINT64 fence_value{ 1 };
    HANDLE fence_event{ CreateEventW(nullptr, FALSE, FALSE, nullptr) };

    if (fence_event == nullptr) {
        std::exit(GetLastError());
    }

    // ----------------------------------------------------------------------------------------------------

    float3 vertices[]{
        {  0.0f,  0.5f, 0.0f },
        {  0.5f, -0.5f, 0.0f },
        { -0.5f, -0.5f, 0.0f }
    };

    D3D12_HEAP_PROPERTIES heap_props{};
    heap_props.Type = D3D12_HEAP_TYPE_UPLOAD;

    D3D12_RESOURCE_DESC buffer_desc{};
    buffer_desc.Dimension = D3D12_RESOURCE_DIMENSION_BUFFER;
    buffer_desc.Width = sizeof(float3) * std::size(vertices);
    buffer_desc.Height = 1;
    buffer_desc.DepthOrArraySize = 1;
    buffer_desc.MipLevels = 1;
    buffer_desc.SampleDesc.Count = 1;
    buffer_desc.Layout = D3D12_TEXTURE_LAYOUT_ROW_MAJOR;

    ComPtr<ID3D12Resource> vertex_buffer{};
    check(device->CreateCommittedResource(&heap_props, D3D12_HEAP_FLAG_NONE, &buffer_desc, D3D12_RESOURCE_STATE_GENERIC_READ, nullptr, IID_PPV_ARGS(&vertex_buffer)));

    // ----------------------------------------------------------------------------------------------------

    void* vertex_data_begin{};
    D3D12_RANGE read_range{ 0, 0 };

    check(vertex_buffer->Map(0, &read_range, &vertex_data_begin));

    std::memcpy(vertex_data_begin, std::data(vertices), sizeof(float3) * std::size(vertices));

    vertex_buffer->Unmap(0, nullptr);

    // ----------------------------------------------------------------------------------------------------

    D3D12_VERTEX_BUFFER_VIEW vertex_buffer_view{};
    vertex_buffer_view.BufferLocation = vertex_buffer->GetGPUVirtualAddress();
    vertex_buffer_view.StrideInBytes = sizeof(float3);
    vertex_buffer_view.SizeInBytes = sizeof(float3) * std::size(vertices);

    D3D12_VIEWPORT viewport{};
    viewport.Width = (float)swapchain_desc.Width;
    viewport.Height = (float)swapchain_desc.Height;
    viewport.MinDepth = D3D12_MIN_DEPTH;
    viewport.MaxDepth = D3D12_MAX_DEPTH;

    D3D12_RECT scissor{};
    scissor.right = swapchain_desc.Width;
    scissor.bottom = swapchain_desc.Height;

    // ----------------------------------------------------------------------------------------------------

    UINT frame_index{ swapchain->GetCurrentBackBufferIndex() };
    bool is_running{ true };

    while (is_running) {

        MSG msg{};
        while (PeekMessageW(&msg, nullptr, 0, 0, PM_REMOVE)) {

            if (msg.message == WM_KEYDOWN) {
                is_running = false;
            }

            TranslateMessage(&msg);
            DispatchMessage(&msg);
        }

        cmd_allocator->Reset();

        cmd_list->Reset(cmd_allocator.Get(), pipeline_state.Get());
        cmd_list->SetGraphicsRootSignature(root_signature.Get());
        cmd_list->RSSetViewports(1, &viewport);
        cmd_list->RSSetScissorRects(1, &scissor);

        D3D12_RESOURCE_BARRIER barrier{};
        barrier.Type = D3D12_RESOURCE_BARRIER_TYPE_TRANSITION;
        barrier.Flags = D3D12_RESOURCE_BARRIER_FLAG_NONE;
        barrier.Transition = {
            render_targets[frame_index].Get(),
            D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
            D3D12_RESOURCE_STATE_PRESENT,
            D3D12_RESOURCE_STATE_RENDER_TARGET
        };
        cmd_list->ResourceBarrier(1, &barrier);

        rtv_handle = rtv_heap->GetCPUDescriptorHandleForHeapStart();
        rtv_handle.ptr = SIZE_T(INT64(rtv_handle.ptr) + INT64(frame_index) * INT64(rtv_descriptor_size));
        cmd_list->OMSetRenderTargets(1, &rtv_handle, FALSE, nullptr);

        const float clear_colour[]{ 0.0f, 0.2f, 0.4f, 1.0f };
        cmd_list->ClearRenderTargetView(rtv_handle, clear_colour, 0, nullptr);
        cmd_list->IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
        cmd_list->IASetVertexBuffers(0, 1, &vertex_buffer_view);
        cmd_list->DrawInstanced(3, 1, 0, 0);

        barrier.Transition = {
            render_targets[frame_index].Get(),
            D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
            D3D12_RESOURCE_STATE_RENDER_TARGET,
            D3D12_RESOURCE_STATE_PRESENT
        };
        cmd_list->ResourceBarrier(1, &barrier);
        cmd_list->Close();

        ID3D12CommandList* cmd_lists[]{ cmd_list.Get() };
        cmd_queue->ExecuteCommandLists(std::size(cmd_lists), cmd_lists);

        swapchain->Present(1, 0);

        const UINT64 current_fence_value{ fence_value };
        cmd_queue->Signal(fence.Get(), current_fence_value);
        fence_value += 1;

        if (fence->GetCompletedValue() < current_fence_value) {
            fence->SetEventOnCompletion(current_fence_value, fence_event);
            WaitForSingleObject(fence_event, INFINITE);
        }

        frame_index = swapchain->GetCurrentBackBufferIndex();
    }

    ShowWindow(hwnd, SW_HIDE);
    CloseHandle(fence_event);

    return 0;
}