extern crate derive_builder;

pub mod l2d;
pub mod l3d;
pub mod macros;
pub mod utility;

use derive_more::Deref;
pub use enum_dispatch::enum_dispatch;
use lambda_space::space::{Vertex, VerticesAndIndices};
use lambda_vulkan::GeomProperties;
use nalgebra::Vector3;

pub mod prelude {
    pub use crate::{
        enum_dispatch,
        l2d::{
            plane::{Plane, PlaneBuilder},
            ring::{Ring, RingBuilder},
        },
        l3d::{
            cube::{Cube, CubeBuilder},
            model::{Model, ModelBuilder},
            sphere::{Sphere, SphereBuilder},
        },
        Behavior, GeomBuilder, Indexed, TextureBuffer,
    };
}

pub const WHITE: Vector3<f32> = Vector3::new(1., 1., 1.);
pub const VEC3_ZERO: Vector3<f32> = Vector3::new(0., 0., 0.);

#[derive(Clone, Copy, Debug, Deref)]
pub struct Indexed(pub bool);

impl Default for Indexed {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(Clone, Debug, Deref, Default)]
pub struct TextureBuffer(pub Vec<u8>);

#[enum_dispatch]
pub trait GeomBuilder {
    fn vertices_and_indices(&self) -> VerticesAndIndices;

    fn features(&self) -> GeomProperties;
}

#[enum_dispatch]
pub trait Behavior {
    fn actions(&mut self);
}
