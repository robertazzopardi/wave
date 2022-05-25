use lambda_engine::prelude::*;

const CUBE_MODEL: &str = "./examples/assets/models/monkey_model/monkey_head.obj";
const SATURN_TEXTURE: &str = "./examples/assets/textures/2k_saturn.jpg";

fn main() {
    let mut display = Display::new(Resolution::ResHD);

    let mut camera = Camera::new(2., 0., 0.);

    let monkey_model = Model::new(
        GeometryBuilder::default()
            .properties(
                ModelInfoBuilder::default()
                    .radius(0.3)
                    .model_path(CUBE_MODEL.to_owned())
                    .build()
                    .unwrap(),
            )
            .texture(SATURN_TEXTURE)
            .shader(ShaderType::LightTexture)
            .cull_mode(ModelCullMode::Back)
            .indexed()
            .build()
            .unwrap(),
    );

    let objects: Geometries = vec![monkey_model.into()];

    let mut engine = Engine::new(&display, &mut camera, objects, None);

    engine.run(&mut display, camera)
}
