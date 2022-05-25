use crate::{vector2, GeomBehavior, Geometry, VerticesAndIndices, WHITE};
use derive_builder::Builder;
use derive_more::{Deref, DerefMut};
use lambda_space::{
    space::{Orientation, Vertex, Vertices},
    vertex,
};
use lambda_vulkan::{
    buffer::ModelBuffers, command_buffer::CommandPool, graphics_pipeline::GraphicsPipeline,
    swap_chain::SwapChain, texture::Texture, utility::InstanceDevices, RenderPass, VulkanObject,
};
use nalgebra::{Point3, Vector3};

const SQUARE_INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];

#[derive(Builder, Default, Debug, Clone, new)]
#[builder(default)]
pub struct SquareInfo {
    pub position: Point3<f32>,
    pub orientation: Orientation,
    pub radius: f32,
    pub has_depth: bool,
}

#[derive(new, Deref, DerefMut)]
pub struct Square(Geometry<SquareInfo>);

impl GeomBehavior for Square {
    fn vertices_and_indices(&mut self) -> VerticesAndIndices {
        let mut vertices = square_from_vertices(&[
            [-0.5, -0.5, 0.5],
            [0.5, -0.5, 0.5],
            [0.5, 0.5, 0.5],
            [-0.5, 0.5, 0.5],
        ]);

        vertices.iter_mut().for_each(|vert| {
            vert.pos += self.properties.position.coords;
        });

        self.vulkan_object.vertices_and_indices = Some(VerticesAndIndices::new(
            vertices,
            SQUARE_INDICES.to_vec().into(),
        ));
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

pub fn square_from_vertices(vertices: &[[f32; 3]]) -> Vertices {
    let tex_coord = vec![
        vector2!(1., 0.),
        vector2!(0., 0.),
        vector2!(0., 1.),
        vector2!(1., 1.),
    ];

    let mut tex_coords = Vec::new();
    for _ in 0..(vertices.len() / 4) {
        tex_coords.extend(tex_coord.clone());
    }

    Vertices::new(
        vertices
            .iter()
            .enumerate()
            .map(|(index, vert)| {
                vertex!(
                    Point3::new(vert[0], vert[1], vert[2]),
                    WHITE,
                    Vector3::zeros(),
                    tex_coords[index]
                )
            })
            .collect::<Vec<Vertex>>(),
    )
}
