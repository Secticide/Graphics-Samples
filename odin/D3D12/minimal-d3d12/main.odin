package main

import "core:mem"
import "core:os"

import win "core:sys/windows"

import d3d12 "vendor:directx/d3d12"
import d3dc "vendor:directx/d3d_compiler"
import dxgi "vendor:directx/dxgi"

// ----------------------------------------------------------------------------------------------------

TITLE :: "Minimal D3D12 by Secticide"
PASSTHROUGH: string : `
struct vertex_output
{
    float4 position : SV_POSITION;
};

vertex_output vert_main(float3 position : POSITION)
{
    vertex_output output;
    output.position = float4(position, 1.0f);
    return output;
}

float4 frag_main(vertex_output input) : SV_TARGET
{
    return float4(1.0f, 0.0f, 0.0f, 1.0f);
}`

Float3 :: [3]f32

check :: proc(hr: win.HRESULT) {
	if win.FAILED(hr) {
		os.exit(int(hr))
	}
}

// ----------------------------------------------------------------------------------------------------

main :: proc() {
	wnd_class := win.WNDCLASSW{
		lpfnWndProc = win.DefWindowProcW,
		lpszClassName = win.L(TITLE),
		hCursor = win.LoadCursorA(nil, win.IDC_ARROW),
	}

	win.RegisterClassW(&wnd_class)

	hwnd := win.CreateWindowExW(0, win.L(TITLE), win.L(TITLE), win.WS_POPUP | win.WS_MAXIMIZE | win.WS_VISIBLE, 0, 0, 0, 0, nil, nil, nil, nil)

	// ----------------------------------------------------------------------------------------------------

	debug_controller: ^d3d12.IDebug
	check(d3d12.GetDebugInterface(d3d12.IDebug_UUID, (^rawptr)(&debug_controller)))
	defer debug_controller->Release()

	debug_controller->EnableDebugLayer()

	// ----------------------------------------------------------------------------------------------------

	device: ^d3d12.IDevice
	check(d3d12.CreateDevice(nil, ._12_0, d3d12.IDevice_UUID, (^rawptr)(&device)))
	defer device->Release()

	// ----------------------------------------------------------------------------------------------------

	queue_desc := d3d12.COMMAND_QUEUE_DESC{
		Type = .DIRECT,
	}

	cmd_queue: ^d3d12.ICommandQueue
	check(device->CreateCommandQueue(&queue_desc, d3d12.ICommandQueue_UUID, (^rawptr)(&cmd_queue)))
	defer cmd_queue->Release()

	// ----------------------------------------------------------------------------------------------------

	factory: ^dxgi.IFactory4
	check(dxgi.CreateDXGIFactory2({.DEBUG}, dxgi.IFactory4_UUID, (^rawptr)(&factory)))
	defer factory->Release()

	// ----------------------------------------------------------------------------------------------------

	swapchain_desc := dxgi.SWAP_CHAIN_DESC1{
		Width = 0, // use window width
		Height = 0, // use window height
		Format = .R8G8B8A8_UNORM,
		SampleDesc = {Count = 1},
		BufferUsage = {.RENDER_TARGET_OUTPUT},
		BufferCount = 2,
		SwapEffect = .FLIP_DISCARD,
	}

	swapchain1: ^dxgi.ISwapChain1
	check(factory->CreateSwapChainForHwnd(cmd_queue, hwnd, &swapchain_desc, nil, nil, &swapchain1))
	defer swapchain1->Release()

	factory->MakeWindowAssociation(hwnd, {.NO_ALT_ENTER})

	// `ISwapChain3` needed for the `GetCurrentBackBufferIndex` method
	swapchain: ^dxgi.ISwapChain3
	check(swapchain1->QueryInterface(dxgi.ISwapChain3_UUID, (^rawptr)(&swapchain)))
	defer swapchain->Release()

	swapchain->GetDesc1(&swapchain_desc)

	// ----------------------------------------------------------------------------------------------------

	rtv_heap_desc := d3d12.DESCRIPTOR_HEAP_DESC{
		Type = .RTV,
		NumDescriptors = 2,
	}

	rtv_heap: ^d3d12.IDescriptorHeap
	check(device->CreateDescriptorHeap(&rtv_heap_desc, d3d12.IDescriptorHeap_UUID, (^rawptr)(&rtv_heap)))
	defer rtv_heap->Release()

	// ----------------------------------------------------------------------------------------------------

	render_targets: [2]^d3d12.IResource

	rtv_descriptor_size := device->GetDescriptorHandleIncrementSize(.RTV)

	rtv_handle: d3d12.CPU_DESCRIPTOR_HANDLE
	rtv_heap->GetCPUDescriptorHandleForHeapStart(&rtv_handle)

	for &target, i in render_targets {
		check(swapchain->GetBuffer(u32(i), d3d12.IResource_UUID, (^rawptr)(&target)))
		device->CreateRenderTargetView(target, nil, rtv_handle)
		rtv_handle.ptr += uint(rtv_descriptor_size)
	}
	defer for target in render_targets {
		target->Release()
	}

	// ----------------------------------------------------------------------------------------------------

	cmd_allocator: ^d3d12.ICommandAllocator
	check(device->CreateCommandAllocator(.DIRECT, d3d12.ICommandAllocator_UUID, (^rawptr)(&cmd_allocator)))
	defer cmd_allocator->Release()

	// ----------------------------------------------------------------------------------------------------

	root_signature_desc := d3d12.ROOT_SIGNATURE_DESC{
		Flags = {.ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT},
	}

	signature: ^d3d12.IBlob
	check(d3d12.SerializeRootSignature(&root_signature_desc, ._1, &signature, nil))
	defer signature->Release()

	root_signature: ^d3d12.IRootSignature
	check(device->CreateRootSignature(0, signature->GetBufferPointer(), signature->GetBufferSize(), d3d12.IRootSignature_UUID, (^rawptr)(&root_signature)))
	defer root_signature->Release()

	// ----------------------------------------------------------------------------------------------------

	vertex_shader: ^d3dc.ID3DBlob
	check(d3dc.Compile(raw_data(PASSTHROUGH), len(PASSTHROUGH), nil, nil, nil, "vert_main", "vs_5_0", 0, 0, &vertex_shader, nil))
	defer vertex_shader->Release()

	fragment_shader: ^d3dc.ID3DBlob
	check(d3dc.Compile(raw_data(PASSTHROUGH), len(PASSTHROUGH), nil, nil, nil, "frag_main", "ps_5_0", 0, 0, &fragment_shader, nil))
	defer fragment_shader->Release()

	// ----------------------------------------------------------------------------------------------------

	input_element_descs := [?]d3d12.INPUT_ELEMENT_DESC{
		{"POSITION", 0, .R32G32B32_FLOAT, 0, 0, .PER_VERTEX_DATA, 0},
	}

	rasterizer_desc := d3d12.RASTERIZER_DESC{
		FillMode = .SOLID,
		CullMode = .BACK,
		FrontCounterClockwise = false,
		DepthBias = d3d12.DEFAULT_DEPTH_BIAS,
		DepthBiasClamp = d3d12.DEFAULT_DEPTH_BIAS_CLAMP,
		SlopeScaledDepthBias = d3d12.DEFAULT_SLOPE_SCALED_DEPTH_BIAS,
		DepthClipEnable = true,
		MultisampleEnable = false,
		AntialiasedLineEnable = false,
		ForcedSampleCount = 0,
		ConservativeRaster = .OFF,
	}

	blend_desc: d3d12.BLEND_DESC
	for &rt_blend_desc in blend_desc.RenderTarget {
		rt_blend_desc.SrcBlend = .ONE
		rt_blend_desc.DestBlend = .ZERO
		rt_blend_desc.BlendOp = .ADD
		rt_blend_desc.SrcBlendAlpha = .ONE
		rt_blend_desc.DestBlendAlpha = .ZERO
		rt_blend_desc.BlendOpAlpha = .ADD
		rt_blend_desc.LogicOp = .NOOP
		rt_blend_desc.RenderTargetWriteMask = u8(transmute(u32)d3d12.COLOR_WRITE_ENABLE_ALL)
	}

	pipeline_state_desc := d3d12.GRAPHICS_PIPELINE_STATE_DESC{
		InputLayout = {raw_data(&input_element_descs), len(input_element_descs)},
		pRootSignature = root_signature,
		VS = {vertex_shader->GetBufferPointer(), vertex_shader->GetBufferSize()},
		PS = {fragment_shader->GetBufferPointer(), fragment_shader->GetBufferSize()},
		RasterizerState = rasterizer_desc,
		BlendState = blend_desc,
		DepthStencilState = {DepthEnable = false, StencilEnable = false},
		SampleMask = max(u32),
		PrimitiveTopologyType = .TRIANGLE,
		NumRenderTargets = 1,
		SampleDesc = {Count = 1},
	}

	// Only set the first format
	pipeline_state_desc.RTVFormats[0] = .R8G8B8A8_UNORM

	pipeline_state: ^d3d12.IPipelineState
	check(device->CreateGraphicsPipelineState(&pipeline_state_desc, d3d12.IPipelineState_UUID, (^rawptr)(&pipeline_state)))
	defer pipeline_state->Release()

	// ----------------------------------------------------------------------------------------------------

	cmd_list: ^d3d12.IGraphicsCommandList
	check(device->CreateCommandList(0, .DIRECT, cmd_allocator, pipeline_state, d3d12.IGraphicsCommandList_UUID, (^rawptr)(&cmd_list)))
	defer cmd_list->Release()

	cmd_list->Close()

	// ----------------------------------------------------------------------------------------------------

	fence: ^d3d12.IFence
	check(device->CreateFence(0, {}, d3d12.IFence_UUID, (^rawptr)(&fence)))
	defer fence->Release()

	fence_value: u64 = 1
	fence_event := win.CreateEventW(nil, false, false, nil)

	if fence_event == nil {
		os.exit(int(win.GetLastError()))
	}

	// ----------------------------------------------------------------------------------------------------

	vertices := [?]Float3{
		{ 0.0,  0.5, 0.0},
		{ 0.5, -0.5, 0.0},
		{-0.5, -0.5, 0.0},
	}

	heap_props := d3d12.HEAP_PROPERTIES{
		Type = .UPLOAD,
	}

	buffer_desc := d3d12.RESOURCE_DESC{
		Dimension = .BUFFER,
		Width = size_of(vertices),
		Height = 1,
		DepthOrArraySize = 1,
		MipLevels = 1,
		SampleDesc = {Count = 1},
		Layout = .ROW_MAJOR,
	}

	vertex_buffer: ^d3d12.IResource
	check(device->CreateCommittedResource(&heap_props, {}, &buffer_desc, d3d12.RESOURCE_STATE_GENERIC_READ, nil, d3d12.IResource_UUID, (^rawptr)(&vertex_buffer)))
	defer vertex_buffer->Release()

	// ----------------------------------------------------------------------------------------------------

	vertex_data_begin: rawptr
	read_range := d3d12.RANGE{Begin = 0, End = 0}

	check(vertex_buffer->Map(0, &read_range, &vertex_data_begin))

	mem.copy_non_overlapping(vertex_data_begin, &vertices, size_of(vertices))

	vertex_buffer->Unmap(0, nil)

	// ----------------------------------------------------------------------------------------------------

	vertex_buffer_view := d3d12.VERTEX_BUFFER_VIEW{
		BufferLocation = vertex_buffer->GetGPUVirtualAddress(),
		StrideInBytes = size_of(Float3),
		SizeInBytes = size_of(vertices),
	}

	viewport := d3d12.VIEWPORT{
		Width = f32(swapchain_desc.Width),
		Height = f32(swapchain_desc.Height),
		MinDepth = d3d12.MIN_DEPTH,
		MaxDepth = d3d12.MAX_DEPTH,
	}

	scissor := d3d12.RECT{
		right = i32(swapchain_desc.Width),
		bottom = i32(swapchain_desc.Height),
	}

	// ----------------------------------------------------------------------------------------------------

	frame_index := swapchain->GetCurrentBackBufferIndex()
	is_running := true

	for is_running {
		msg: win.MSG
		for win.PeekMessageW(&msg, nil, 0, 0, win.PM_REMOVE) {
			if msg.message == win.WM_KEYDOWN {
				is_running = false
			}

			win.TranslateMessage(&msg)
			win.DispatchMessageW(&msg)
		}

		check(cmd_allocator->Reset())

		check(cmd_list->Reset(cmd_allocator, pipeline_state))
		cmd_list->SetGraphicsRootSignature(root_signature)
		cmd_list->RSSetViewports(1, &viewport)
		cmd_list->RSSetScissorRects(1, &scissor)

		barrier := d3d12.RESOURCE_BARRIER{
			Type = .TRANSITION,
			Transition = {
				pResource = render_targets[frame_index],
				Subresource = d3d12.RESOURCE_BARRIER_ALL_SUBRESOURCES,
				StateBefore = d3d12.RESOURCE_STATE_PRESENT,
				StateAfter = {.RENDER_TARGET},
			},
		}
		cmd_list->ResourceBarrier(1, &barrier)

		rtv_heap->GetCPUDescriptorHandleForHeapStart(&rtv_handle)
		rtv_handle.ptr += uint(frame_index) * uint(rtv_descriptor_size)
		cmd_list->OMSetRenderTargets(1, &rtv_handle, false, nil)

		clear_colour := [4]f32{0.0, 0.2, 0.4, 1.0}
		cmd_list->ClearRenderTargetView(rtv_handle, &clear_colour, 0, nil)
		cmd_list->IASetPrimitiveTopology(.TRIANGLELIST)
		cmd_list->IASetVertexBuffers(0, 1, &vertex_buffer_view)
		cmd_list->DrawInstanced(3, 1, 0, 0)

		barrier = d3d12.RESOURCE_BARRIER{
			Type = .TRANSITION,
			Transition = {
				pResource = render_targets[frame_index],
				Subresource = d3d12.RESOURCE_BARRIER_ALL_SUBRESOURCES,
				StateBefore = {.RENDER_TARGET},
				StateAfter = d3d12.RESOURCE_STATE_PRESENT,
			},
		}
		cmd_list->ResourceBarrier(1, &barrier)
		check(cmd_list->Close())

		cmd_lists := [?]^d3d12.ICommandList{cmd_list}
		cmd_queue->ExecuteCommandLists(len(cmd_lists), raw_data(&cmd_lists))

		check(swapchain->Present(1, {}))

		current_fence_value := fence_value
		check(cmd_queue->Signal(fence, current_fence_value))
		fence_value += 1

		if fence->GetCompletedValue() < current_fence_value {
			check(fence->SetEventOnCompletion(current_fence_value, fence_event))
			win.WaitForSingleObject(fence_event, win.INFINITE)
		}

		frame_index = swapchain->GetCurrentBackBufferIndex()
	}

	// ----------------------------------------------------------------------------------------------------

	win.ShowWindow(hwnd, win.SW_HIDE)
	win.CloseHandle(fence_event)
}
