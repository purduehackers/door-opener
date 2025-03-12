mod vulkan_helpers;

use std::{collections::HashSet, ffi::NulError, mem::offset_of, ptr};

use ash::{
    LoadingError,
    vk::{
        self, ImageViewCreateInfo, InstanceCreateFlags, PresentModeKHR, SurfaceCapabilitiesKHR,
        SurfaceFormatKHR,
    },
};
use thiserror::Error;
use vulkan_helpers::vk_raw_to_string;
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    error::OsError,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    raw_window_handle::{HandleError, HasDisplayHandle, RawDisplayHandle},
    window::{Cursor, Theme, WindowAttributes, WindowButtons, WindowId, WindowLevel},
};

#[derive(Error, Debug)]
pub enum GUIError {
    #[error("Vulkan Load Error: {0}")]
    VulkanLoad(
        #[from]
        #[backtrace]
        LoadingError,
    ),

    #[error("Vulkan Error: {0}")]
    Vulkan(
        #[from]
        #[backtrace]
        vk::Result,
    ),

    #[error("CString Nul Error: {0}")]
    CStringNul(
        #[from]
        #[backtrace]
        NulError,
    ),

    #[error("Winit OS Error: {0}")]
    WinitOSError(
        #[from]
        #[backtrace]
        OsError,
    ),

    #[error("Winit Window Handle Error: {0}")]
    WinitWindowHandleError(
        #[from]
        #[backtrace]
        HandleError,
    ),

    #[error("No Suitable Physical Device")]
    NoSuitablePhysicalDevice,

    #[error("No Valid {0} Queue")]
    NoValidQueue(String),

    #[error("No Format Available")]
    NoFormatAvailable,
}

#[repr(C)]
#[derive(Clone, Debug, Copy)]
struct Vertex {
    pos: [f32; 2],
    color: [f32; 3],
}
impl Vertex {
    fn get_binding_description() -> [vk::VertexInputBindingDescription; 1] {
        [vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Vertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }]
    }

    fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 2] {
        [
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                format: vk::Format::R32G32_SFLOAT,
                offset: offset_of!(Vertex, pos) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Vertex, color) as u32,
            },
        ]
    }
}

const VERTICES_DATA: [Vertex; 3] = [
    Vertex {
        pos: [0.0, -0.5],
        color: [1.0, 1.0, 1.0],
    },
    Vertex {
        pos: [0.5, 0.5],
        color: [0.0, 1.0, 0.0],
    },
    Vertex {
        pos: [-0.5, 0.5],
        color: [0.0, 0.0, 1.0],
    },
];

pub struct VulkanCompleteSurface {
    pub surface_loader: ash::khr::surface::Instance,
    pub surface: vk::SurfaceKHR,
}

pub struct VulkanSwapchainCapabilities {
    pub capabilities: SurfaceCapabilitiesKHR,
    pub formats: Vec<SurfaceFormatKHR>,
    pub present_modes: Vec<PresentModeKHR>,
}

pub struct VulkanBaseSwapChain {
    pub swapchain_loader: ash::khr::swapchain::Device,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_images: Vec<vk::Image>,
    pub swapchain_format: vk::Format,
    pub swapchain_extent: vk::Extent2D,
}

pub struct VulkanFrameSyncObjects {
    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    pub in_flight_fences: Vec<vk::Fence>,
}

pub struct VulkanGUI {
    pub vulkan_entry: ash::Entry,
    pub vulkan_instance: ash::Instance,

    pub enabled_layers: Vec<*const i8>,

    pub vulkan_window: Option<VulkanWindow>,
}

pub struct VulkanWindow {
    pub physical_device: vk::PhysicalDevice,
    pub logical_device: ash::Device,

    pub window: winit::window::Window,
    pub window_surface: VulkanCompleteSurface,

    pub window_base_swapchain: VulkanBaseSwapChain,
    pub window_swapchain_imageviews: Vec<vk::ImageView>,
    pub window_swapchain_framebuffers: Vec<vk::Framebuffer>,

    pub gui_graphics_queue_family_index: u32,
    pub gui_present_queue_family_index: u32,
    pub gui_graphics_queue: vk::Queue,
    pub gui_present_queue: vk::Queue,

    pub gui_graphics_render_pass: vk::RenderPass,
    pub gui_graphics_pipeline_layout: vk::PipelineLayout,
    pub gui_graphics_pipeline: vk::Pipeline,

    pub TEST_vertex_buffer: vk::Buffer,
    pub TEST_vertex_buffer_memory: vk::DeviceMemory,

    pub gui_command_pool: vk::CommandPool,
    pub gui_command_buffers: Vec<vk::CommandBuffer>,

    pub gui_frame_sync_objects: VulkanFrameSyncObjects,
    pub gui_current_frame: usize,
}

impl VulkanGUI {
    pub fn new(display_handle: RawDisplayHandle) -> Result<VulkanGUI, GUIError> {
        let vulkan_entry = unsafe { ash::Entry::load()? };

        let app_name = c"Door Opener";
        let engine_name = c"Door Opener";
        let app_info = vk::ApplicationInfo {
            s_type: vk::StructureType::APPLICATION_INFO,
            p_application_name: app_name.as_ptr(),
            p_next: ptr::null(),
            application_version: 0,
            p_engine_name: engine_name.as_ptr(),
            engine_version: 0,
            api_version: vk::make_api_version(0, 1, 0, 0),
            _marker: std::marker::PhantomData,
        };

        let layers: Vec<*const i8> = vec![c"VK_LAYER_KHRONOS_validation".as_ptr()];
        let extensions: Vec<*const i8> =
            ash_window::enumerate_required_extensions(display_handle)?.to_vec();

        println!("Loading Layers:");
        for layer in layers.clone() {
            println!("\t{}", vk_raw_to_string(layer));
        }

        println!("Loading Extensions");
        for extension in extensions.clone() {
            println!("\t{}", vk_raw_to_string(extension));
        }

        let create_info = vk::InstanceCreateInfo {
            s_type: vk::StructureType::INSTANCE_CREATE_INFO,
            p_application_info: &app_info,
            p_next: ptr::null(),
            flags: InstanceCreateFlags::default(),
            enabled_layer_count: layers.len() as u32,
            pp_enabled_layer_names: layers.as_ptr(),
            enabled_extension_count: extensions.len() as u32,
            pp_enabled_extension_names: extensions.as_ptr(),
            _marker: std::marker::PhantomData,
        };

        let vulkan_instance = unsafe { vulkan_entry.create_instance(&create_info, None)? };

        Ok(VulkanGUI {
            vulkan_entry,
            vulkan_instance,

            enabled_layers: layers,

            vulkan_window: None,
        })
    }

    pub fn create_window(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<VulkanWindow, GUIError> {
        let window = event_loop.create_window(
            WindowAttributes::default()
                .with_inner_size(PhysicalSize {
                    width: 720,
                    height: 720,
                })
                .with_min_inner_size(PhysicalSize {
                    width: 720,
                    height: 720,
                })
                .with_max_inner_size(PhysicalSize {
                    width: 720,
                    height: 720,
                })
                .with_position(PhysicalPosition { x: 0, y: 0 })
                .with_resizable(false)
                .with_enabled_buttons(WindowButtons::empty())
                .with_title("Door Opener".to_string())
                .with_maximized(false)
                .with_visible(true)
                .with_transparent(false)
                .with_blur(false)
                .with_decorations(false)
                .with_window_icon(None)
                .with_theme(Some(Theme::Dark))
                .with_content_protected(false)
                .with_window_level(WindowLevel::Normal)
                .with_active(true)
                .with_cursor(Cursor::default())
                .with_fullscreen(None),
        )?;

        let window_surface = self.devices_create_complete_graphics_surface(&window)?;

        let physical_device =
            self.devices_get_best_physical_device_graphics_swapchain(&window_surface)?;

        let (gui_graphics_queue_family_index, gui_present_queue_family_index) =
            self.devices_find_queue_family_graphics_swapchain(physical_device, &window_surface)?;

        // Create Logical Device

        let mut unique_queue_families = HashSet::new();
        unique_queue_families.insert(gui_graphics_queue_family_index);
        unique_queue_families.insert(gui_present_queue_family_index);

        let queue_priorities = [1.0_f32];
        let mut queue_create_infos = vec![];
        for &queue_family in unique_queue_families.iter() {
            let queue_create_info = vk::DeviceQueueCreateInfo {
                s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::DeviceQueueCreateFlags::empty(),
                queue_family_index: queue_family,
                p_queue_priorities: queue_priorities.as_ptr(),
                queue_count: queue_priorities.len() as u32,
                _marker: std::marker::PhantomData,
            };
            queue_create_infos.push(queue_create_info);
        }

        let physical_device_features = vk::PhysicalDeviceFeatures {
            sampler_anisotropy: vk::TRUE,
            ..Default::default()
        };

        let enable_device_extensions = [c"VK_KHR_swapchain".as_ptr()];

        #[allow(deprecated)]
        let device_create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DeviceCreateFlags::empty(),
            queue_create_info_count: queue_create_infos.len() as u32,
            p_queue_create_infos: queue_create_infos.as_ptr(),

            enabled_layer_count: self.enabled_layers.len() as u32,
            pp_enabled_layer_names: self.enabled_layers.as_ptr(),
            enabled_extension_count: enable_device_extensions.len() as u32,
            pp_enabled_extension_names: enable_device_extensions.as_ptr(),
            p_enabled_features: &physical_device_features,
            _marker: std::marker::PhantomData,
        };

        let logical_device = unsafe {
            self.vulkan_instance
                .create_device(physical_device, &device_create_info, None)?
        };

        // Extract GUI Graphics Queues

        let gui_graphics_queue =
            unsafe { logical_device.get_device_queue(gui_graphics_queue_family_index, 0) };
        let gui_present_queue =
            unsafe { logical_device.get_device_queue(gui_present_queue_family_index, 0) };

        // Create Window Swapchain

        let swapchain_support =
            Self::devices_get_swapchain_capabilities(physical_device, &window_surface)?;

        let surface_format = Self::choose_gui_swapchain_format(&swapchain_support.formats)?;
        let present_mode =
            Self::choose_gui_swapchain_present_mode(&swapchain_support.present_modes);
        let extent = Self::choose_gui_swapchain_extent(&swapchain_support.capabilities, &window);

        let image_count = swapchain_support.capabilities.min_image_count + 1;
        let image_count = if swapchain_support.capabilities.max_image_count > 0 {
            image_count.min(swapchain_support.capabilities.max_image_count)
        } else {
            image_count
        };

        let (image_sharing_mode, queue_family_index_count, queue_family_indices) =
            if gui_graphics_queue_family_index != gui_present_queue_family_index {
                (
                    vk::SharingMode::CONCURRENT,
                    2,
                    vec![
                        gui_graphics_queue_family_index,
                        gui_present_queue_family_index,
                    ],
                )
            } else {
                (vk::SharingMode::EXCLUSIVE, 0, vec![])
            };

        let swapchain_create_info = vk::SwapchainCreateInfoKHR {
            s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
            p_next: ptr::null(),
            flags: vk::SwapchainCreateFlagsKHR::empty(),
            surface: window_surface.surface,
            min_image_count: image_count,
            image_color_space: surface_format.color_space,
            image_format: surface_format.format,
            image_extent: extent,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            image_sharing_mode,
            p_queue_family_indices: queue_family_indices.as_ptr(),
            queue_family_index_count,
            pre_transform: swapchain_support.capabilities.current_transform,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode,
            clipped: vk::TRUE,
            old_swapchain: vk::SwapchainKHR::null(),
            image_array_layers: 1,
            _marker: std::marker::PhantomData,
        };

        let swapchain_loader =
            ash::khr::swapchain::Device::new(&self.vulkan_instance, &logical_device);
        let swapchain = unsafe { swapchain_loader.create_swapchain(&swapchain_create_info, None)? };

        let swapchain_images = unsafe { swapchain_loader.get_swapchain_images(swapchain)? };

        let window_base_swapchain = VulkanBaseSwapChain {
            swapchain_loader,
            swapchain,
            swapchain_images,
            swapchain_format: surface_format.format,
            swapchain_extent: extent,
        };

        // Create Swapchain ImageViews

        let mut window_swapchain_imageviews: Vec<vk::ImageView> = Vec::new();

        for image in window_base_swapchain.swapchain_images.iter() {
            window_swapchain_imageviews.push(unsafe {
                logical_device.create_image_view(
                    &ImageViewCreateInfo {
                        s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
                        p_next: ptr::null(),
                        flags: vk::ImageViewCreateFlags::empty(),
                        image: *image,
                        view_type: vk::ImageViewType::TYPE_2D,
                        format: surface_format.format,
                        components: vk::ComponentMapping {
                            r: vk::ComponentSwizzle::IDENTITY,
                            g: vk::ComponentSwizzle::IDENTITY,
                            b: vk::ComponentSwizzle::IDENTITY,
                            a: vk::ComponentSwizzle::IDENTITY,
                        },
                        subresource_range: vk::ImageSubresourceRange {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            base_mip_level: 0,
                            level_count: 1,
                            base_array_layer: 0,
                            layer_count: 1,
                        },
                        _marker: std::marker::PhantomData,
                    },
                    None,
                )?
            });
        }

        // Create GUI Graphics RenderPass

        let color_attachment = vk::AttachmentDescription {
            format: surface_format.format,
            flags: vk::AttachmentDescriptionFlags::empty(),
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
        };

        let color_attachment_ref = vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };

        let subpasses = [vk::SubpassDescription {
            color_attachment_count: 1,
            p_color_attachments: &color_attachment_ref,
            p_depth_stencil_attachment: ptr::null(),
            flags: vk::SubpassDescriptionFlags::empty(),
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            input_attachment_count: 0,
            p_input_attachments: ptr::null(),
            p_resolve_attachments: ptr::null(),
            preserve_attachment_count: 0,
            p_preserve_attachments: ptr::null(),
            _marker: std::marker::PhantomData,
        }];

        let render_pass_attachments = [color_attachment];

        let subpass_dependencies = [vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            src_access_mask: vk::AccessFlags::empty(),
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dependency_flags: vk::DependencyFlags::empty(),
        }];

        let render_pass_create_info = vk::RenderPassCreateInfo {
            s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,
            flags: vk::RenderPassCreateFlags::empty(),
            p_next: ptr::null(),
            attachment_count: render_pass_attachments.len() as u32,
            p_attachments: render_pass_attachments.as_ptr(),
            subpass_count: subpasses.len() as u32,
            p_subpasses: subpasses.as_ptr(),
            dependency_count: subpass_dependencies.len() as u32,
            p_dependencies: subpass_dependencies.as_ptr(),
            _marker: std::marker::PhantomData,
        };

        let gui_graphics_render_pass =
            unsafe { logical_device.create_render_pass(&render_pass_create_info, None)? };

        // Create GUI Graphics Pipeline + Layout

        let (gui_graphics_pipeline, gui_graphics_pipeline_layout) =
            Self::create_gui_graphics_pipeline(
                &logical_device,
                gui_graphics_render_pass,
                window_base_swapchain.swapchain_extent,
            )?;

        // Create Swapchain Framebuffers

        let mut window_swapchain_framebuffers = vec![];

        for &image_view in window_swapchain_imageviews.iter() {
            let attachments = [image_view];

            let framebuffer_create_info = vk::FramebufferCreateInfo {
                s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::FramebufferCreateFlags::empty(),
                render_pass: gui_graphics_render_pass,
                attachment_count: attachments.len() as u32,
                p_attachments: attachments.as_ptr(),
                width: window_base_swapchain.swapchain_extent.width,
                height: window_base_swapchain.swapchain_extent.height,
                layers: 1,
                _marker: std::marker::PhantomData,
            };

            let framebuffer =
                unsafe { logical_device.create_framebuffer(&framebuffer_create_info, None)? };

            window_swapchain_framebuffers.push(framebuffer);
        }

        // Create GUI Command Pool

        let gui_command_pool_create_info = vk::CommandPoolCreateInfo {
            s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::CommandPoolCreateFlags::empty(),
            queue_family_index: gui_graphics_queue_family_index,
            _marker: std::marker::PhantomData,
        };

        let gui_command_pool =
            unsafe { logical_device.create_command_pool(&gui_command_pool_create_info, None)? };

        // Using a sample vertex buffer set from an ash example for testing...

        let (TEST_vertex_buffer, TEST_vertex_buffer_memory) =
            self.create_TEST_vertex_buffer(&logical_device, physical_device)?;

        // Create GUI Command Buffer

        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next: ptr::null(),
            command_buffer_count: window_swapchain_framebuffers.len() as u32,
            command_pool: gui_command_pool,
            level: vk::CommandBufferLevel::PRIMARY,
            _marker: std::marker::PhantomData,
        };

        let gui_command_buffers =
            unsafe { logical_device.allocate_command_buffers(&command_buffer_allocate_info)? };

        for (i, &command_buffer) in gui_command_buffers.iter().enumerate() {
            let command_buffer_begin_info = vk::CommandBufferBeginInfo {
                s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
                p_next: ptr::null(),
                flags: vk::CommandBufferUsageFlags::SIMULTANEOUS_USE,
                p_inheritance_info: ptr::null(),
                _marker: std::marker::PhantomData,
            };

            unsafe {
                logical_device.begin_command_buffer(command_buffer, &command_buffer_begin_info)?;
            }

            let clear_values = [vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 1.0],
                },
            }];

            let render_pass_begin_info = vk::RenderPassBeginInfo {
                s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
                p_next: ptr::null(),
                framebuffer: window_swapchain_framebuffers[i],
                render_pass: gui_graphics_render_pass,
                clear_value_count: clear_values.len() as u32,
                p_clear_values: clear_values.as_ptr(),
                render_area: vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: window_base_swapchain.swapchain_extent,
                },
                _marker: std::marker::PhantomData,
            };

            unsafe {
                logical_device.cmd_begin_render_pass(
                    command_buffer,
                    &render_pass_begin_info,
                    vk::SubpassContents::INLINE,
                );
                logical_device.cmd_bind_pipeline(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    gui_graphics_pipeline,
                );

                let vertex_buffers = [TEST_vertex_buffer];
                let offsets = [0_u64];

                logical_device.cmd_bind_vertex_buffers(
                    command_buffer,
                    0,
                    &vertex_buffers,
                    &offsets,
                );

                logical_device.cmd_draw(command_buffer, VERTICES_DATA.len() as u32, 1, 0, 0);

                logical_device.cmd_end_render_pass(command_buffer);

                logical_device.end_command_buffer(command_buffer)?;
            }
        }

        // Create Frame Sync Objects

        let mut gui_frame_sync_objects = VulkanFrameSyncObjects {
            image_available_semaphores: vec![],
            render_finished_semaphores: vec![],
            in_flight_fences: vec![],
        };

        let semaphore_create_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::SemaphoreCreateFlags::empty(),
            _marker: std::marker::PhantomData,
        };

        let fence_create_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::FenceCreateFlags::SIGNALED,
            _marker: std::marker::PhantomData,
        };

        for _ in 0..3 {
            unsafe {
                let image_available_semaphore =
                    logical_device.create_semaphore(&semaphore_create_info, None)?;
                let render_finished_semaphore =
                    logical_device.create_semaphore(&semaphore_create_info, None)?;
                let in_flight_fence = logical_device.create_fence(&fence_create_info, None)?;

                gui_frame_sync_objects
                    .image_available_semaphores
                    .push(image_available_semaphore);
                gui_frame_sync_objects
                    .render_finished_semaphores
                    .push(render_finished_semaphore);
                gui_frame_sync_objects
                    .in_flight_fences
                    .push(in_flight_fence);
            }
        }

        // Return our new Window

        Ok(VulkanWindow {
            physical_device,
            logical_device,

            window,
            window_surface,

            window_base_swapchain,
            window_swapchain_imageviews,
            window_swapchain_framebuffers,

            gui_graphics_queue_family_index,
            gui_present_queue_family_index,
            gui_graphics_queue,
            gui_present_queue,

            gui_graphics_render_pass,
            gui_graphics_pipeline_layout,
            gui_graphics_pipeline,

            TEST_vertex_buffer,
            TEST_vertex_buffer_memory,

            gui_command_pool,
            gui_command_buffers,

            gui_frame_sync_objects,
            gui_current_frame: 0,
        })
    }

    fn create_TEST_vertex_buffer(
        &self,
        device: &ash::Device,
        physical_device: vk::PhysicalDevice,
    ) -> Result<(vk::Buffer, vk::DeviceMemory), GUIError> {
        let vertex_buffer_create_info = vk::BufferCreateInfo {
            s_type: vk::StructureType::BUFFER_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::BufferCreateFlags::empty(),
            size: std::mem::size_of_val(&VERTICES_DATA) as u64,
            usage: vk::BufferUsageFlags::VERTEX_BUFFER,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: 0,
            p_queue_family_indices: ptr::null(),
            _marker: std::marker::PhantomData,
        };

        let vertex_buffer = unsafe { device.create_buffer(&vertex_buffer_create_info, None)? };

        let mem_requirements = unsafe { device.get_buffer_memory_requirements(vertex_buffer) };
        let mem_properties = unsafe {
            self.vulkan_instance
                .get_physical_device_memory_properties(physical_device)
        };
        let required_memory_flags: vk::MemoryPropertyFlags =
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
        let memory_type = VulkanGUI::find_memory_type(
            mem_requirements.memory_type_bits,
            required_memory_flags,
            mem_properties,
        );

        let allocate_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            p_next: ptr::null(),
            allocation_size: mem_requirements.size,
            memory_type_index: memory_type,
            _marker: std::marker::PhantomData,
        };

        let vertex_buffer_memory = unsafe { device.allocate_memory(&allocate_info, None)? };

        unsafe {
            device.bind_buffer_memory(vertex_buffer, vertex_buffer_memory, 0)?;

            let data_ptr = device.map_memory(
                vertex_buffer_memory,
                0,
                vertex_buffer_create_info.size,
                vk::MemoryMapFlags::empty(),
            )? as *mut Vertex;

            data_ptr.copy_from_nonoverlapping(VERTICES_DATA.as_ptr(), VERTICES_DATA.len());

            device.unmap_memory(vertex_buffer_memory);
        }

        Ok((vertex_buffer, vertex_buffer_memory))
    }
}

impl Drop for VulkanGUI {
    fn drop(&mut self) {
        unsafe {
            if let Some(vulkan_window) = &self.vulkan_window {
                for i in 0..3 {
                    vulkan_window.logical_device.destroy_semaphore(
                        vulkan_window
                            .gui_frame_sync_objects
                            .image_available_semaphores[i],
                        None,
                    );
                    vulkan_window.logical_device.destroy_semaphore(
                        vulkan_window
                            .gui_frame_sync_objects
                            .render_finished_semaphores[i],
                        None,
                    );
                    vulkan_window.logical_device.destroy_fence(
                        vulkan_window.gui_frame_sync_objects.in_flight_fences[i],
                        None,
                    );
                }

                vulkan_window.logical_device.free_command_buffers(
                    vulkan_window.gui_command_pool,
                    &vulkan_window.gui_command_buffers,
                );

                for &framebuffer in vulkan_window.window_swapchain_framebuffers.iter() {
                    vulkan_window
                        .logical_device
                        .destroy_framebuffer(framebuffer, None);
                }

                vulkan_window
                    .logical_device
                    .destroy_pipeline(vulkan_window.gui_graphics_pipeline, None);
                vulkan_window
                    .logical_device
                    .destroy_pipeline_layout(vulkan_window.gui_graphics_pipeline_layout, None);

                vulkan_window
                    .logical_device
                    .destroy_render_pass(vulkan_window.gui_graphics_render_pass, None);

                for &image_view in vulkan_window.window_swapchain_imageviews.iter() {
                    vulkan_window
                        .logical_device
                        .destroy_image_view(image_view, None);
                }

                vulkan_window
                    .window_base_swapchain
                    .swapchain_loader
                    .destroy_swapchain(vulkan_window.window_base_swapchain.swapchain, None);

                vulkan_window
                    .logical_device
                    .destroy_buffer(vulkan_window.TEST_vertex_buffer, None);
                vulkan_window
                    .logical_device
                    .free_memory(vulkan_window.TEST_vertex_buffer_memory, None);

                vulkan_window
                    .logical_device
                    .destroy_command_pool(vulkan_window.gui_command_pool, None);

                vulkan_window.logical_device.destroy_device(None);
                vulkan_window
                    .window_surface
                    .surface_loader
                    .destroy_surface(vulkan_window.window_surface.surface, None);
            }

            self.vulkan_instance.destroy_instance(None);
        }
    }
}

impl VulkanGUI {
    fn create_gui_graphics_pipeline(
        logical_device: &ash::Device,
        render_pass: vk::RenderPass,
        swapchain_extent: vk::Extent2D,
    ) -> Result<(vk::Pipeline, vk::PipelineLayout), GUIError> {
        let vert_shader_module = VulkanGUI::devices_create_shader_module(
            logical_device,
            include_bytes!("./shaders/test-vertexbuffer.vert.spv").to_vec(),
        )?;
        let frag_shader_module = VulkanGUI::devices_create_shader_module(
            logical_device,
            include_bytes!("./shaders/test-vertexbuffer.frag.spv").to_vec(),
        )?;

        let main_function_name = c"main"; // the beginning function name in shader code.

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo {
                // Vertex Shader
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::PipelineShaderStageCreateFlags::empty(),
                module: vert_shader_module,
                p_name: main_function_name.as_ptr(),
                stage: vk::ShaderStageFlags::VERTEX,
                p_specialization_info: ptr::null(),
                _marker: std::marker::PhantomData,
            },
            vk::PipelineShaderStageCreateInfo {
                // Fragment Shader
                s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::PipelineShaderStageCreateFlags::empty(),
                module: frag_shader_module,
                p_name: main_function_name.as_ptr(),
                stage: vk::ShaderStageFlags::FRAGMENT,
                p_specialization_info: ptr::null(),
                _marker: std::marker::PhantomData,
            },
        ];

        let binding_description = Vertex::get_binding_description();
        let attribute_description = Vertex::get_attribute_descriptions();

        let vertex_input_state_create_info = vk::PipelineVertexInputStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineVertexInputStateCreateFlags::empty(),
            vertex_attribute_description_count: attribute_description.len() as u32,
            p_vertex_attribute_descriptions: attribute_description.as_ptr(),
            vertex_binding_description_count: binding_description.len() as u32,
            p_vertex_binding_descriptions: binding_description.as_ptr(),
            _marker: std::marker::PhantomData,
        };
        let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
            flags: vk::PipelineInputAssemblyStateCreateFlags::empty(),
            p_next: ptr::null(),
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            primitive_restart_enable: vk::FALSE,
            _marker: std::marker::PhantomData,
        };

        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: swapchain_extent.width as f32,
            height: swapchain_extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];

        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: swapchain_extent,
        }];

        let viewport_state_create_info = vk::PipelineViewportStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineViewportStateCreateFlags::empty(),
            scissor_count: scissors.len() as u32,
            p_scissors: scissors.as_ptr(),
            viewport_count: viewports.len() as u32,
            p_viewports: viewports.as_ptr(),
            _marker: std::marker::PhantomData,
        };

        let rasterization_statue_create_info = vk::PipelineRasterizationStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineRasterizationStateCreateFlags::empty(),
            cull_mode: vk::CullModeFlags::BACK,
            front_face: vk::FrontFace::CLOCKWISE,
            line_width: 1.0,
            polygon_mode: vk::PolygonMode::FILL,
            rasterizer_discard_enable: vk::FALSE,
            depth_clamp_enable: vk::FALSE,
            depth_bias_clamp: 0.0,
            depth_bias_constant_factor: 0.0,
            depth_bias_enable: vk::FALSE,
            depth_bias_slope_factor: 0.0,
            _marker: std::marker::PhantomData,
        };

        let multisample_state_create_info = vk::PipelineMultisampleStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
            flags: vk::PipelineMultisampleStateCreateFlags::empty(),
            p_next: ptr::null(),
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            sample_shading_enable: vk::FALSE,
            min_sample_shading: 0.0,
            p_sample_mask: ptr::null(),
            alpha_to_one_enable: vk::FALSE,
            alpha_to_coverage_enable: vk::FALSE,
            _marker: std::marker::PhantomData,
        };

        let stencil_state = vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::ALWAYS,
            compare_mask: 0,
            write_mask: 0,
            reference: 0,
        };

        let depth_state_create_info = vk::PipelineDepthStencilStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineDepthStencilStateCreateFlags::empty(),
            depth_test_enable: vk::FALSE,
            depth_write_enable: vk::FALSE,
            depth_compare_op: vk::CompareOp::LESS_OR_EQUAL,
            depth_bounds_test_enable: vk::FALSE,
            stencil_test_enable: vk::FALSE,
            front: stencil_state,
            back: stencil_state,
            max_depth_bounds: 1.0,
            min_depth_bounds: 0.0,
            _marker: std::marker::PhantomData,
        };

        let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState {
            blend_enable: vk::FALSE,
            color_write_mask: vk::ColorComponentFlags::RGBA,
            src_color_blend_factor: vk::BlendFactor::ONE,
            dst_color_blend_factor: vk::BlendFactor::ZERO,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ONE,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
        }];

        let color_blend_state = vk::PipelineColorBlendStateCreateInfo {
            s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineColorBlendStateCreateFlags::empty(),
            logic_op_enable: vk::FALSE,
            logic_op: vk::LogicOp::COPY,
            attachment_count: color_blend_attachment_states.len() as u32,
            p_attachments: color_blend_attachment_states.as_ptr(),
            blend_constants: [0.0, 0.0, 0.0, 0.0],
            _marker: std::marker::PhantomData,
        };

        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo {
            s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineLayoutCreateFlags::empty(),
            set_layout_count: 0,
            p_set_layouts: ptr::null(),
            push_constant_range_count: 0,
            p_push_constant_ranges: ptr::null(),
            _marker: std::marker::PhantomData,
        };

        let pipeline_layout =
            unsafe { logical_device.create_pipeline_layout(&pipeline_layout_create_info, None)? };

        let graphic_pipeline_create_infos = [vk::GraphicsPipelineCreateInfo {
            s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::PipelineCreateFlags::empty(),
            stage_count: shader_stages.len() as u32,
            p_stages: shader_stages.as_ptr(),
            p_vertex_input_state: &vertex_input_state_create_info,
            p_input_assembly_state: &vertex_input_assembly_state_info,
            p_tessellation_state: ptr::null(),
            p_viewport_state: &viewport_state_create_info,
            p_rasterization_state: &rasterization_statue_create_info,
            p_multisample_state: &multisample_state_create_info,
            p_depth_stencil_state: &depth_state_create_info,
            p_color_blend_state: &color_blend_state,
            p_dynamic_state: ptr::null(),
            layout: pipeline_layout,
            render_pass,
            subpass: 0,
            base_pipeline_handle: vk::Pipeline::null(),
            base_pipeline_index: -1,
            _marker: std::marker::PhantomData,
        }];

        let graphics_pipelines = unsafe {
            match logical_device.create_graphics_pipelines(
                vk::PipelineCache::null(),
                &graphic_pipeline_create_infos,
                None,
            ) {
                Ok(pipelines) => pipelines,
                Err((_, vulkan_result)) => {
                    return Err(GUIError::from(vulkan_result));
                }
            }
        };

        unsafe {
            logical_device.destroy_shader_module(vert_shader_module, None);
            logical_device.destroy_shader_module(frag_shader_module, None);
        }

        Ok((graphics_pipelines[0], pipeline_layout))
    }

    fn draw_frame(vulkan_window: &mut VulkanWindow) {
        let wait_fences =
            [vulkan_window.gui_frame_sync_objects.in_flight_fences
                [vulkan_window.gui_current_frame]];

        unsafe {
            vulkan_window
                .logical_device
                .wait_for_fences(&wait_fences, true, u64::MAX)
                .expect("Failed to wait for Fence!");
        }

        let (image_index, _is_sub_optimal) = unsafe {
            let result = vulkan_window
                .window_base_swapchain
                .swapchain_loader
                .acquire_next_image(
                    vulkan_window.window_base_swapchain.swapchain,
                    u64::MAX,
                    vulkan_window
                        .gui_frame_sync_objects
                        .image_available_semaphores[vulkan_window.gui_current_frame],
                    vk::Fence::null(),
                );
            match result {
                Ok(image_index) => image_index,
                Err(vk_result) => match vk_result {
                    vk::Result::ERROR_OUT_OF_DATE_KHR => {
                        // i should recreate the swapchain here with the new surface properties but that's a problem for later me :3
                        return;
                    }
                    _ => panic!("Failed to acquire Swap Chain Image!"),
                },
            }
        };

        let wait_semaphores = [vulkan_window
            .gui_frame_sync_objects
            .image_available_semaphores[vulkan_window.gui_current_frame]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [vulkan_window
            .gui_frame_sync_objects
            .render_finished_semaphores[vulkan_window.gui_current_frame]];

        let submit_infos = [vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: ptr::null(),
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: &vulkan_window.gui_command_buffers[image_index as usize],
            signal_semaphore_count: signal_semaphores.len() as u32,
            p_signal_semaphores: signal_semaphores.as_ptr(),
            _marker: std::marker::PhantomData,
        }];

        unsafe {
            vulkan_window
                .logical_device
                .reset_fences(&wait_fences)
                .expect("Failed to reset Fence!");

            vulkan_window
                .logical_device
                .queue_submit(
                    vulkan_window.gui_graphics_queue,
                    &submit_infos,
                    vulkan_window.gui_frame_sync_objects.in_flight_fences
                        [vulkan_window.gui_current_frame],
                )
                .expect("Failed to execute queue submit.");
        }

        let swapchains = [vulkan_window.window_base_swapchain.swapchain];

        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PRESENT_INFO_KHR,
            p_next: ptr::null(),
            wait_semaphore_count: 1,
            p_wait_semaphores: signal_semaphores.as_ptr(),
            swapchain_count: 1,
            p_swapchains: swapchains.as_ptr(),
            p_image_indices: &image_index,
            p_results: ptr::null_mut(),
            _marker: std::marker::PhantomData,
        };

        let _ = unsafe {
            vulkan_window
                .window_base_swapchain
                .swapchain_loader
                .queue_present(vulkan_window.gui_present_queue, &present_info)
        };

        vulkan_window.gui_current_frame = (vulkan_window.gui_current_frame + 1) % 3;
    }
}

impl ApplicationHandler for VulkanGUI {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.vulkan_window.is_none() {
            let new_vulkan_window = match self.create_window(event_loop) {
                Ok(new_vulkan_window) => new_vulkan_window,
                Err(gui_error) => {
                    println!("Vulkan Window Create Error: {}", gui_error);

                    event_loop.exit();

                    return;
                }
            };

            self.vulkan_window = Some(new_vulkan_window);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        #[allow(clippy::single_match)] // Could be adding more later, am lazy
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            _ => (),
        };

        let vulkan_window = match &mut self.vulkan_window {
            Some(e) => e,
            None => return,
        };

        if id != vulkan_window.window.id() {
            return;
        }

        match event {
            WindowEvent::RedrawRequested => {
                Self::draw_frame(vulkan_window);

                vulkan_window.window.request_redraw();
            }
            WindowEvent::Destroyed => {
                unsafe {
                    if vulkan_window.logical_device.device_wait_idle().is_err() {
                        println!("Failed to wait for idle on logical device");
                    }
                };
            }
            _ => (),
        }
    }
}

pub fn gui_entry() {
    let event_loop = match EventLoop::new() {
        Ok(event_loop) => event_loop,
        Err(event_loop_error) => {
            println!("Event Loop Creation Error: {}", event_loop_error);

            return;
        }
    };

    event_loop.set_control_flow(ControlFlow::Poll);

    let display_handle = match event_loop.display_handle() {
        Ok(display_handle) => display_handle,
        Err(handle_error) => {
            println!("Winit Display Handle Error: {}", handle_error);

            return;
        }
    };

    let mut vulkan_gui = match VulkanGUI::new(display_handle.as_raw()) {
        Ok(vulkan_gui) => vulkan_gui,
        Err(gui_error) => {
            println!("Vulkan Init Error: {}", gui_error);

            return;
        }
    };

    if let Err(event_loop_error) = event_loop.run_app(&mut vulkan_gui) {
        println!("Event Loop Error: {}", event_loop_error);
    }
}
