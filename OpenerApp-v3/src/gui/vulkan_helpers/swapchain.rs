use ash::vk::{self};

use crate::gui::GUIError;

use crate::gui::VulkanGUI;

impl VulkanGUI {
    pub fn choose_gui_swapchain_format(
        available_formats: &Vec<vk::SurfaceFormatKHR>,
    ) -> Result<vk::SurfaceFormatKHR, GUIError> {
        for available_format in available_formats {
            if available_format.format == vk::Format::B8G8R8A8_SRGB
                && available_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            {
                return Ok(*available_format);
            }
        }

        let Some(first_available_format) = available_formats.first() else {
            return Err(GUIError::NoFormatAvailable);
        };

        Ok(*first_available_format)
    }

    pub fn choose_gui_swapchain_present_mode(
        available_present_modes: &[vk::PresentModeKHR],
    ) -> vk::PresentModeKHR {
        for &available_present_mode in available_present_modes.iter() {
            if available_present_mode == vk::PresentModeKHR::MAILBOX {
                return available_present_mode;
            }
        }

        vk::PresentModeKHR::FIFO
    }

    pub fn choose_gui_swapchain_extent(
        capabilities: &vk::SurfaceCapabilitiesKHR,
        window: &winit::window::Window,
    ) -> vk::Extent2D {
        if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            let window_size = window.inner_size();

            vk::Extent2D {
                width: window_size.width.clamp(
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: window_size.height.clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            }
        }
    }
}
