use crate::{
    device::{self, Devices},
    utility::InstanceDevices,
};
use ash::{
    extensions::khr::{Surface, Swapchain},
    vk::{self, PresentModeKHR, SurfaceCapabilitiesKHR, SurfaceFormatKHR},
};
use winit::window::Window;

#[derive(new)]
pub(crate) struct SwapChainSupport {
    capabilities: SurfaceCapabilitiesKHR,
    surface_formats: Vec<SurfaceFormatKHR>,
    present_modes: Vec<PresentModeKHR>,
}

pub struct SwapChain {
    pub loader: Swapchain,
    pub swap_chain: vk::SwapchainKHR,
    pub image_format: vk::Format,
    pub extent: vk::Extent2D,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
}

impl SwapChain {
    pub fn new(
        InstanceDevices { instance, devices }: &InstanceDevices,
        surface: vk::SurfaceKHR,
        surface_loader: &Surface,
        window: &Window,
    ) -> SwapChain {
        let SwapChainSupport {
            capabilities,
            surface_formats,
            present_modes,
        } = query_swap_chain_support(devices, surface, surface_loader);

        let surface_format = choose_swap_surface_format(&surface_formats);

        let present_mode = choose_present_mode(present_modes);

        let extent = choose_swap_extent(capabilities, window);

        let mut swap_chain_image_count = capabilities.min_image_count + 1;

        if capabilities.max_image_count > 0 && swap_chain_image_count > capabilities.max_image_count
        {
            swap_chain_image_count = capabilities.max_image_count;
        }

        let mut create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface)
            .min_image_count(swap_chain_image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::TRANSIENT_ATTACHMENT)
            .pre_transform(capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .old_swapchain(vk::SwapchainKHR::null());

        let queue_family_indices =
            device::find_queue_family(instance, devices.physical.device, surface_loader, &surface);

        if queue_family_indices.graphics_family != queue_family_indices.present_family {
            create_info.image_sharing_mode = vk::SharingMode::CONCURRENT;
            create_info.queue_family_index_count = 2;

            let queue_family_indices_arr = [
                queue_family_indices.graphics_family.unwrap(),
                queue_family_indices.present_family.unwrap(),
            ];

            create_info.p_queue_family_indices = queue_family_indices_arr.as_ptr();
        } else {
            create_info.image_sharing_mode = vk::SharingMode::EXCLUSIVE;
        }

        let swap_chain = Swapchain::new(instance, &devices.logical.device);

        unsafe {
            let swap_chain_khr = swap_chain
                .create_swapchain(&create_info, None)
                .expect("Failed to create swapchain");

            let swap_chain_images = swap_chain
                .get_swapchain_images(swap_chain_khr)
                .expect("Could not get swapchain images");

            let image_views = create_image_views(devices, &swap_chain_images, &surface_format, 1);

            SwapChain {
                loader: swap_chain,
                swap_chain: swap_chain_khr,
                images: swap_chain_images,
                image_format: surface_format.format,
                extent,
                image_views,
            }
        }
    }
}

fn create_image_views(
    devices: &Devices,
    swap_chain_images: &[vk::Image],
    surface_format: &vk::SurfaceFormatKHR,
    mip_levels: u32,
) -> Vec<vk::ImageView> {
    let mut swap_chain_image_views = vec![];

    let components = vk::ComponentMapping::builder()
        .r(vk::ComponentSwizzle::IDENTITY)
        .g(vk::ComponentSwizzle::IDENTITY)
        .b(vk::ComponentSwizzle::IDENTITY)
        .a(vk::ComponentSwizzle::IDENTITY)
        .build();

    let sub_resource_range = vk::ImageSubresourceRange::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .base_mip_level(0)
        .level_count(mip_levels)
        .base_array_layer(0)
        .layer_count(1)
        .build();

    for &image in swap_chain_images.iter() {
        let image_view_create_info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(surface_format.format)
            .components(components)
            .subresource_range(sub_resource_range);

        let image_view = unsafe {
            devices
                .logical
                .device
                .create_image_view(&image_view_create_info, None)
                .expect("Failed to create vk::Image View!")
        };

        swap_chain_image_views.push(image_view);
    }

    swap_chain_image_views
}

pub(crate) fn query_swap_chain_support(
    devices: &Devices,
    surface: vk::SurfaceKHR,
    surface_loader: &Surface,
) -> SwapChainSupport {
    let capabilities = unsafe {
        surface_loader
            .get_physical_device_surface_capabilities(devices.physical.device, surface)
            .unwrap()
    };

    let formats = unsafe {
        surface_loader
            .get_physical_device_surface_formats(devices.physical.device, surface)
            .expect("Could not get Physical Device Surface Formats")
    };

    let present_modes = unsafe {
        surface_loader
            .get_physical_device_surface_present_modes(devices.physical.device, surface)
            .expect("Could not get Physical Device Present Modes")
    };

    SwapChainSupport::new(capabilities, formats, present_modes)
}

fn choose_swap_surface_format(formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
    for format in formats {
        if format.format == vk::Format::R8G8B8A8_SRGB
            && format.color_space == vk::ColorSpaceKHR::EXTENDED_SRGB_NONLINEAR_EXT
        {
            return *format;
        }
    }

    formats[0]
}

fn choose_present_mode(present_modes: Vec<vk::PresentModeKHR>) -> vk::PresentModeKHR {
    for present_mode in present_modes {
        if present_mode == vk::PresentModeKHR::MAILBOX {
            return present_mode;
        }
    }
    vk::PresentModeKHR::FIFO
}

fn choose_swap_extent(capabilities: vk::SurfaceCapabilitiesKHR, window: &Window) -> vk::Extent2D {
    if capabilities.current_extent.width != u32::MAX {
        capabilities.current_extent
    } else {
        let size = window.inner_size();

        vk::Extent2D {
            width: size.width.clamp(
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
            ),
            height: size.height.clamp(
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
            ),
        }
    }
}
