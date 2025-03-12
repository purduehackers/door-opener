use std::ffi::{CStr, c_char};

use ash::vk;

use super::VulkanGUI;

pub mod devices;
pub mod swapchain;

pub fn vk_to_string(raw_string_array: &[c_char]) -> String {
    let raw_string = unsafe {
        let pointer = raw_string_array.as_ptr();
        CStr::from_ptr(pointer)
    };

    raw_string
        .to_str()
        .expect("Failed to convert vulkan raw string.")
        .to_owned()
}

pub fn vk_raw_to_string(raw_string_pointer: *const i8) -> String {
    let raw_string = unsafe { CStr::from_ptr(raw_string_pointer) };

    raw_string
        .to_str()
        .expect("Failed to convert vulkan raw string.")
        .to_owned()
}

impl VulkanGUI {
    pub fn find_memory_type(
        type_filter: u32,
        required_properties: vk::MemoryPropertyFlags,
        mem_properties: vk::PhysicalDeviceMemoryProperties,
    ) -> u32 {
        for (i, memory_type) in mem_properties.memory_types.iter().enumerate() {
            if (type_filter & (1 << i)) > 0
                && memory_type.property_flags.contains(required_properties)
            {
                return i as u32;
            }
        }

        panic!("Failed to find suitable memory type!")
    }
}
