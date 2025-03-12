use std::{collections::HashSet, ptr};

use ash::{
    khr::surface,
    vk::{self, PhysicalDevice},
};
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use crate::gui::{GUIError, VulkanSwapchainCapabilities};

use crate::gui::{VulkanCompleteSurface, VulkanGUI, vulkan_helpers::vk_to_string};

impl VulkanGUI {
    pub fn devices_create_complete_graphics_surface(
        &self,
        window: &winit::window::Window,
    ) -> Result<VulkanCompleteSurface, GUIError> {
        Ok(VulkanCompleteSurface {
            surface_loader: surface::Instance::new(&self.vulkan_entry, &self.vulkan_instance),
            surface: unsafe {
                ash_window::create_surface(
                    &self.vulkan_entry,
                    &self.vulkan_instance,
                    window.display_handle()?.into(),
                    window.window_handle()?.into(),
                    None,
                )?
            },
        })
    }

    pub fn devices_get_best_physical_device_graphics_swapchain(
        &self,
        complete_surface: &VulkanCompleteSurface,
    ) -> Result<PhysicalDevice, GUIError> {
        let physical_devices = unsafe { self.vulkan_instance.enumerate_physical_devices()? };

        let required_device_extensions = vec!["VK_KHR_swapchain".to_string()];

        let Some(physical_device) = physical_devices.iter().find(|physical_device| {
            let device_features = unsafe {
                self.vulkan_instance
                    .get_physical_device_features(**physical_device)
            };

            let is_queue_family_supported = self
                .devices_find_queue_family_graphics_swapchain(**physical_device, complete_surface)
                .is_ok();

            let is_device_extension_supported = self
                .devices_check_graphics_extension_support(
                    **physical_device,
                    &required_device_extensions,
                )
                .unwrap_or(false);

            let is_swapchain_supported = if is_device_extension_supported {
                Self::devices_check_swapchain_support(**physical_device, complete_surface)
                    .unwrap_or(false)
            } else {
                false
            };

            is_queue_family_supported
                && is_device_extension_supported
                && is_swapchain_supported
                && device_features.sampler_anisotropy == 1
        }) else {
            return Err(GUIError::NoSuitablePhysicalDevice);
        };

        Ok(*physical_device)
    }

    fn devices_check_swapchain_support(
        physical_device: vk::PhysicalDevice,
        complete_surface: &VulkanCompleteSurface,
    ) -> Result<bool, GUIError> {
        let swapchain_capabilities =
            Self::devices_get_swapchain_capabilities(physical_device, complete_surface)?;

        Ok(!swapchain_capabilities.formats.is_empty()
            && !swapchain_capabilities.present_modes.is_empty())
    }

    pub fn devices_check_graphics_extension_support(
        &self,
        physical_device: vk::PhysicalDevice,
        device_extensions: &[String],
    ) -> Result<bool, GUIError> {
        let available_extensions = unsafe {
            self.vulkan_instance
                .enumerate_device_extension_properties(physical_device)?
        };

        let mut available_extension_names = vec![];

        for extension in available_extensions.iter() {
            available_extension_names.push(vk_to_string(&extension.extension_name));
        }

        let mut required_extensions = HashSet::new();
        for extension in device_extensions.iter() {
            required_extensions.insert(extension);
        }

        for extension_name in available_extension_names.iter() {
            required_extensions.remove(extension_name);
        }

        Ok(required_extensions.is_empty())
    }

    pub fn devices_find_queue_family_graphics_swapchain(
        &self,
        physical_device: vk::PhysicalDevice,
        surface: &VulkanCompleteSurface,
    ) -> Result<(u32, u32), GUIError> {
        let queue_families = unsafe {
            self.vulkan_instance
                .get_physical_device_queue_family_properties(physical_device)
        };

        let mut graphics_queue_family_index: Option<u32> = None;
        let mut present_queue_family_index: Option<u32> = None;

        let mut index = 0;
        for queue_family in queue_families.iter() {
            if queue_family.queue_count > 0
                && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            {
                graphics_queue_family_index = Some(index);
            }

            let Ok(is_present_support) = (unsafe {
                surface.surface_loader.get_physical_device_surface_support(
                    physical_device,
                    index,
                    surface.surface,
                )
            }) else {
                index += 1;
                continue;
            };

            if queue_family.queue_count > 0 && is_present_support {
                present_queue_family_index = Some(index);
            }

            if graphics_queue_family_index.is_some() && present_queue_family_index.is_some() {
                break;
            }

            index += 1;
        }

        let Some(graphics_queue_family_index) = graphics_queue_family_index else {
            return Err(GUIError::NoValidQueue("Graphics".to_string()));
        };

        let Some(present_queue_family_index) = present_queue_family_index else {
            return Err(GUIError::NoValidQueue("Present".to_string()));
        };

        Ok((graphics_queue_family_index, present_queue_family_index))
    }

    pub fn devices_get_swapchain_capabilities(
        physical_device: vk::PhysicalDevice,
        complete_surface: &VulkanCompleteSurface,
    ) -> Result<VulkanSwapchainCapabilities, GUIError> {
        unsafe {
            let capabilities = complete_surface
                .surface_loader
                .get_physical_device_surface_capabilities(
                    physical_device,
                    complete_surface.surface,
                )?;
            let formats = complete_surface
                .surface_loader
                .get_physical_device_surface_formats(physical_device, complete_surface.surface)?;
            let present_modes = complete_surface
                .surface_loader
                .get_physical_device_surface_present_modes(
                    physical_device,
                    complete_surface.surface,
                )?;

            Ok(VulkanSwapchainCapabilities {
                capabilities,
                formats,
                present_modes,
            })
        }
    }

    pub fn devices_create_shader_module(
        device: &ash::Device,
        code: Vec<u8>,
    ) -> Result<vk::ShaderModule, GUIError> {
        let shader_module_create_info = vk::ShaderModuleCreateInfo {
            s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::ShaderModuleCreateFlags::empty(),
            code_size: code.len(),
            p_code: code.as_ptr() as *const u32,
            _marker: std::marker::PhantomData,
        };

        unsafe { Ok(device.create_shader_module(&shader_module_create_info, None)?) }
    }
}
