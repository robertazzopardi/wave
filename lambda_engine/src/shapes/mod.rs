pub mod l2d;
pub mod l3d;
pub mod macros;
pub mod utility;

use self::{
    l2d::{ring::RingInfo, square::SquareInfo},
    l3d::{cube::CubeInfo, sphere::SphereInfo},
    utility::{ModelCullMode, ModelTopology},
};
use crate::{
    pipeline::GraphicsPipeline,
    swap_chain::SwapChain,
    texture::{self, Texture},
    utility::InstanceDevices,
};
use ash::vk;
use derive_builder::Builder;
use derive_more::{Deref, DerefMut, From};
use enum_as_inner::EnumAsInner;
use nalgebra::{Point3, Vector2, Vector3};
use std::{fs::File, io::Read, mem::size_of};

pub const WHITE: Vector3<f32> = Vector3::new(1., 1., 1.);
pub const VEC3_ZERO: Vector3<f32> = Vector3::new(0., 0., 0.);

#[derive(Debug, EnumAsInner)]
pub enum ShapeProperties {
    Cube(CubeInfo),
    Sphere(SphereInfo),
    Ring(RingInfo),
    Square(SquareInfo),
}

#[derive(Default, Builder, Debug, Clone)]
#[builder(default)]
pub struct Shape<T: Default> {
    pub properties: T,

    #[builder(setter(custom))]
    pub texture: Vec<u8>,
    pub indexed: bool,
    pub topology: ModelTopology,
    pub cull_mode: ModelCullMode,

    pub(crate) vertices_and_indices: VerticesAndIndices,
    pub(crate) texture_buffer: Option<Texture>,
    pub(crate) graphics_pipeline: Option<GraphicsPipeline>,
    pub(crate) buffers: Option<ModelBuffers>,
}

impl<'a, T: Default> ShapeBuilder<T> {
    pub fn texture(&mut self, path: &'a str) -> &mut Self {
        let file = File::open(path);

        if let Ok(mut texture_file) = file {
            let mut data = Vec::new();
            texture_file.read_to_end(&mut data).unwrap();
            self.texture = Some(data);
        }

        self
    }
}

impl<T: Default> private::Object for Shape<T>
where
    Shape<T>: Object,
{
    fn buffers(&mut self, model_buffers: ModelBuffers) {
        self.buffers = Some(model_buffers);
    }

    fn texture(&mut self, command_pool: vk::CommandPool, instance_devices: &InstanceDevices) {
        if !self.texture.is_empty() {
            self.texture_buffer = Some(texture::Texture::new(
                &self.texture,
                command_pool,
                instance_devices,
            ));
        }
    }

    fn object_graphics_pipeline(&self) -> &GraphicsPipeline {
        self.graphics_pipeline.as_ref().unwrap()
    }

    fn object_buffers(&self) -> &ModelBuffers {
        self.buffers.as_ref().unwrap()
    }

    fn object_texture(&self) -> &Texture {
        self.texture_buffer.as_ref().unwrap()
    }

    fn object_vertices_and_indices(&self) -> &VerticesAndIndices {
        &self.vertices_and_indices
    }

    fn is_indexed(&self) -> bool {
        self.indexed
    }

    fn graphics_pipeline(
        &mut self,
        swap_chain: &crate::swap_chain::SwapChain,
        render_pass: ash::vk::RenderPass,
        instance_devices: &crate::utility::InstanceDevices,
    ) {
        self.graphics_pipeline = Some(GraphicsPipeline::new(
            swap_chain,
            render_pass,
            &self.texture_buffer.unwrap(),
            self.topology,
            self.cull_mode,
            instance_devices,
        ));
    }
}

pub trait Object: private::Object {
    fn vertices_and_indices(&mut self);

    fn construct(
        &mut self,
        command_pool: vk::CommandPool,
        command_buffer_count: u32,
        swap_chain: &SwapChain,
        render_pass: vk::RenderPass,
        instance_devices: &InstanceDevices,
    ) {
        self.texture(command_pool, instance_devices);

        self.vertices_and_indices();

        let model_buffers = self.object_vertices_and_indices().create_buffers(
            command_pool,
            command_buffer_count,
            instance_devices,
        );

        self.buffers(model_buffers);

        self.graphics_pipeline(swap_chain, render_pass, instance_devices);
    }
}

pub(crate) mod private {
    use super::{ModelBuffers, VerticesAndIndices};
    use crate::{
        device::{Devices, LogicalDeviceFeatures},
        memory,
        pipeline::GraphicsPipeline,
        swap_chain::SwapChain,
        texture::Texture,
        uniform_buffer::UniformBufferObject,
        utility::InstanceDevices,
    };
    use ash::vk;

    pub trait Object {
        fn buffers(&mut self, model_buffers: ModelBuffers);
        fn texture(&mut self, command_pool: vk::CommandPool, instance_devices: &InstanceDevices);

        fn object_graphics_pipeline(&self) -> &GraphicsPipeline {
            unimplemented!()
        }
        fn object_buffers(&self) -> &ModelBuffers {
            unimplemented!()
        }
        fn object_texture(&self) -> &Texture {
            unimplemented!()
        }
        fn object_vertices_and_indices(&self) -> &VerticesAndIndices {
            unimplemented!()
        }

        fn is_indexed(&self) -> bool {
            unimplemented!()
        }
        fn graphics_pipeline(
            &mut self,
            _: &SwapChain,
            _: ash::vk::RenderPass,
            _: &InstanceDevices,
        ) {
            unimplemented!()
        }

        /// # Safety
        ///
        ///
        unsafe fn destroy(&self, logical: &LogicalDeviceFeatures) {
            let object_texture = self.object_texture();

            logical.device.destroy_sampler(object_texture.sampler, None);

            logical
                .device
                .destroy_image_view(object_texture.image_view, None);

            logical
                .device
                .destroy_image(object_texture.image.image, None);

            logical
                .device
                .free_memory(object_texture.image.memory, None);

            logical.device.destroy_descriptor_set_layout(
                self.object_graphics_pipeline()
                    .descriptor_set
                    .descriptor_set_layout,
                None,
            );

            let object_buffers = self.object_buffers();

            logical
                .device
                .destroy_buffer(object_buffers.vertex.buffer, None);

            logical
                .device
                .free_memory(object_buffers.vertex.memory, None);

            logical
                .device
                .destroy_buffer(object_buffers.index.buffer, None);

            logical
                .device
                .free_memory(object_buffers.index.memory, None);
        }

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
            let object_graphics_pipeline = self.object_graphics_pipeline();

            devices.logical.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                object_graphics_pipeline.features.pipeline,
            );

            devices.logical.device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                object_graphics_pipeline.features.layout,
                0,
                std::slice::from_ref(
                    &object_graphics_pipeline.descriptor_set.descriptor_sets[index],
                ),
                &[],
            );

            let object_buffers = self.object_buffers();

            let vertex_buffers = [object_buffers.vertex.buffer];

            devices.logical.device.cmd_bind_vertex_buffers(
                command_buffer,
                0,
                &vertex_buffers,
                offsets,
            );

            let object_and_vertices_and_indices = self.object_vertices_and_indices();

            devices.logical.device.cmd_draw(
                command_buffer,
                object_and_vertices_and_indices.vertices.len() as u32,
                1,
                0,
                0,
            );

            if self.is_indexed() {
                devices.logical.device.cmd_bind_index_buffer(
                    command_buffer,
                    object_buffers.index.buffer,
                    0,
                    vk::IndexType::UINT16,
                );

                devices.logical.device.cmd_draw_indexed(
                    command_buffer,
                    object_and_vertices_and_indices.indices.len() as u32,
                    1,
                    0,
                    0,
                    0,
                );
            }
        }

        fn map_memory(
            &self,
            logical: &LogicalDeviceFeatures,
            current_image: usize,
            buffer_size: u64,
            ubos: &[UniformBufferObject; 1],
        ) {
            memory::map_memory(
                &logical.device,
                self.object_graphics_pipeline()
                    .descriptor_set
                    .uniform_buffers[current_image]
                    .memory,
                buffer_size,
                ubos,
            );
        }

        /// # Safety
        ///
        ///
        unsafe fn recreate_drop(&self, logical: &LogicalDeviceFeatures, swap_chain: &SwapChain) {
            let object_graphics_pipeline = self.object_graphics_pipeline();

            logical
                .device
                .destroy_pipeline(object_graphics_pipeline.features.pipeline, None);
            logical
                .device
                .destroy_pipeline_layout(object_graphics_pipeline.features.layout, None);

            logical.device.destroy_descriptor_pool(
                object_graphics_pipeline.descriptor_set.descriptor_pool,
                None,
            );

            for i in 0..swap_chain.images.len() {
                logical.device.destroy_buffer(
                    object_graphics_pipeline.descriptor_set.uniform_buffers[i].buffer,
                    None,
                );
                logical.device.free_memory(
                    object_graphics_pipeline.descriptor_set.uniform_buffers[i].memory,
                    None,
                );
            }
        }
    }
}

#[derive(new, Clone, Default, Debug, From, Deref, DerefMut)]
pub struct Vertices(Vec<Vertex>);

#[derive(new, Clone, Default, Debug, From, Deref, DerefMut)]
pub struct Indices(Vec<u16>);

#[derive(new, Clone, Default, Debug)]
pub struct VerticesAndIndices {
    vertices: Vertices,
    indices: Indices,
}

impl VerticesAndIndices {
    pub fn create_buffers(
        &self,
        command_pool: ash::vk::CommandPool,
        command_buffer_count: u32,
        instance_devices: &crate::utility::InstanceDevices,
    ) -> ModelBuffers {
        let vertex = utility::create_vertex_index_buffer(
            (size_of::<Vertex>() * self.vertices.len())
                .try_into()
                .unwrap(),
            &self.vertices,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            command_pool,
            command_buffer_count,
            instance_devices,
        );

        let index = utility::create_vertex_index_buffer(
            (size_of::<u16>() * self.indices.len()).try_into().unwrap(),
            &self.indices,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            command_pool,
            command_buffer_count,
            instance_devices,
        );

        ModelBuffers::new(vertex, index)
    }
}

#[derive(Clone, Copy, Debug, new)]
pub struct Vertex {
    pub pos: Point3<f32>,
    pub colour: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub tex_coord: Vector2<f32>,
}

#[derive(new, Clone, Copy, Default, Debug)]
pub struct Buffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
}

#[derive(new, Clone, Copy, Default, Debug)]
pub struct ModelBuffers {
    pub vertex: Buffer,
    pub index: Buffer,
}
