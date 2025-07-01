use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::{
            Direct3D, Direct3D12,
            Dxgi::{self, Common},
        },
        System::Threading,
        UI::WindowsAndMessaging::*,
    },
};

use std::{mem::ManuallyDrop, ptr};

const TITLE: PCWSTR = windows::core::w!("Minimal D3D12 by Secticide");
const PASSTHROUGH: &'static std::ffi::CStr = c"
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
}";

#[allow(dead_code)]
struct Float3 {
    x: f32,
    y: f32,
    z: f32,
}

extern "system" fn window_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

// ----------------------------------------------------------------------------------------------------

fn main() -> Result<()> {
    unsafe {
        let wnd_class = WNDCLASSW {
            lpfnWndProc: Some(window_proc),
            lpszClassName: TITLE,
            ..Default::default()
        };

        RegisterClassW(&wnd_class);

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            TITLE,
            TITLE,
            WS_POPUP | WS_MAXIMIZE | WS_VISIBLE,
            0,
            0,
            0,
            0,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        // ----------------------------------------------------------------------------------------------------

        let debug_controller: Direct3D12::ID3D12Debug = {
            let mut result = None;
            Direct3D12::D3D12GetDebugInterface(&mut result)?;
            result.unwrap()
        };

        debug_controller.EnableDebugLayer();

        // ----------------------------------------------------------------------------------------------------

        let device: Direct3D12::ID3D12Device = {
            let mut result = None;
            Direct3D12::D3D12CreateDevice(None, Direct3D::D3D_FEATURE_LEVEL_12_0, &mut result)?;
            result.unwrap_unchecked()
        };

        // ----------------------------------------------------------------------------------------------------

        let queue_desc = Direct3D12::D3D12_COMMAND_QUEUE_DESC {
            Type: Direct3D12::D3D12_COMMAND_LIST_TYPE_DIRECT,
            Flags: Direct3D12::D3D12_COMMAND_QUEUE_FLAG_NONE,
            ..Default::default()
        };

        let cmd_queue: Direct3D12::ID3D12CommandQueue = device.CreateCommandQueue(&queue_desc)?;

        // ----------------------------------------------------------------------------------------------------

        let factory: Dxgi::IDXGIFactory4 = Dxgi::CreateDXGIFactory2(Dxgi::DXGI_CREATE_FACTORY_DEBUG)?;

        // ----------------------------------------------------------------------------------------------------

        let swapchain_desc = Dxgi::DXGI_SWAP_CHAIN_DESC1 {
            Format: Common::DXGI_FORMAT_R8G8B8A8_UNORM,
            SampleDesc: Common::DXGI_SAMPLE_DESC { Count: 1, Quality: 0, },
            BufferUsage: Dxgi::DXGI_USAGE_RENDER_TARGET_OUTPUT,
            BufferCount: 2,
            SwapEffect: Dxgi::DXGI_SWAP_EFFECT_FLIP_DISCARD,
            ..Default::default()
        };

        let swapchain: Dxgi::IDXGISwapChain1 =
            factory.CreateSwapChainForHwnd(&cmd_queue, hwnd, &swapchain_desc, None, None)?;
        factory.MakeWindowAssociation(hwnd, Dxgi::DXGI_MWA_NO_ALT_ENTER)?;

        // `IDXGISwapChain3` needed for the `GetCurrentBackBufferIndex` method
        let swapchain: Dxgi::IDXGISwapChain3 = swapchain.cast()?;
        let swapchain_desc = swapchain.GetDesc1()?;

        // ----------------------------------------------------------------------------------------------------

        let rtv_descriptor_heap = Direct3D12::D3D12_DESCRIPTOR_HEAP_DESC {
            Type: Direct3D12::D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
            NumDescriptors: 2,
            ..Default::default()
        };

        let rtv_heap: Direct3D12::ID3D12DescriptorHeap =
            device.CreateDescriptorHeap(&rtv_descriptor_heap)?;

        // ----------------------------------------------------------------------------------------------------

        let rtv_descriptor_size =
            device.GetDescriptorHandleIncrementSize(Direct3D12::D3D12_DESCRIPTOR_HEAP_TYPE_RTV);
        let mut rtv_handle = rtv_heap.GetCPUDescriptorHandleForHeapStart();

        let render_targets: [Direct3D12::ID3D12Resource; 2] =
            [swapchain.GetBuffer(0)?, swapchain.GetBuffer(1)?];

        for target in render_targets.iter() {
            device.CreateRenderTargetView(target, None, rtv_handle);
            rtv_handle.ptr += rtv_descriptor_size as usize;
        }

        // ----------------------------------------------------------------------------------------------------

        let cmd_allocator: Direct3D12::ID3D12CommandAllocator =
            device.CreateCommandAllocator(Direct3D12::D3D12_COMMAND_LIST_TYPE_DIRECT)?;

        // ----------------------------------------------------------------------------------------------------

        let root_signature_desc = Direct3D12::D3D12_ROOT_SIGNATURE_DESC {
            Flags: Direct3D12::D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT,
            ..Default::default()
        };

        let signature = {
            let mut result = None;
            Direct3D12::D3D12SerializeRootSignature(
                &root_signature_desc,
                Direct3D12::D3D_ROOT_SIGNATURE_VERSION_1,
                &mut result,
                None,
            )?;
            result.unwrap_unchecked()
        };

        let signature_slice = std::slice::from_raw_parts(
            signature.GetBufferPointer() as *const _,
            signature.GetBufferSize(),
        );
        let root_signature: Direct3D12::ID3D12RootSignature =
            device.CreateRootSignature(0, signature_slice)?;

        // ----------------------------------------------------------------------------------------------------

        let vertex_shader = {
            let mut result = None;
            Direct3D::Fxc::D3DCompile(
                PASSTHROUGH.as_ptr() as _,
                PASSTHROUGH.count_bytes(),
                PSTR(ptr::null_mut()),
                None,
                None,
                s!("vert_main"),
                s!("vs_5_0"),
                0,
                0,
                &mut result,
                None,
            )?;
            result.unwrap_unchecked()
        };

        let fragment_shader = {
            let mut result = None;
            Direct3D::Fxc::D3DCompile(
                PASSTHROUGH.as_ptr() as _,
                PASSTHROUGH.count_bytes(),
                PSTR(ptr::null_mut()),
                None,
                None,
                s!("frag_main"),
                s!("ps_5_0"),
                0,
                0,
                &mut result,
                None,
            )?;
            result.unwrap_unchecked()
        };

        // ----------------------------------------------------------------------------------------------------

        let input_element_descs = [Direct3D12::D3D12_INPUT_ELEMENT_DESC {
            SemanticName: s!("POSITION"),
            SemanticIndex: 0,
            Format: Common::DXGI_FORMAT_R32G32B32_FLOAT,
            InputSlot: 0,
            AlignedByteOffset: 0,
            InputSlotClass: Direct3D12::D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
            InstanceDataStepRate: 0,
        }];

        let rasterizer_desc = Direct3D12::D3D12_RASTERIZER_DESC {
            FillMode: Direct3D12::D3D12_FILL_MODE_SOLID,
            CullMode: Direct3D12::D3D12_CULL_MODE_BACK,
            FrontCounterClockwise: false.into(),
            DepthBias: Direct3D12::D3D12_DEFAULT_DEPTH_BIAS,
            DepthBiasClamp: Direct3D12::D3D12_DEFAULT_DEPTH_BIAS_CLAMP,
            SlopeScaledDepthBias: Direct3D12::D3D12_DEFAULT_SLOPE_SCALED_DEPTH_BIAS,
            DepthClipEnable: true.into(),
            MultisampleEnable: false.into(),
            AntialiasedLineEnable: false.into(),
            ForcedSampleCount: 0,
            ConservativeRaster: Direct3D12::D3D12_CONSERVATIVE_RASTERIZATION_MODE_OFF,
        };

        let blend_desc = {
            let mut result = Direct3D12::D3D12_BLEND_DESC::default();
            for rt in result.RenderTarget.iter_mut() {
                rt.SrcBlend = Direct3D12::D3D12_BLEND_ONE;
                rt.DestBlend = Direct3D12::D3D12_BLEND_ZERO;
                rt.BlendOp = Direct3D12::D3D12_BLEND_OP_ADD;
                rt.SrcBlendAlpha = Direct3D12::D3D12_BLEND_ONE;
                rt.DestBlendAlpha = Direct3D12::D3D12_BLEND_ZERO;
                rt.BlendOpAlpha = Direct3D12::D3D12_BLEND_OP_ADD;
                rt.LogicOp = Direct3D12::D3D12_LOGIC_OP_NOOP;
                rt.RenderTargetWriteMask = Direct3D12::D3D12_COLOR_WRITE_ENABLE_ALL.0 as u8;
            }
            result
        };

        let mut pipeline_state_desc = Direct3D12::D3D12_GRAPHICS_PIPELINE_STATE_DESC {
            pRootSignature: ManuallyDrop::new(Some(std::mem::transmute_copy(&root_signature))),
            VS: Direct3D12::D3D12_SHADER_BYTECODE {
                pShaderBytecode: vertex_shader.GetBufferPointer(),
                BytecodeLength: vertex_shader.GetBufferSize(),
            },
            PS: Direct3D12::D3D12_SHADER_BYTECODE {
                pShaderBytecode: fragment_shader.GetBufferPointer(),
                BytecodeLength: fragment_shader.GetBufferSize(),
            },
            BlendState: blend_desc,
            SampleMask: std::u32::MAX,
            RasterizerState: rasterizer_desc,
            DepthStencilState: Direct3D12::D3D12_DEPTH_STENCIL_DESC {
                DepthEnable: false.into(),
                StencilEnable: false.into(),
                ..Default::default()
            },
            InputLayout: Direct3D12::D3D12_INPUT_LAYOUT_DESC {
                pInputElementDescs: input_element_descs.as_ptr(),
                NumElements: input_element_descs.len() as u32,
            },
            PrimitiveTopologyType: Direct3D12::D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
            NumRenderTargets: 1,
            SampleDesc: Common::DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            ..Default::default()
        };

        // Only set the first format
        pipeline_state_desc.RTVFormats[0] = Common::DXGI_FORMAT_R8G8B8A8_UNORM;

        let pipeline_state: Direct3D12::ID3D12PipelineState =
            device.CreateGraphicsPipelineState(&pipeline_state_desc)?;

        // ----------------------------------------------------------------------------------------------------

        let cmd_list: Direct3D12::ID3D12GraphicsCommandList = device.CreateCommandList(
            0,
            Direct3D12::D3D12_COMMAND_LIST_TYPE_DIRECT,
            &cmd_allocator,
            &pipeline_state,
        )?;

        let _ = cmd_list.Close();

        // ----------------------------------------------------------------------------------------------------

        let fence: Direct3D12::ID3D12Fence =
            device.CreateFence(0, Direct3D12::D3D12_FENCE_FLAG_NONE)?;

        let mut fence_value = 1;
        let fence_event = Threading::CreateEventW(None, false, false, None)?;

        // ----------------------------------------------------------------------------------------------------

        let vertices = [
            Float3 { x: 0.0, y: 0.5, z: 0.0, },
            Float3 { x: 0.5, y: -0.5, z: 0.0, },
            Float3 { x: -0.5, y: -0.5, z: 0.0, },
        ];

        let heap_props = Direct3D12::D3D12_HEAP_PROPERTIES {
            Type: Direct3D12::D3D12_HEAP_TYPE_UPLOAD,
            ..Default::default()
        };

        let buffer_desc = Direct3D12::D3D12_RESOURCE_DESC {
            Dimension: Direct3D12::D3D12_RESOURCE_DIMENSION_BUFFER,
            Width: std::mem::size_of_val(&vertices) as u64,
            Height: 1,
            DepthOrArraySize: 1,
            MipLevels: 1,
            SampleDesc: Common::DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Layout: Direct3D12::D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
            ..Default::default()
        };

        let vertex_buffer: Direct3D12::ID3D12Resource = {
            let mut result = None;
            device.CreateCommittedResource(
                &heap_props,
                Direct3D12::D3D12_HEAP_FLAG_NONE,
                &buffer_desc,
                Direct3D12::D3D12_RESOURCE_STATE_GENERIC_READ,
                None,
                &mut result,
            )?;
            result.unwrap_unchecked()
        };

        // ----------------------------------------------------------------------------------------------------

        let mut vertex_data_begin = ptr::null_mut();
        let read_range = Direct3D12::D3D12_RANGE { Begin: 0, End: 0 };

        vertex_buffer.Map(0, Some(&read_range), Some(&mut vertex_data_begin))?;

        ptr::copy_nonoverlapping::<Float3>(vertices.as_ptr(), vertex_data_begin as _, vertices.len());

        vertex_buffer.Unmap(0, None);

        // ----------------------------------------------------------------------------------------------------

        let vertex_buffer_view = Direct3D12::D3D12_VERTEX_BUFFER_VIEW {
            BufferLocation: vertex_buffer.GetGPUVirtualAddress(),
            SizeInBytes: std::mem::size_of_val(&vertices) as u32,
            StrideInBytes: std::mem::size_of::<Float3>() as u32,
        };

        let viewport = Direct3D12::D3D12_VIEWPORT {
            Width: swapchain_desc.Width as f32,
            Height: swapchain_desc.Height as f32,
            MinDepth: Direct3D12::D3D12_MIN_DEPTH,
            MaxDepth: Direct3D12::D3D12_MAX_DEPTH,
            ..Default::default()
        };

        let scissor = RECT {
            right: swapchain_desc.Width as i32,
            bottom: swapchain_desc.Height as i32,
            ..Default::default()
        };

        // ----------------------------------------------------------------------------------------------------

        let mut frame_index = swapchain.GetCurrentBackBufferIndex();
        let mut is_running = true;

        while is_running {
            let mut msg = MSG::default();
            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).into() {
                if msg.message == WM_KEYDOWN {
                    is_running = false;
                }

                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            cmd_allocator.Reset()?;

            cmd_list.Reset(&cmd_allocator, &pipeline_state)?;
            cmd_list.SetGraphicsRootSignature(&root_signature);
            cmd_list.RSSetViewports(&[viewport]);
            cmd_list.RSSetScissorRects(&[scissor]);

            let barrier = Direct3D12::D3D12_RESOURCE_BARRIER {
                Type: Direct3D12::D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
                Flags: Direct3D12::D3D12_RESOURCE_BARRIER_FLAG_NONE,
                Anonymous: Direct3D12::D3D12_RESOURCE_BARRIER_0 {
                    Transition: ManuallyDrop::new(Direct3D12::D3D12_RESOURCE_TRANSITION_BARRIER {
                        pResource: ManuallyDrop::new(Some(std::mem::transmute_copy(
                            &render_targets[frame_index as usize],
                        ))),
                        Subresource: Direct3D12::D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                        StateBefore: Direct3D12::D3D12_RESOURCE_STATE_PRESENT,
                        StateAfter: Direct3D12::D3D12_RESOURCE_STATE_RENDER_TARGET,
                    }),
                },
            };
            cmd_list.ResourceBarrier(&[barrier]);

            rtv_handle = rtv_heap.GetCPUDescriptorHandleForHeapStart();
            rtv_handle.ptr += frame_index as usize * rtv_descriptor_size as usize;
            cmd_list.OMSetRenderTargets(1, Some(&rtv_handle), false, None);

            const CLEAR_COLOUR: [f32; 4] = [0.0, 0.2, 0.4, 1.0];
            cmd_list.ClearRenderTargetView(rtv_handle, &CLEAR_COLOUR, None);
            cmd_list.IASetPrimitiveTopology(Direct3D::D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
            cmd_list.IASetVertexBuffers(0, Some(&[vertex_buffer_view]));
            cmd_list.DrawInstanced(3, 1, 0, 0);

            let barrier = Direct3D12::D3D12_RESOURCE_BARRIER {
                Type: Direct3D12::D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
                Flags: Direct3D12::D3D12_RESOURCE_BARRIER_FLAG_NONE,
                Anonymous: Direct3D12::D3D12_RESOURCE_BARRIER_0 {
                    Transition: ManuallyDrop::new(Direct3D12::D3D12_RESOURCE_TRANSITION_BARRIER {
                        pResource: ManuallyDrop::new(Some(std::mem::transmute_copy(
                            &render_targets[frame_index as usize],
                        ))),
                        Subresource: Direct3D12::D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                        StateBefore: Direct3D12::D3D12_RESOURCE_STATE_RENDER_TARGET,
                        StateAfter: Direct3D12::D3D12_RESOURCE_STATE_PRESENT,
                    }),
                },
            };
            cmd_list.ResourceBarrier(&[barrier]);
            cmd_list.Close()?;

            cmd_queue.ExecuteCommandLists(&[Some(cmd_list.cast()?)]);

            swapchain.Present(1, Dxgi::DXGI_PRESENT(0)).ok()?;

            let current_fence_value = fence_value;
            cmd_queue.Signal(&fence, current_fence_value)?;
            fence_value += 1;

            if fence.GetCompletedValue() < current_fence_value {
                fence.SetEventOnCompletion(current_fence_value, fence_event)?;
                Threading::WaitForSingleObject(fence_event, Threading::INFINITE);
            }

            frame_index = swapchain.GetCurrentBackBufferIndex();
        }

        let _ = ShowWindow(hwnd, SW_HIDE);
        let _ = CloseHandle(fence_event);

        Ok(())
    }
}
