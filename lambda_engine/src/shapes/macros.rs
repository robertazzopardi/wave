#[macro_export]
macro_rules! vector2 {
    ($a:expr, $b:expr) => {
        nalgebra::Vector2::new($a, $b)
    };
}

#[macro_export]
macro_rules! pos3d {
    ($a1:expr, $a2:expr, $a3:expr) => {
        nalgebra::Point3::new($a1, $a2, $a3)
    };
}

#[macro_export]
macro_rules! vertex {
    ($pos:expr, $col:expr, $norm:expr, $tex:expr) => {
        crate::shapes::Vertex {
            pos: $pos,
            colour: $col,
            normal: $norm,
            tex_coord: $tex,
        }
    };
}
