use crate::{resource::Resources, swap_chain::SwapChain};
use ash::{vk, Device};
use derive_more::{From, Deref};

#[derive(new, Debug, From, Deref)]
pub struct FrameBuffers(pub(crate) Vec<vk::Framebuffer>);

pub(crate) fn create_frame_buffers(
    swap_chain: &SwapChain,
    render_pass: vk::RenderPass,
    device: &Device,
    resources: &Resources,
) -> FrameBuffers {
    let mut frame_buffers = Vec::new();

    for i in 0..swap_chain.images.len() {
        let attachments = &[
            resources.colour.view,
            resources.depth.view,
            swap_chain.image_views[i],
        ];

        let frame_buffer_info = vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass)
            .attachments(attachments)
            .width(swap_chain.extent.width)
            .height(swap_chain.extent.height)
            .layers(1);

        unsafe {
            frame_buffers.push(
                device
                    .create_framebuffer(&frame_buffer_info, None)
                    .expect("Failed to create Frame Buffer!"),
            );
        }
    }

    frame_buffers.into()
}