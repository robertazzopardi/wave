use crate::{
    l2d::square::square_from_vertices,
    utility::{self, calculate_indices},
    GeomBehavior, Geometry, VerticesAndIndices,
};
use derive_builder::Builder;
use derive_more::{Deref, DerefMut};
use lambda_space::space::{Coordinate3, Orientation};
use lambda_vulkan::{
    buffer::ModelBuffers, command_buffer::CommandPool, graphics_pipeline::GraphicsPipeline,
    swap_chain::SwapChain, texture::Texture, utility::InstanceDevices, RenderPass, VulkanObject,
};

pub const CUBE_VERTICES: [[f32; 3]; 36] = [
    [1.0, 1.0, -1.0],
    [-1.0, 1.0, -1.0],
    [-1.0, 1.0, 1.0],
    [1.0, 1.0, -1.0],
    [-1.0, 1.0, 1.0],
    [1.0, 1.0, 1.0],
    [1.0, -1.0, 1.0],
    [1.0, 1.0, 1.0],
    [-1.0, 1.0, 1.0],
    [1.0, -1.0, 1.0],
    [-1.0, 1.0, 1.0],
    [-1.0, -1.0, 1.0],
    [-1.0, -1.0, 1.0],
    [-1.0, 1.0, 1.0],
    [-1.0, 1.0, -1.0],
    [-1.0, -1.0, 1.0],
    [-1.0, 1.0, -1.0],
    [-1.0, -1.0, -1.0],
    [-1.0, -1.0, -1.0],
    [1.0, -1.0, -1.0],
    [1.0, -1.0, 1.0],
    [-1.0, -1.0, -1.0],
    [1.0, -1.0, 1.0],
    [-1.0, -1.0, 1.0],
    [1.0, -1.0, -1.0],
    [1.0, 1.0, -1.0],
    [1.0, 1.0, 1.0],
    [1.0, -1.0, -1.0],
    [1.0, 1.0, 1.0],
    [1.0, -1.0, 1.0],
    [-1.0, -1.0, -1.0],
    [-1.0, 1.0, -1.0],
    [1.0, 1.0, -1.0],
    [-1.0, -1.0, -1.0],
    [1.0, 1.0, -1.0],
    [1.0, -1.0, -1.0],
];

#[derive(Builder, Default, Debug, Clone, Copy, new)]
#[builder(default)]
pub struct CubeInfo {
    pub position: Coordinate3,
    pub orientation: Orientation,
    pub radius: f32,
}

#[derive(new, Deref, DerefMut)]
pub struct Cube(Geometry<CubeInfo>);

impl GeomBehavior for Cube {
    fn vertices_and_indices(&mut self) -> VerticesAndIndices {
        let mut vertices = square_from_vertices(&CUBE_VERTICES);

        vertices.chunks_mut(4).for_each(|face| {
            utility::calculate_normals(face);
            utility::scale(face, self.properties.radius);
        });

        let indices = calculate_indices(&vertices);

        self.vulkan_object.vertices_and_indices = Some(VerticesAndIndices::new(vertices, indices));
        self.vulkan_object.vertices_and_indices.clone().unwrap()
    }

    fn vulkan_object(&self) -> VulkanObject {
        self.vulkan_object.clone()
    }

    fn defer_build(
        &mut self,
        command_pool: &CommandPool,
        command_buffer_count: u32,
        swap_chain: &SwapChain,
        render_pass: &RenderPass,
        instance_devices: &InstanceDevices,
    ) {
        if let Some(texture) = self.texture.clone() {
            self.vulkan_object.texture_buffer =
                Some(Texture::new(&texture, command_pool, instance_devices));
        }

        let vertices_and_indices = self.vertices_and_indices();

        let model_buffers = ModelBuffers::new(
            &vertices_and_indices,
            command_pool,
            command_buffer_count,
            instance_devices,
        );

        self.vulkan_object.buffers = Some(model_buffers);

        self.vulkan_object.graphics_pipeline = Some(GraphicsPipeline::new(
            swap_chain,
            render_pass.0,
            &self.vulkan_object.texture_buffer,
            self.topology,
            self.cull_mode,
            instance_devices,
            self.shader,
        ));
    }
}
