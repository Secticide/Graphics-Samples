use windows::{
    Win32::{Foundation::*, UI::WindowsAndMessaging::*},
    core::*,
};

use ash::{prelude::VkResult, *};

const TITLE: PCWSTR = windows::core::w!("Minimal Vulkan by Secticide");

#[allow(dead_code)]
struct Float3 {
    x: f32,
    y: f32,
    z: f32,
}

extern "system" fn window_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

unsafe extern "system" fn debug_callback(
    _severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    _msg_type: vk::DebugUtilsMessageTypeFlagsEXT,
    data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    unsafe {
        eprintln!("[Vulkan] {:?}", std::ffi::CStr::from_ptr((*data).p_message));
    }
    vk::FALSE
}

// Vertex shader GLSL source:
// #version 450
// layout(location = 0) in vec3 in_position;
// void main() { gl_Position = vec4(in_position, 1.0); }
const VERT_SPV: &[u32] = &[
    0x07230203, 0x00010000, 0x00080001, 0x00000017, 0x00000000, 0x00020011, 0x00000001, 0x0003000e,
    0x00000000, 0x00000001, 0x0007000f, 0x00000000, 0x0000000f, 0x6e69616d, 0x00000000, 0x0000000a,
    0x0000000c, 0x00050048, 0x00000008, 0x00000000, 0x0000000b, 0x00000000, 0x00030047, 0x00000008,
    0x00000002, 0x00040047, 0x0000000c, 0x0000001e, 0x00000000, 0x00020013, 0x00000001, 0x00030021,
    0x00000002, 0x00000001, 0x00030016, 0x00000003, 0x00000020, 0x00040017, 0x00000004, 0x00000003,
    0x00000004, 0x00040017, 0x00000005, 0x00000003, 0x00000003, 0x00040015, 0x00000006, 0x00000020,
    0x00000001, 0x0004002b, 0x00000006, 0x00000007, 0x00000000, 0x0003001e, 0x00000008, 0x00000004,
    0x00040020, 0x00000009, 0x00000003, 0x00000008, 0x0004003b, 0x00000009, 0x0000000a, 0x00000003,
    0x00040020, 0x0000000b, 0x00000001, 0x00000005, 0x0004003b, 0x0000000b, 0x0000000c, 0x00000001,
    0x00040020, 0x0000000d, 0x00000003, 0x00000004, 0x0004002b, 0x00000003, 0x0000000e, 0x3f800000,
    0x00050036, 0x00000001, 0x0000000f, 0x00000000, 0x00000002, 0x000200f8, 0x00000010, 0x0004003d,
    0x00000005, 0x00000011, 0x0000000c, 0x00050051, 0x00000003, 0x00000012, 0x00000011, 0x00000000,
    0x00050051, 0x00000003, 0x00000013, 0x00000011, 0x00000001, 0x00050051, 0x00000003, 0x00000014,
    0x00000011, 0x00000002, 0x00070050, 0x00000004, 0x00000015, 0x00000012, 0x00000013, 0x00000014,
    0x0000000e, 0x00050041, 0x0000000d, 0x00000016, 0x0000000a, 0x00000007, 0x0003003e, 0x00000016,
    0x00000015, 0x000100fd, 0x00010038,
];

// Fragment shader GLSL source:
// #version 450
// layout(location = 0) out vec4 out_color;
// void main() { out_color = vec4(1.0, 0.0, 0.0, 1.0); }
const FRAG_SPV: &[u32] = &[
    0x07230203, 0x00010000, 0x00080001, 0x0000000c, 0x00000000, 0x00020011, 0x00000001, 0x0003000e,
    0x00000000, 0x00000001, 0x0006000f, 0x00000004, 0x0000000a, 0x6e69616d, 0x00000000, 0x00000006,
    0x00030010, 0x0000000a, 0x00000007, 0x00040047, 0x00000006, 0x0000001e, 0x00000000, 0x00020013,
    0x00000001, 0x00030021, 0x00000002, 0x00000001, 0x00030016, 0x00000003, 0x00000020, 0x00040017,
    0x00000004, 0x00000003, 0x00000004, 0x00040020, 0x00000005, 0x00000003, 0x00000004, 0x0004003b,
    0x00000005, 0x00000006, 0x00000003, 0x0004002b, 0x00000003, 0x00000007, 0x3f800000, 0x0004002b,
    0x00000003, 0x00000008, 0x00000000, 0x0007002c, 0x00000004, 0x00000009, 0x00000007, 0x00000008,
    0x00000008, 0x00000007, 0x00050036, 0x00000001, 0x0000000a, 0x00000000, 0x00000002, 0x000200f8,
    0x0000000b, 0x0003003e, 0x00000006, 0x00000009, 0x000100fd, 0x00010038,
];

// ----------------------------------------------------------------------------------------------------

fn main() -> VkResult<()> {
    unsafe {
        let wnd_class = WNDCLASSW {
            lpfnWndProc: Some(window_proc),
            lpszClassName: TITLE,
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
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

        let mut rect = RECT::default();
        GetClientRect(hwnd, &mut rect).unwrap();
        let width = (rect.right - rect.left) as u32;
        let height = (rect.bottom - rect.top) as u32;

        // ----------------------------------------------------------------------------------------------------

        let entry = Entry::load().unwrap();

        let app_info = vk::ApplicationInfo {
            p_application_name: c"Minimal Vulkan by Secticide".as_ptr(),
            api_version: vk::make_api_version(0, 1, 0, 0),
            ..Default::default()
        };

        let layers = [c"VK_LAYER_KHRONOS_validation".as_ptr()];
        let instance_exts = [
            ash::khr::surface::NAME.as_ptr(),
            ash::khr::win32_surface::NAME.as_ptr(),
            ash::ext::debug_utils::NAME.as_ptr(),
        ];

        let instance = entry.create_instance(
            &vk::InstanceCreateInfo {
                p_application_info: &app_info,
                enabled_layer_count: layers.len() as u32,
                pp_enabled_layer_names: layers.as_ptr(),
                enabled_extension_count: instance_exts.len() as u32,
                pp_enabled_extension_names: instance_exts.as_ptr(),
                ..Default::default()
            },
            None,
        )?;

        // ----------------------------------------------------------------------------------------------------

        let debug_utils = ash::ext::debug_utils::Instance::new(&entry, &instance);
        let debug_messenger = debug_utils.create_debug_utils_messenger(
            &vk::DebugUtilsMessengerCreateInfoEXT {
                message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING,
                message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
                pfn_user_callback: Some(debug_callback),
                ..Default::default()
            },
            None,
        )?;

        // ----------------------------------------------------------------------------------------------------

        let win32_surface_loader = ash::khr::win32_surface::Instance::new(&entry, &instance);
        let surface = win32_surface_loader.create_win32_surface(
            &vk::Win32SurfaceCreateInfoKHR {
                hwnd: hwnd.0 as isize,
                ..Default::default()
            },
            None,
        )?;

        let surface_loader = ash::khr::surface::Instance::new(&entry, &instance);

        // ----------------------------------------------------------------------------------------------------

        let physical_devices = instance.enumerate_physical_devices()?;
        let physical_device = physical_devices[0];

        // ----------------------------------------------------------------------------------------------------

        let queue_families = instance.get_physical_device_queue_family_properties(physical_device);
        let graphics_queue_family = queue_families
            .iter()
            .enumerate()
            .find(|(i, qf)| {
                qf.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                    && surface_loader
                        .get_physical_device_surface_support(physical_device, *i as u32, surface)
                        .unwrap_or(false)
            })
            .map(|(i, _)| i as u32)
            .expect("no suitable queue family");

        // ----------------------------------------------------------------------------------------------------

        let queue_priority = [1.0f32];
        let queue_create_info = vk::DeviceQueueCreateInfo {
            queue_family_index: graphics_queue_family,
            queue_count: 1,
            p_queue_priorities: queue_priority.as_ptr(),
            ..Default::default()
        };

        let device_exts = [ash::khr::swapchain::NAME.as_ptr()];
        let device = instance.create_device(
            physical_device,
            &vk::DeviceCreateInfo {
                queue_create_info_count: 1,
                p_queue_create_infos: &queue_create_info,
                enabled_extension_count: device_exts.len() as u32,
                pp_enabled_extension_names: device_exts.as_ptr(),
                ..Default::default()
            },
            None,
        )?;

        let queue = device.get_device_queue(graphics_queue_family, 0);
        let swapchain_loader = ash::khr::swapchain::Device::new(&instance, &device);

        // ----------------------------------------------------------------------------------------------------

        let surface_caps =
            surface_loader.get_physical_device_surface_capabilities(physical_device, surface)?;

        let swapchain = swapchain_loader.create_swapchain(
            &vk::SwapchainCreateInfoKHR {
                surface,
                min_image_count: 2,
                image_format: vk::Format::B8G8R8A8_UNORM,
                image_color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
                image_extent: vk::Extent2D { width, height },
                image_array_layers: 1,
                image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
                image_sharing_mode: vk::SharingMode::EXCLUSIVE,
                pre_transform: surface_caps.current_transform,
                composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
                present_mode: vk::PresentModeKHR::FIFO,
                clipped: vk::TRUE,
                ..Default::default()
            },
            None,
        )?;

        let swapchain_images = swapchain_loader.get_swapchain_images(swapchain)?;

        // ----------------------------------------------------------------------------------------------------

        let img_views: Vec<vk::ImageView> = swapchain_images
            .iter()
            .map(|&image| {
                device
                    .create_image_view(
                        &vk::ImageViewCreateInfo {
                            image,
                            view_type: vk::ImageViewType::TYPE_2D,
                            format: vk::Format::B8G8R8A8_UNORM,
                            subresource_range: vk::ImageSubresourceRange {
                                aspect_mask: vk::ImageAspectFlags::COLOR,
                                level_count: 1,
                                layer_count: 1,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        None,
                    )
                    .unwrap()
            })
            .collect();

        // ----------------------------------------------------------------------------------------------------

        let color_attachment = vk::AttachmentDescription {
            format: vk::Format::B8G8R8A8_UNORM,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
            ..Default::default()
        };

        let color_ref = vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };

        let subpass = vk::SubpassDescription {
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            color_attachment_count: 1,
            p_color_attachments: &color_ref,
            ..Default::default()
        };

        let subpass_dep = vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            src_access_mask: vk::AccessFlags::empty(),
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            ..Default::default()
        };

        let render_pass = device.create_render_pass(
            &vk::RenderPassCreateInfo {
                attachment_count: 1,
                p_attachments: &color_attachment,
                subpass_count: 1,
                p_subpasses: &subpass,
                dependency_count: 1,
                p_dependencies: &subpass_dep,
                ..Default::default()
            },
            None,
        )?;

        // ----------------------------------------------------------------------------------------------------

        let pipeline_layout =
            device.create_pipeline_layout(&vk::PipelineLayoutCreateInfo::default(), None)?;

        // ----------------------------------------------------------------------------------------------------

        let vert_module = device.create_shader_module(
            &vk::ShaderModuleCreateInfo {
                code_size: std::mem::size_of_val(VERT_SPV),
                p_code: VERT_SPV.as_ptr(),
                ..Default::default()
            },
            None,
        )?;

        let frag_module = device.create_shader_module(
            &vk::ShaderModuleCreateInfo {
                code_size: std::mem::size_of_val(FRAG_SPV),
                p_code: FRAG_SPV.as_ptr(),
                ..Default::default()
            },
            None,
        )?;

        // ----------------------------------------------------------------------------------------------------

        let entry_point = c"main";

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo {
                stage: vk::ShaderStageFlags::VERTEX,
                module: vert_module,
                p_name: entry_point.as_ptr(),
                ..Default::default()
            },
            vk::PipelineShaderStageCreateInfo {
                stage: vk::ShaderStageFlags::FRAGMENT,
                module: frag_module,
                p_name: entry_point.as_ptr(),
                ..Default::default()
            },
        ];

        let vertex_binding = vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Float3>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        };

        let vertex_attribute = vk::VertexInputAttributeDescription {
            binding: 0,
            location: 0,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: 0,
        };

        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo {
            vertex_binding_description_count: 1,
            p_vertex_binding_descriptions: &vertex_binding,
            vertex_attribute_description_count: 1,
            p_vertex_attribute_descriptions: &vertex_attribute,
            ..Default::default()
        };

        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo {
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            ..Default::default()
        };

        let viewport = vk::Viewport {
            width: width as f32,
            height: height as f32,
            max_depth: 1.0,
            ..Default::default()
        };

        let scissor = vk::Rect2D {
            extent: vk::Extent2D { width, height },
            ..Default::default()
        };

        let viewport_state = vk::PipelineViewportStateCreateInfo {
            viewport_count: 1,
            p_viewports: &viewport,
            scissor_count: 1,
            p_scissors: &scissor,
            ..Default::default()
        };

        let rasterization_state = vk::PipelineRasterizationStateCreateInfo {
            polygon_mode: vk::PolygonMode::FILL,
            cull_mode: vk::CullModeFlags::NONE,
            front_face: vk::FrontFace::CLOCKWISE,
            line_width: 1.0,
            ..Default::default()
        };

        let multisample_state = vk::PipelineMultisampleStateCreateInfo {
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            ..Default::default()
        };

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState {
            color_write_mask: vk::ColorComponentFlags::R
                | vk::ColorComponentFlags::G
                | vk::ColorComponentFlags::B
                | vk::ColorComponentFlags::A,
            ..Default::default()
        };

        let color_blend_state = vk::PipelineColorBlendStateCreateInfo {
            attachment_count: 1,
            p_attachments: &color_blend_attachment,
            ..Default::default()
        };

        let pipeline_info = vk::GraphicsPipelineCreateInfo {
            stage_count: shader_stages.len() as u32,
            p_stages: shader_stages.as_ptr(),
            p_vertex_input_state: &vertex_input_state,
            p_input_assembly_state: &input_assembly_state,
            p_viewport_state: &viewport_state,
            p_rasterization_state: &rasterization_state,
            p_multisample_state: &multisample_state,
            p_color_blend_state: &color_blend_state,
            layout: pipeline_layout,
            render_pass,
            subpass: 0,
            ..Default::default()
        };

        let pipelines = device
            .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
            .map_err(|(_, e)| e)?;

        let pipeline = pipelines[0];

        // ----------------------------------------------------------------------------------------------------

        let framebuffers: Vec<vk::Framebuffer> = img_views
            .iter()
            .map(|view| {
                device
                    .create_framebuffer(
                        &vk::FramebufferCreateInfo {
                            render_pass,
                            attachment_count: 1,
                            p_attachments: view,
                            width,
                            height,
                            layers: 1,
                            ..Default::default()
                        },
                        None,
                    )
                    .unwrap()
            })
            .collect();

        // ----------------------------------------------------------------------------------------------------

        let cmd_pool = device.create_command_pool(
            &vk::CommandPoolCreateInfo {
                queue_family_index: graphics_queue_family,
                flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
                ..Default::default()
            },
            None,
        )?;

        let cmd_buffers = device.allocate_command_buffers(&vk::CommandBufferAllocateInfo {
            command_pool: cmd_pool,
            level: vk::CommandBufferLevel::PRIMARY,
            command_buffer_count: 1,
            ..Default::default()
        })?;

        let cmd_buffer = cmd_buffers[0];

        // ----------------------------------------------------------------------------------------------------

        let vertices = [
            Float3 {
                x: 0.0,
                y: -0.5,
                z: 0.0,
            },
            Float3 {
                x: 0.5,
                y: 0.5,
                z: 0.0,
            },
            Float3 {
                x: -0.5,
                y: 0.5,
                z: 0.0,
            },
        ];

        let buffer_size = std::mem::size_of_val(&vertices) as vk::DeviceSize;

        let vertex_buffer = device.create_buffer(
            &vk::BufferCreateInfo {
                size: buffer_size,
                usage: vk::BufferUsageFlags::VERTEX_BUFFER,
                sharing_mode: vk::SharingMode::EXCLUSIVE,
                ..Default::default()
            },
            None,
        )?;

        let mem_reqs = device.get_buffer_memory_requirements(vertex_buffer);
        let mem_props = instance.get_physical_device_memory_properties(physical_device);

        let required_flags =
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;

        let mem_type_index = (0..mem_props.memory_type_count)
            .find(|&i| {
                (mem_reqs.memory_type_bits & (1 << i)) != 0
                    && mem_props.memory_types[i as usize]
                        .property_flags
                        .contains(required_flags)
            })
            .expect("no suitable memory type");

        let vertex_buffer_memory = device.allocate_memory(
            &vk::MemoryAllocateInfo {
                allocation_size: mem_reqs.size,
                memory_type_index: mem_type_index,
                ..Default::default()
            },
            None,
        )?;

        device.bind_buffer_memory(vertex_buffer, vertex_buffer_memory, 0)?;

        // ----------------------------------------------------------------------------------------------------

        let vertex_data_begin = device.map_memory(
            vertex_buffer_memory,
            0,
            buffer_size,
            vk::MemoryMapFlags::empty(),
        )? as *mut Float3;

        std::ptr::copy_nonoverlapping::<Float3>(
            vertices.as_ptr(),
            vertex_data_begin,
            vertices.len(),
        );

        device.unmap_memory(vertex_buffer_memory);

        // ----------------------------------------------------------------------------------------------------

        let image_available = device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None)?;
        let render_finished = device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None)?;

        let in_flight_fence = device.create_fence(
            &vk::FenceCreateInfo {
                flags: vk::FenceCreateFlags::SIGNALED,
                ..Default::default()
            },
            None,
        )?;

        // ----------------------------------------------------------------------------------------------------

        let wait_stage = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
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

            device.wait_for_fences(&[in_flight_fence], true, u64::MAX)?;
            device.reset_fences(&[in_flight_fence])?;

            let (image_index, _) = swapchain_loader.acquire_next_image(
                swapchain,
                u64::MAX,
                image_available,
                vk::Fence::null(),
            )?;

            device.reset_command_buffer(cmd_buffer, vk::CommandBufferResetFlags::empty())?;
            device.begin_command_buffer(
                cmd_buffer,
                &vk::CommandBufferBeginInfo {
                    flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
                    ..Default::default()
                },
            )?;

            const CLEAR_COLOUR: [f32; 4] = [0.0, 0.2, 0.4, 1.0];
            device.cmd_begin_render_pass(
                cmd_buffer,
                &vk::RenderPassBeginInfo {
                    render_pass,
                    framebuffer: framebuffers[image_index as usize],
                    render_area: vk::Rect2D {
                        extent: vk::Extent2D { width, height },
                        ..Default::default()
                    },
                    clear_value_count: 1,
                    p_clear_values: &vk::ClearValue {
                        color: vk::ClearColorValue {
                            float32: CLEAR_COLOUR,
                        },
                    },
                    ..Default::default()
                },
                vk::SubpassContents::INLINE,
            );

            device.cmd_bind_pipeline(cmd_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline);
            device.cmd_bind_vertex_buffers(cmd_buffer, 0, &[vertex_buffer], &[0]);
            device.cmd_draw(cmd_buffer, 3, 1, 0, 0);
            device.cmd_end_render_pass(cmd_buffer);
            device.end_command_buffer(cmd_buffer)?;

            device.queue_submit(
                queue,
                &[vk::SubmitInfo {
                    wait_semaphore_count: 1,
                    p_wait_semaphores: &image_available,
                    p_wait_dst_stage_mask: wait_stage.as_ptr(),
                    command_buffer_count: 1,
                    p_command_buffers: &cmd_buffer,
                    signal_semaphore_count: 1,
                    p_signal_semaphores: &render_finished,
                    ..Default::default()
                }],
                in_flight_fence,
            )?;

            swapchain_loader.queue_present(
                queue,
                &vk::PresentInfoKHR {
                    wait_semaphore_count: 1,
                    p_wait_semaphores: &render_finished,
                    swapchain_count: 1,
                    p_swapchains: &swapchain,
                    p_image_indices: &image_index,
                    ..Default::default()
                },
            )?;
        }

        // ----------------------------------------------------------------------------------------------------

        let _ = ShowWindow(hwnd, SW_HIDE);

        device.device_wait_idle()?;

        device.destroy_fence(in_flight_fence, None);
        device.destroy_semaphore(render_finished, None);
        device.destroy_semaphore(image_available, None);
        device.free_memory(vertex_buffer_memory, None);
        device.destroy_buffer(vertex_buffer, None);
        device.destroy_command_pool(cmd_pool, None);
        for fb in &framebuffers {
            device.destroy_framebuffer(*fb, None);
        }
        device.destroy_pipeline(pipeline, None);
        device.destroy_pipeline_layout(pipeline_layout, None);
        device.destroy_shader_module(frag_module, None);
        device.destroy_shader_module(vert_module, None);
        device.destroy_render_pass(render_pass, None);
        for view in &img_views {
            device.destroy_image_view(*view, None);
        }
        swapchain_loader.destroy_swapchain(swapchain, None);
        device.destroy_device(None);
        surface_loader.destroy_surface(surface, None);
        debug_utils.destroy_debug_utils_messenger(debug_messenger, None);
        instance.destroy_instance(None);

        // ----------------------------------------------------------------------------------------------------

        Ok(())
    }
}
