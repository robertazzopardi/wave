use crate::{device::Devices, resource, swapchain::SwapChain};
use ash::{vk, Instance};

pub fn create_render_pass(
    instance: &Instance,
    devices: &Devices,
    swapchain: &SwapChain,
) -> vk::RenderPass {
    let renderpass_attachments = [
        vk::AttachmentDescription {
            format: swapchain.image_format,
            samples: devices.msaa_samples,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::DONT_CARE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            ..Default::default()
        },
        vk::AttachmentDescription {
            format: resource::find_depth_format(instance, &devices.physical),
            samples: devices.msaa_samples,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::DONT_CARE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            ..Default::default()
        },
        vk::AttachmentDescription {
            format: swapchain.image_format,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::DONT_CARE,
            store_op: vk::AttachmentStoreOp::DONT_CARE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
            ..Default::default()
        },
    ];

    let color_attachment_refs = vk::AttachmentReference {
        attachment: 0,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
    };
    let depth_attachment_ref = vk::AttachmentReference {
        attachment: 1,
        layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
    };
    let color_attachment_resolver_ref = vk::AttachmentReference {
        attachment: 2,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
    };

    let subpasses = vk::SubpassDescription::builder()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(std::slice::from_ref(&color_attachment_refs))
        .depth_stencil_attachment(&depth_attachment_ref)
        .resolve_attachments(std::slice::from_ref(&color_attachment_resolver_ref));

    let dependencies = vk::SubpassDependency::builder()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .dst_subpass(0)
        .src_access_mask(vk::AccessFlags::NONE_KHR)
        .src_stage_mask(
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
        )
        .dst_stage_mask(
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
        )
        .dst_access_mask(
            vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
        );

    let renderpass_create_info = vk::RenderPassCreateInfo::builder()
        .attachments(&renderpass_attachments)
        .subpasses(std::slice::from_ref(&subpasses))
        .dependencies(std::slice::from_ref(&dependencies));

    unsafe {
        devices
            .logical
            .create_render_pass(&renderpass_create_info, None)
            .expect("Failed to create render pass!")
    }
}
