pub mod utilities;

use crate::{
    device::Devices,
    pipeline::GraphicsPipeline,
    swapchain::SwapChain,
    texture::{self, Texture},
};
use ash::{vk, Instance};
use cgmath::{Vector2, Vector3, Zero};
use std::{mem::size_of, ops::Mul};
use self::utilities::{ModelTopology, ModelCullMode};

pub(crate) const WHITE: Vector3<f32> = Vector3::new(1., 1., 1.);

// #[derive(Debug, Clone, Copy)]
// pub struct Generic<const S: usize, const B: usize> {
//     vertices: [Vertex; S],
//     indices: [u16; B],
// }

// impl<const S: usize, const B: usize> Generic<S, B> {
//     pub fn new(vertices: &[Vertex; S], indices: &[u16; B]) -> Self {
//         Self {
//             vertices: *vertices,
//             indices: *indices,
//         }
//     }
// }

#[derive(Clone)]
pub struct VerticesAndIndices {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
}

impl VerticesAndIndices {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u16>) -> Self {
        Self { vertices, indices }
    }
}

#[derive(Clone)]
pub struct ModelProperties {
    pub texture: Vec<u8>,
    pub indexed: bool,
    pub topology: ModelTopology,
    pub cull_mode: ModelCullMode,
    pub vertices_and_indices: VerticesAndIndices,
}

#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub pos: Vector3<f32>,
    pub colour: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub tex_coord: Vector2<f32>,
}

impl Vertex {
    pub fn new(
        pos: Vector3<f32>,
        colour: Vector3<f32>,
        normal: Vector3<f32>,
        tex_coord: Vector2<f32>,
    ) -> Self {
        Self {
            pos,
            colour,
            normal,
            tex_coord,
        }
    }
}

pub struct Model {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub texture: Texture,
    pub graphics_pipeline: GraphicsPipeline,
    pub vertex_buffer: vk::Buffer,
    pub vertex_buffer_memory: vk::DeviceMemory,
    pub index_buffer: vk::Buffer,
    pub index_buffer_memory: vk::DeviceMemory,
    pub properties: ModelProperties,
}

impl Model {
    pub fn new(
        instance: &Instance,
        devices: &Devices,
        command_pool: vk::CommandPool,
        command_buffer_count: u32,
        swapchain: &SwapChain,
        render_pass: vk::RenderPass,
        property: ModelProperties,
    ) -> Self {
        let VerticesAndIndices { vertices, indices } = property.vertices_and_indices.clone();

        let texture = texture::Texture::new(
            instance,
            devices,
            &property.texture,
            command_pool,
            command_buffer_count,
        );

        let (vertex_buffer, vertex_buffer_memory) = utilities::create_vertex_index_buffer(
            instance,
            devices,
            (size_of::<Vertex>() * vertices.len()).try_into().unwrap(),
            &vertices,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            command_pool,
            command_buffer_count,
        );

        let (index_buffer, index_buffer_memory) = utilities::create_vertex_index_buffer(
            instance,
            devices,
            (size_of::<u16>() * indices.len()).try_into().unwrap(),
            &indices,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            command_pool,
            command_buffer_count,
        );

        let graphics_pipeline = GraphicsPipeline::new(
            instance,
            devices,
            swapchain,
            render_pass,
            texture.image_view,
            texture.sampler,
            property.clone(),
        );

        Self {
            vertices,
            indices,
            texture,
            graphics_pipeline,
            vertex_buffer,
            vertex_buffer_memory,
            index_buffer,
            index_buffer_memory,
            properties: property,
        }
    }

    /// # Safety
    ///
    /// Expand on safety of this function
    pub unsafe fn bind_index_and_vertex_buffers(
        &self,
        devices: &Devices,
        command_buffer: vk::CommandBuffer,
        offsets: &[vk::DeviceSize],
        index: usize,
    ) {
        devices.logical.cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.graphics_pipeline.pipeline,
        );

        devices.logical.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.graphics_pipeline.layout,
            0,
            std::slice::from_ref(&self.graphics_pipeline.descriptor_set.descriptor_sets[index]),
            &[],
        );

        let vertex_buffers = [self.vertex_buffer];

        devices
            .logical
            .cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, offsets);

        devices
            .logical
            .cmd_draw(command_buffer, self.vertices.len() as u32, 1, 0, 0);

        if self.properties.indexed {
            devices.logical.cmd_bind_index_buffer(
                command_buffer,
                self.index_buffer,
                0,
                vk::IndexType::UINT16,
            );

            devices
                .logical
                .cmd_draw_indexed(command_buffer, self.indices.len() as u32, 1, 0, 0, 0);
        }
    }
}

pub fn ring(inner_radius: f32, outer_radius: f32, sector_count: u32) -> VerticesAndIndices {
    assert!(
        inner_radius <= outer_radius,
        "Ring inner radius mut be smaller or equal to its outer radius"
    );

    let stack_count = 2;

    let mut angle = 0.;
    let angle_step = 180. / sector_count as f32;
    let length = 1.;

    let mut vertices = Vec::new();

    for _ in 0..=sector_count {
        vertices.push(utilities::make_point(
            &mut angle,
            outer_radius,
            angle_step,
            length,
            Vector2::zero(),
        ));
        vertices.push(utilities::make_point(
            &mut angle,
            inner_radius,
            angle_step,
            length,
            Vector2::new(1., 1.),
        ));
    }

    VerticesAndIndices::new(
        vertices,
        utilities::calculate_sphere_indices(sector_count, stack_count),
    )
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
        utilities::calculate_sphere_indices(sector_count, stack_count),
    )
}

pub fn cube() -> VerticesAndIndices {
    let cube = utilities::CUBE_VERTICES;

    cube.map(|_| utilities::calculate_normals);

    VerticesAndIndices::new(
        cube.into_iter().flatten().collect(),
        utilities::CUBE_INDICES.to_vec(),
    )
}
