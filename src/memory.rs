use crate::device::Devices;
use ash::{util::Align, vk, Device, Instance};

pub fn find_memory_type(
    instance: &Instance,
    devices: &Devices,
    type_filter: u32,
    properties: vk::MemoryPropertyFlags,
) -> u32 {
    unsafe {
        let mem_properties = instance.get_physical_device_memory_properties(devices.physical);

        for i in 0..mem_properties.memory_type_count {
            if ((1 << i) & type_filter) != 0
                && mem_properties.memory_types[i as usize].property_flags & properties == properties
            {
                return i;
            }
        }

        panic!("Failed to find suitable memory type!")
    }
}

/// # Safety
///
/// Expand on the safety of this function
pub unsafe fn map_memory<T>(
    device: &Device,
    device_memory: vk::DeviceMemory,
    device_size: vk::DeviceSize,
    to_map: &[T],
) where
    T: std::marker::Copy,
{
    let data = device
        .map_memory(device_memory, 0, device_size, vk::MemoryMapFlags::empty())
        .unwrap();
    let mut vert_align = Align::new(data, std::mem::align_of::<T>() as u64, device_size);
    vert_align.copy_from_slice(to_map);
    device.unmap_memory(device_memory);
}
