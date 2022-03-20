pub mod l2d;
pub mod l3d;
pub mod utility;

use self::utility::{ModelCullMode, ModelTopology};
use crate::{
    device::Devices, pipeline::GraphicsPipeline, space::Orientation, swap_chain::SwapChain,
    texture::Texture, utility::InstanceDevices,
};
use ash::vk;
use cgmath::{Point3, Vector2, Vector3, Zero};
use std::ops::Mul;

pub(crate) const WHITE: Vector3<f32> = Vector3::new(1., 1., 1.);

pub trait Object {
    fn translate(&mut self) {}
    fn rotate(&mut self) {}
    fn scale(&mut self) {}

    fn vertices_and_indices() -> VerticesAndIndices;
    fn object_vertices_and_indices(&self) -> &VerticesAndIndices;

    fn builder(position: Point3<f32>, orientation: Orientation) -> Self;
    fn destroy() {}

    fn texture_buffer(self, texture_buffer: Vec<u8>) -> Self;
    fn indexed(self) -> Self;
    fn topology(self, topology: ModelTopology) -> Self;
    fn cull_mode(self, cull_mode: ModelCullMode) -> Self;

    fn is_indexed(&self) -> bool;
    fn object_topology(&self) -> &ModelTopology;
    fn object_cull_mode(&self) -> &ModelCullMode;
}

pub trait ObjectBuilder: Object {
    fn object_graphics_pipeline(&self) -> &GraphicsPipeline;
    fn object_buffers(&self) -> &ModelBuffers;
    fn object_texture(&self) -> &Texture;

    fn graphics_pipeline(
        &mut self,
        swap_chain: &SwapChain,
        render_pass: ash::vk::RenderPass,
        instance_devices: &InstanceDevices,
    );

    fn build(
        self,
        command_pool: vk::CommandPool,
        command_buffer_count: u32,
        swap_chain: &SwapChain,
        render_pass: vk::RenderPass,
        instance_devices: &InstanceDevices,
    ) -> Self;

    /// # Safety
    ///
    /// Expand on safety of this function
    unsafe fn bind_index_and_vertex_buffers(
        &self,
        devices: &Devices,
        command_buffer: vk::CommandBuffer,
        offsets: &[vk::DeviceSize],
        index: usize,
    ) {
        devices.logical.device.cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.object_graphics_pipeline().features.pipeline,
        );

        devices.logical.device.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.object_graphics_pipeline().features.layout,
            0,
            std::slice::from_ref(
                &self
                    .object_graphics_pipeline()
                    .descriptor_set
                    .descriptor_sets[index],
            ),
            &[],
        );

        let vertex_buffers = [self.object_buffers().vertex.buffer];

        devices
            .logical
            .device
            .cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, offsets);

        devices.logical.device.cmd_draw(
            command_buffer,
            self.object_vertices_and_indices().vertices.len() as u32,
            1,
            0,
            0,
        );

        if self.is_indexed() {
            devices.logical.device.cmd_bind_index_buffer(
                command_buffer,
                self.object_buffers().index.buffer,
                0,
                vk::IndexType::UINT16,
            );

            devices.logical.device.cmd_draw_indexed(
                command_buffer,
                self.object_vertices_and_indices().indices.len() as u32,
                1,
                0,
                0,
                0,
            );
        }
    }
}

#[derive(Clone, new)]
pub struct VerticesAndIndices {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
}

#[derive(Clone, Copy, Debug, new)]
pub struct Vertex {
    pub pos: Vector3<f32>,
    pub colour: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub tex_coord: Vector2<f32>,
}

#[derive(new)]
pub struct Buffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
}

#[derive(new)]
pub struct ModelBuffers {
    pub vertex: Buffer,
    pub index: Buffer,
}

// #[derive(Clone)]
// pub struct ModelProperties {
//     pub texture: Vec<u8>,
//     pub indexed: bool,
//     pub topology: ModelTopology,
//     pub cull_mode: ModelCullMode,
//     pub vertices_and_indices: VerticesAndIndices,
// }

// pub(crate) struct Model {
//     pub texture: Texture,
//     pub graphics_pipeline: GraphicsPipeline,
//     pub buffers: ModelBuffers,
//     pub properties: ModelProperties,
// }

// impl Model {
//     pub fn new(
//         command_pool: vk::CommandPool,
//         command_buffer_count: u32,
//         swap_chain: &SwapChain,
//         render_pass: vk::RenderPass,
//         property: ModelProperties,
//         instance_devices: &InstanceDevices,
//     ) -> Self {
//         let VerticesAndIndices { vertices, indices } = property.vertices_and_indices.clone();

//         let texture = texture::Texture::new(&property.texture, command_pool, instance_devices);

//         let vertex_buffer = utility::create_vertex_index_buffer(
//             (size_of::<Vertex>() * vertices.len()).try_into().unwrap(),
//             &vertices,
//             vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
//             command_pool,
//             command_buffer_count,
//             instance_devices,
//         );

//         let index_buffer = utility::create_vertex_index_buffer(
//             (size_of::<u16>() * indices.len()).try_into().unwrap(),
//             &indices,
//             vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
//             command_pool,
//             command_buffer_count,
//             instance_devices,
//         );

//         let graphics_pipeline = GraphicsPipeline::new(
//             swap_chain,
//             render_pass,
//             texture.image_view,
//             texture.sampler,
//             property.clone(),
//             instance_devices,
//         );

//         Self {
//             texture,
//             graphics_pipeline,
//             buffers: ModelBuffers::new(vertex_buffer, index_buffer),
//             properties: property,
//         }
//     }

//     /// # Safety
//     ///
//     /// Expand on safety of this function
//     pub unsafe fn bind_index_and_vertex_buffers(
//         &self,
//         devices: &Devices,
//         command_buffer: vk::CommandBuffer,
//         offsets: &[vk::DeviceSize],
//         index: usize,
//     ) {
//         devices.logical.device.cmd_bind_pipeline(
//             command_buffer,
//             vk::PipelineBindPoint::GRAPHICS,
//             self.graphics_pipeline.features.pipeline,
//         );

//         devices.logical.device.cmd_bind_descriptor_sets(
//             command_buffer,
//             vk::PipelineBindPoint::GRAPHICS,
//             self.graphics_pipeline.features.layout,
//             0,
//             std::slice::from_ref(&self.graphics_pipeline.descriptor_set.descriptor_sets[index]),
//             &[],
//         );

//         let vertex_buffers = [self.buffers.vertex.buffer];

//         devices
//             .logical
//             .device
//             .cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, offsets);

//         devices.logical.device.cmd_draw(
//             command_buffer,
//             self.properties.vertices_and_indices.vertices.len() as u32,
//             1,
//             0,
//             0,
//         );

//         if self.properties.indexed {
//             devices.logical.device.cmd_bind_index_buffer(
//                 command_buffer,
//                 self.buffers.index.buffer,
//                 0,
//                 vk::IndexType::UINT16,
//             );

//             devices.logical.device.cmd_draw_indexed(
//                 command_buffer,
//                 self.properties.vertices_and_indices.indices.len() as u32,
//                 1,
//                 0,
//                 0,
//                 0,
//             );
//         }
//     }
// }

pub fn ring(inner_radius: f32, outer_radius: f32, sector_count: u32) -> VerticesAndIndices {
    assert!(
        inner_radius <= outer_radius,
        "Ring inner radius mut be smaller or equal to its outer radius"
    );

    let mut angle = 0.;
    let angle_step = 180. / sector_count as f32;
    let length = 1.;

    let mut vertices = Vec::new();

    for _ in 0..=sector_count {
        vertices.push(utility::make_point(
            &mut angle,
            outer_radius,
            angle_step,
            length,
            Vector2::zero(),
        ));
        vertices.push(utility::make_point(
            &mut angle,
            inner_radius,
            angle_step,
            length,
            Vector2::new(1., 1.),
        ));
    }

    VerticesAndIndices::new(vertices, utility::spherical_indices(sector_count, 2))
}

pub fn sphere(radius: f32, sector_count: u32, stack_count: u32) -> VerticesAndIndices {
    let length = 1. / radius;

    let sector_step = 2. * std::f32::consts::PI / sector_count as f32;
    let stack_step = std::f32::consts::PI / stack_count as f32;

    let mut pos = Vector3::zero();

    let mut vertices = Vec::new();

    for i in 0..=stack_count {
        let stack_angle = std::f32::consts::FRAC_PI_2 - i as f32 * stack_step;
        let xy = radius * stack_angle.cos();
        pos[2] = radius * stack_angle.sin();

        for j in 0..=sector_count {
            let sector_angle = j as f32 * sector_step;

            pos[0] = xy * sector_angle.cos();
            pos[1] = xy * sector_angle.sin();

            let normal = pos.mul(length);

            let tex_coord = Vector2::new(
                j as f32 / sector_count as f32,
                i as f32 / stack_count as f32,
            );

            vertices.push(Vertex::new(pos, WHITE, normal, tex_coord));
        }
    }

    VerticesAndIndices::new(
        vertices,
        utility::spherical_indices(sector_count, stack_count),
    )
}
