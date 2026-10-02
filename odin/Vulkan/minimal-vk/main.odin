package main

import "base:runtime"
import "core:fmt"
import "core:mem"
import "core:os"

import win "core:sys/windows"

import vk "vendor:vulkan"

// ----------------------------------------------------------------------------------------------------

TITLE :: "Minimal Vulkan by Secticide"

Float3 :: [3]f32

check :: proc(result: vk.Result) {
	if result < .SUCCESS {
		os.exit(int(result))
	}
}

debug_callback :: proc "system" (severity: vk.DebugUtilsMessageSeverityFlagsEXT, msg_type: vk.DebugUtilsMessageTypeFlagsEXT, data: ^vk.DebugUtilsMessengerCallbackDataEXT, user_data: rawptr) -> b32 {
	context = runtime.default_context()
	fmt.eprintln("[Vulkan]", data.pMessage)
	return false
}

// Vertex shader GLSL source:
// #version 450
// layout(location = 0) in vec3 in_position;
// void main() { gl_Position = vec4(in_position, 1.0); }
@(rodata)
VERT_SPV := [?]u32{
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
}

// Fragment shader GLSL source:
// #version 450
// layout(location = 0) out vec4 out_color;
// void main() { out_color = vec4(1.0, 0.0, 0.0, 1.0); }
@(rodata)
FRAG_SPV := [?]u32{
	0x07230203, 0x00010000, 0x00080001, 0x0000000c, 0x00000000, 0x00020011, 0x00000001, 0x0003000e,
	0x00000000, 0x00000001, 0x0006000f, 0x00000004, 0x0000000a, 0x6e69616d, 0x00000000, 0x00000006,
	0x00030010, 0x0000000a, 0x00000007, 0x00040047, 0x00000006, 0x0000001e, 0x00000000, 0x00020013,
	0x00000001, 0x00030021, 0x00000002, 0x00000001, 0x00030016, 0x00000003, 0x00000020, 0x00040017,
	0x00000004, 0x00000003, 0x00000004, 0x00040020, 0x00000005, 0x00000003, 0x00000004, 0x0004003b,
	0x00000005, 0x00000006, 0x00000003, 0x0004002b, 0x00000003, 0x00000007, 0x3f800000, 0x0004002b,
	0x00000003, 0x00000008, 0x00000000, 0x0007002c, 0x00000004, 0x00000009, 0x00000007, 0x00000008,
	0x00000008, 0x00000007, 0x00050036, 0x00000001, 0x0000000a, 0x00000000, 0x00000002, 0x000200f8,
	0x0000000b, 0x0003003e, 0x00000006, 0x00000009, 0x000100fd, 0x00010038,
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

	rect: win.RECT
	win.GetClientRect(hwnd, &rect)
	width := u32(rect.right - rect.left)
	height := u32(rect.bottom - rect.top)

	// ----------------------------------------------------------------------------------------------------

	vulkan_lib := win.LoadLibraryW(win.L("vulkan-1.dll"))
	ensure(vulkan_lib != nil, "failed to load vulkan-1.dll")
	defer win.FreeLibrary(vulkan_lib)

	vk.load_proc_addresses_global(rawptr(win.GetProcAddress(vulkan_lib, "vkGetInstanceProcAddr")))

	app_info := vk.ApplicationInfo{
		sType = .APPLICATION_INFO,
		pApplicationName = TITLE,
		apiVersion = vk.MAKE_VERSION(1, 3, 0),
	}

	layers := [?]cstring{"VK_LAYER_KHRONOS_validation"}
	instance_exts := [?]cstring{
		vk.KHR_SURFACE_EXTENSION_NAME,
		vk.KHR_WIN32_SURFACE_EXTENSION_NAME,
		vk.EXT_DEBUG_UTILS_EXTENSION_NAME,
	}

	instance: vk.Instance
	check(vk.CreateInstance(&vk.InstanceCreateInfo{
		sType = .INSTANCE_CREATE_INFO,
		pApplicationInfo = &app_info,
		enabledLayerCount = len(layers),
		ppEnabledLayerNames = raw_data(&layers),
		enabledExtensionCount = len(instance_exts),
		ppEnabledExtensionNames = raw_data(&instance_exts),
	}, nil, &instance))
	defer vk.DestroyInstance(instance, nil)

	vk.load_proc_addresses_instance(instance)

	// ----------------------------------------------------------------------------------------------------

	debug_messenger: vk.DebugUtilsMessengerEXT
	check(vk.CreateDebugUtilsMessengerEXT(instance, &vk.DebugUtilsMessengerCreateInfoEXT{
		sType = .DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
		messageSeverity = {.ERROR, .WARNING},
		messageType = {.GENERAL, .VALIDATION},
		pfnUserCallback = debug_callback,
	}, nil, &debug_messenger))
	defer vk.DestroyDebugUtilsMessengerEXT(instance, debug_messenger, nil)

	// ----------------------------------------------------------------------------------------------------

	surface: vk.SurfaceKHR
	check(vk.CreateWin32SurfaceKHR(instance, &vk.Win32SurfaceCreateInfoKHR{
		sType = .WIN32_SURFACE_CREATE_INFO_KHR,
		hwnd = hwnd,
	}, nil, &surface))
	defer vk.DestroySurfaceKHR(instance, surface, nil)

	// ----------------------------------------------------------------------------------------------------

	physical_device_count: u32
	check(vk.EnumeratePhysicalDevices(instance, &physical_device_count, nil))

	physical_devices := make([]vk.PhysicalDevice, physical_device_count)
	defer delete(physical_devices)
	check(vk.EnumeratePhysicalDevices(instance, &physical_device_count, raw_data(physical_devices)))

	physical_device := physical_devices[0]

	// ----------------------------------------------------------------------------------------------------

	queue_family_count: u32
	vk.GetPhysicalDeviceQueueFamilyProperties(physical_device, &queue_family_count, nil)

	queue_families := make([]vk.QueueFamilyProperties, queue_family_count)
	defer delete(queue_families)
	vk.GetPhysicalDeviceQueueFamilyProperties(physical_device, &queue_family_count, raw_data(queue_families))

	graphics_queue_family: u32
	found_queue_family := false

	for queue_family, i in queue_families {
		surface_support: b32
		check(vk.GetPhysicalDeviceSurfaceSupportKHR(physical_device, u32(i), surface, &surface_support))

		if .GRAPHICS in queue_family.queueFlags && surface_support {
			graphics_queue_family = u32(i)
			found_queue_family = true
			break
		}
	}

	ensure(found_queue_family, "no suitable queue family")

	// ----------------------------------------------------------------------------------------------------

	queue_priority: f32 = 1.0
	queue_create_info := vk.DeviceQueueCreateInfo{
		sType = .DEVICE_QUEUE_CREATE_INFO,
		queueFamilyIndex = graphics_queue_family,
		queueCount = 1,
		pQueuePriorities = &queue_priority,
	}

	features13 := vk.PhysicalDeviceVulkan13Features{
		sType = .PHYSICAL_DEVICE_VULKAN_1_3_FEATURES,
		dynamicRendering = true,
		synchronization2 = true,
	}

	device_exts := [?]cstring{vk.KHR_SWAPCHAIN_EXTENSION_NAME}

	device: vk.Device
	check(vk.CreateDevice(physical_device, &vk.DeviceCreateInfo{
		sType = .DEVICE_CREATE_INFO,
		pNext = &features13,
		queueCreateInfoCount = 1,
		pQueueCreateInfos = &queue_create_info,
		enabledExtensionCount = len(device_exts),
		ppEnabledExtensionNames = raw_data(&device_exts),
	}, nil, &device))
	defer vk.DestroyDevice(device, nil)

	vk.load_proc_addresses_device(device)

	queue: vk.Queue
	vk.GetDeviceQueue(device, graphics_queue_family, 0, &queue)

	// ----------------------------------------------------------------------------------------------------

	surface_caps: vk.SurfaceCapabilitiesKHR
	check(vk.GetPhysicalDeviceSurfaceCapabilitiesKHR(physical_device, surface, &surface_caps))

	swapchain: vk.SwapchainKHR
	check(vk.CreateSwapchainKHR(device, &vk.SwapchainCreateInfoKHR{
		sType = .SWAPCHAIN_CREATE_INFO_KHR,
		surface = surface,
		minImageCount = 2,
		imageFormat = .B8G8R8A8_UNORM,
		imageColorSpace = .SRGB_NONLINEAR,
		imageExtent = {width, height},
		imageArrayLayers = 1,
		imageUsage = {.COLOR_ATTACHMENT},
		imageSharingMode = .EXCLUSIVE,
		preTransform = surface_caps.currentTransform,
		compositeAlpha = {.OPAQUE},
		presentMode = .FIFO,
		clipped = true,
	}, nil, &swapchain))
	defer vk.DestroySwapchainKHR(device, swapchain, nil)

	swapchain_image_count: u32
	check(vk.GetSwapchainImagesKHR(device, swapchain, &swapchain_image_count, nil))

	swapchain_images := make([]vk.Image, swapchain_image_count)
	defer delete(swapchain_images)
	check(vk.GetSwapchainImagesKHR(device, swapchain, &swapchain_image_count, raw_data(swapchain_images)))

	// ----------------------------------------------------------------------------------------------------

	img_views := make([]vk.ImageView, len(swapchain_images))
	defer delete(img_views)

	for image, i in swapchain_images {
		check(vk.CreateImageView(device, &vk.ImageViewCreateInfo{
			sType = .IMAGE_VIEW_CREATE_INFO,
			image = image,
			viewType = .D2,
			format = .B8G8R8A8_UNORM,
			subresourceRange = {
				aspectMask = {.COLOR},
				levelCount = 1,
				layerCount = 1,
			},
		}, nil, &img_views[i]))
	}
	defer for view in img_views {
		vk.DestroyImageView(device, view, nil)
	}

	// ----------------------------------------------------------------------------------------------------

	pipeline_layout: vk.PipelineLayout
	check(vk.CreatePipelineLayout(device, &vk.PipelineLayoutCreateInfo{sType = .PIPELINE_LAYOUT_CREATE_INFO}, nil, &pipeline_layout))
	defer vk.DestroyPipelineLayout(device, pipeline_layout, nil)

	// ----------------------------------------------------------------------------------------------------

	vert_module: vk.ShaderModule
	check(vk.CreateShaderModule(device, &vk.ShaderModuleCreateInfo{
		sType = .SHADER_MODULE_CREATE_INFO,
		codeSize = size_of(VERT_SPV),
		pCode = raw_data(&VERT_SPV),
	}, nil, &vert_module))
	defer vk.DestroyShaderModule(device, vert_module, nil)

	frag_module: vk.ShaderModule
	check(vk.CreateShaderModule(device, &vk.ShaderModuleCreateInfo{
		sType = .SHADER_MODULE_CREATE_INFO,
		codeSize = size_of(FRAG_SPV),
		pCode = raw_data(&FRAG_SPV),
	}, nil, &frag_module))
	defer vk.DestroyShaderModule(device, frag_module, nil)

	// ----------------------------------------------------------------------------------------------------

	shader_stages := [?]vk.PipelineShaderStageCreateInfo{
		{
			sType = .PIPELINE_SHADER_STAGE_CREATE_INFO,
			stage = {.VERTEX},
			module = vert_module,
			pName = "main",
		},
		{
			sType = .PIPELINE_SHADER_STAGE_CREATE_INFO,
			stage = {.FRAGMENT},
			module = frag_module,
			pName = "main",
		},
	}

	vertex_binding := vk.VertexInputBindingDescription{
		binding = 0,
		stride = size_of(Float3),
		inputRate = .VERTEX,
	}

	vertex_attribute := vk.VertexInputAttributeDescription{
		binding = 0,
		location = 0,
		format = .R32G32B32_SFLOAT,
		offset = 0,
	}

	vertex_input_state := vk.PipelineVertexInputStateCreateInfo{
		sType = .PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
		vertexBindingDescriptionCount = 1,
		pVertexBindingDescriptions = &vertex_binding,
		vertexAttributeDescriptionCount = 1,
		pVertexAttributeDescriptions = &vertex_attribute,
	}

	input_assembly_state := vk.PipelineInputAssemblyStateCreateInfo{
		sType = .PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
		topology = .TRIANGLE_LIST,
	}

	viewport := vk.Viewport{
		width = f32(width),
		height = f32(height),
		maxDepth = 1.0,
	}

	scissor := vk.Rect2D{
		extent = {width, height},
	}

	viewport_state := vk.PipelineViewportStateCreateInfo{
		sType = .PIPELINE_VIEWPORT_STATE_CREATE_INFO,
		viewportCount = 1,
		pViewports = &viewport,
		scissorCount = 1,
		pScissors = &scissor,
	}

	rasterization_state := vk.PipelineRasterizationStateCreateInfo{
		sType = .PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
		polygonMode = .FILL,
		cullMode = {},
		frontFace = .CLOCKWISE,
		lineWidth = 1.0,
	}

	multisample_state := vk.PipelineMultisampleStateCreateInfo{
		sType = .PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
		rasterizationSamples = {._1},
	}

	color_blend_attachment := vk.PipelineColorBlendAttachmentState{
		colorWriteMask = {.R, .G, .B, .A},
	}

	color_blend_state := vk.PipelineColorBlendStateCreateInfo{
		sType = .PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
		attachmentCount = 1,
		pAttachments = &color_blend_attachment,
	}

	color_format := vk.Format.B8G8R8A8_UNORM
	rendering_info := vk.PipelineRenderingCreateInfo{
		sType = .PIPELINE_RENDERING_CREATE_INFO,
		colorAttachmentCount = 1,
		pColorAttachmentFormats = &color_format,
	}

	pipeline_info := vk.GraphicsPipelineCreateInfo{
		sType = .GRAPHICS_PIPELINE_CREATE_INFO,
		pNext = &rendering_info,
		stageCount = len(shader_stages),
		pStages = raw_data(&shader_stages),
		pVertexInputState = &vertex_input_state,
		pInputAssemblyState = &input_assembly_state,
		pViewportState = &viewport_state,
		pRasterizationState = &rasterization_state,
		pMultisampleState = &multisample_state,
		pColorBlendState = &color_blend_state,
		layout = pipeline_layout,
	}

	pipeline: vk.Pipeline
	check(vk.CreateGraphicsPipelines(device, 0, 1, &pipeline_info, nil, &pipeline))
	defer vk.DestroyPipeline(device, pipeline, nil)

	// ----------------------------------------------------------------------------------------------------

	cmd_pool: vk.CommandPool
	check(vk.CreateCommandPool(device, &vk.CommandPoolCreateInfo{
		sType = .COMMAND_POOL_CREATE_INFO,
		queueFamilyIndex = graphics_queue_family,
		flags = {.RESET_COMMAND_BUFFER},
	}, nil, &cmd_pool))
	defer vk.DestroyCommandPool(device, cmd_pool, nil)

	cmd_buffer: vk.CommandBuffer
	check(vk.AllocateCommandBuffers(device, &vk.CommandBufferAllocateInfo{
		sType = .COMMAND_BUFFER_ALLOCATE_INFO,
		commandPool = cmd_pool,
		level = .PRIMARY,
		commandBufferCount = 1,
	}, &cmd_buffer))

	// ----------------------------------------------------------------------------------------------------

	vertices := [?]Float3{
		{ 0.0, -0.5, 0.0},
		{ 0.5,  0.5, 0.0},
		{-0.5,  0.5, 0.0},
	}

	buffer_size := vk.DeviceSize(size_of(vertices))

	vertex_buffer: vk.Buffer
	check(vk.CreateBuffer(device, &vk.BufferCreateInfo{
		sType = .BUFFER_CREATE_INFO,
		size = buffer_size,
		usage = {.VERTEX_BUFFER},
		sharingMode = .EXCLUSIVE,
	}, nil, &vertex_buffer))
	defer vk.DestroyBuffer(device, vertex_buffer, nil)

	mem_reqs: vk.MemoryRequirements
	vk.GetBufferMemoryRequirements(device, vertex_buffer, &mem_reqs)

	mem_props: vk.PhysicalDeviceMemoryProperties
	vk.GetPhysicalDeviceMemoryProperties(physical_device, &mem_props)

	required_flags := vk.MemoryPropertyFlags{.HOST_VISIBLE, .HOST_COHERENT}

	mem_type_index: u32
	found_mem_type := false

	for i in 0 ..< mem_props.memoryTypeCount {
		if mem_reqs.memoryTypeBits & (1 << i) != 0 && mem_props.memoryTypes[i].propertyFlags >= required_flags {
			mem_type_index = i
			found_mem_type = true
			break
		}
	}

	ensure(found_mem_type, "no suitable memory type")

	vertex_buffer_memory: vk.DeviceMemory
	check(vk.AllocateMemory(device, &vk.MemoryAllocateInfo{
		sType = .MEMORY_ALLOCATE_INFO,
		allocationSize = mem_reqs.size,
		memoryTypeIndex = mem_type_index,
	}, nil, &vertex_buffer_memory))
	defer vk.FreeMemory(device, vertex_buffer_memory, nil)

	check(vk.BindBufferMemory(device, vertex_buffer, vertex_buffer_memory, 0))

	// ----------------------------------------------------------------------------------------------------

	vertex_data_begin: rawptr
	check(vk.MapMemory(device, vertex_buffer_memory, 0, buffer_size, {}, &vertex_data_begin))

	mem.copy_non_overlapping(vertex_data_begin, &vertices, size_of(vertices))

	vk.UnmapMemory(device, vertex_buffer_memory)

	// ----------------------------------------------------------------------------------------------------

	image_available: vk.Semaphore
	check(vk.CreateSemaphore(device, &vk.SemaphoreCreateInfo{sType = .SEMAPHORE_CREATE_INFO}, nil, &image_available))
	defer vk.DestroySemaphore(device, image_available, nil)

	// One per swapchain image, as a semaphore can't be re-signaled until its image is re-acquired
	render_finished := make([]vk.Semaphore, len(swapchain_images))
	defer delete(render_finished)

	for &semaphore in render_finished {
		check(vk.CreateSemaphore(device, &vk.SemaphoreCreateInfo{sType = .SEMAPHORE_CREATE_INFO}, nil, &semaphore))
	}
	defer for semaphore in render_finished {
		vk.DestroySemaphore(device, semaphore, nil)
	}

	in_flight_fence: vk.Fence
	check(vk.CreateFence(device, &vk.FenceCreateInfo{
		sType = .FENCE_CREATE_INFO,
		flags = {.SIGNALED},
	}, nil, &in_flight_fence))
	defer vk.DestroyFence(device, in_flight_fence, nil)

	// ----------------------------------------------------------------------------------------------------

	subresource_range := vk.ImageSubresourceRange{
		aspectMask = {.COLOR},
		levelCount = 1,
		layerCount = 1,
	}

	wait_stage := vk.PipelineStageFlags{.COLOR_ATTACHMENT_OUTPUT}
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

		check(vk.WaitForFences(device, 1, &in_flight_fence, true, max(u64)))
		check(vk.ResetFences(device, 1, &in_flight_fence))

		image_index: u32
		check(vk.AcquireNextImageKHR(device, swapchain, max(u64), image_available, 0, &image_index))

		check(vk.ResetCommandBuffer(cmd_buffer, {}))
		check(vk.BeginCommandBuffer(cmd_buffer, &vk.CommandBufferBeginInfo{
			sType = .COMMAND_BUFFER_BEGIN_INFO,
			flags = {.ONE_TIME_SUBMIT},
		}))

		barrier := vk.ImageMemoryBarrier2{
			sType = .IMAGE_MEMORY_BARRIER_2,
			srcStageMask = {.COLOR_ATTACHMENT_OUTPUT},
			srcAccessMask = {},
			dstStageMask = {.COLOR_ATTACHMENT_OUTPUT},
			dstAccessMask = {.COLOR_ATTACHMENT_WRITE},
			oldLayout = .UNDEFINED,
			newLayout = .COLOR_ATTACHMENT_OPTIMAL,
			image = swapchain_images[image_index],
			subresourceRange = subresource_range,
		}
		vk.CmdPipelineBarrier2(cmd_buffer, &vk.DependencyInfo{
			sType = .DEPENDENCY_INFO,
			imageMemoryBarrierCount = 1,
			pImageMemoryBarriers = &barrier,
		})

		color_attachment := vk.RenderingAttachmentInfo{
			sType = .RENDERING_ATTACHMENT_INFO,
			imageView = img_views[image_index],
			imageLayout = .COLOR_ATTACHMENT_OPTIMAL,
			loadOp = .CLEAR,
			storeOp = .STORE,
			clearValue = {color = {float32 = {0.0, 0.2, 0.4, 1.0}}},
		}
		vk.CmdBeginRendering(cmd_buffer, &vk.RenderingInfo{
			sType = .RENDERING_INFO,
			renderArea = {extent = {width, height}},
			layerCount = 1,
			colorAttachmentCount = 1,
			pColorAttachments = &color_attachment,
		})

		vertex_offset: vk.DeviceSize = 0
		vk.CmdBindPipeline(cmd_buffer, .GRAPHICS, pipeline)
		vk.CmdBindVertexBuffers(cmd_buffer, 0, 1, &vertex_buffer, &vertex_offset)
		vk.CmdDraw(cmd_buffer, 3, 1, 0, 0)
		vk.CmdEndRendering(cmd_buffer)

		barrier = vk.ImageMemoryBarrier2{
			sType = .IMAGE_MEMORY_BARRIER_2,
			srcStageMask = {.COLOR_ATTACHMENT_OUTPUT},
			srcAccessMask = {.COLOR_ATTACHMENT_WRITE},
			dstStageMask = {},
			dstAccessMask = {},
			oldLayout = .COLOR_ATTACHMENT_OPTIMAL,
			newLayout = .PRESENT_SRC_KHR,
			image = swapchain_images[image_index],
			subresourceRange = subresource_range,
		}
		vk.CmdPipelineBarrier2(cmd_buffer, &vk.DependencyInfo{
			sType = .DEPENDENCY_INFO,
			imageMemoryBarrierCount = 1,
			pImageMemoryBarriers = &barrier,
		})

		check(vk.EndCommandBuffer(cmd_buffer))

		check(vk.QueueSubmit(queue, 1, &vk.SubmitInfo{
			sType = .SUBMIT_INFO,
			waitSemaphoreCount = 1,
			pWaitSemaphores = &image_available,
			pWaitDstStageMask = &wait_stage,
			commandBufferCount = 1,
			pCommandBuffers = &cmd_buffer,
			signalSemaphoreCount = 1,
			pSignalSemaphores = &render_finished[image_index],
		}, in_flight_fence))

		check(vk.QueuePresentKHR(queue, &vk.PresentInfoKHR{
			sType = .PRESENT_INFO_KHR,
			waitSemaphoreCount = 1,
			pWaitSemaphores = &render_finished[image_index],
			swapchainCount = 1,
			pSwapchains = &swapchain,
			pImageIndices = &image_index,
		}))
	}

	// ----------------------------------------------------------------------------------------------------

	win.ShowWindow(hwnd, win.SW_HIDE)

	check(vk.DeviceWaitIdle(device))
}
