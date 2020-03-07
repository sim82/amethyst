//! Demonstrates how to use the fly camera
#[macro_use]
extern crate itertools;

mod crystal;
mod custom_pass;
mod light;
mod math;
mod quad;
mod quad_pass;
mod systems;
mod vertex;
use crate::quad_pass::RenderQuad;
use amethyst::{
    animation::{
        get_animation_set, Animation, AnimationBundle, AnimationCommand, AnimationControlSet,
        AnimationSet, AnimationSetPrefab, EndControl, InterpolationFunction, Sampler,
        SamplerPrimitive, TransformChannel,
    },
    assets::{Loader, PrefabLoader, PrefabLoaderSystemDesc, RonFormat},
    controls::{FlyControlBundle, HideCursor},
    core::{
        math::{Vector3, Vector4},
        transform::TransformBundle,
        Transform,
    },
    ecs::{prelude::*, WorldExt, WriteExpect},
    input::{is_key_down, is_mouse_button_down, InputBundle, StringBindings},
    prelude::*,
    renderer::{
        palette::Srgb,
        plugins::{RenderShaded3D, RenderSkybox, RenderToWindow},
        rendy::mesh::{Normal, Position, TexCoord},
        types::DefaultBackend,
        RenderingBundle,
    },
    utils::{application_root_dir, scene::BasicScenePrefab},
    winit::{MouseButton, VirtualKeyCode},
    Error,
};

type MyPrefabData = BasicScenePrefab<(Vec<Position>, Vec<Normal>, Vec<TexCoord>)>;

#[derive(PartialEq, Debug)]
enum LightMode {
    RandomFlashing,
    Tron,
    LightSources,
}

impl Default for LightMode {
    fn default() -> Self {
        LightMode::RandomFlashing
    }
}

struct ExampleState;

struct MapLoadState;
impl SimpleState for MapLoadState {
    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        let world = &mut data.world;

        let bm = crystal::read_map("hidden_ramp.txt").expect("could not read file");
        let mut planes = crystal::PlanesSep::new();
        planes.create_planes(&bm);
        world.insert(bm);
        // let planes_copy: Vec<crystal::Plane> = planes.planes_iter().cloned().collect();
        world.register::<crystal::Plane>();
        world.register::<quad::QuadInstance>();
        world.register::<light::PointLight>();
        world.insert(Some(quad::ColorGeneration(0)));
        world.insert(LightMode::RandomFlashing);

        world
            .create_entity()
            .named("the pointlight")
            .with(light::PointLight::default())
            .build();

        for (i, p) in planes.planes_iter().cloned().enumerate() {
            let point = &p.cell;
            let dir = match p.dir {
                crystal::Dir::ZxPos => 4,
                crystal::Dir::ZxNeg => 5,
                crystal::Dir::YzPos => 2,
                crystal::Dir::YzNeg => 3,
                crystal::Dir::XyPos => 0,
                crystal::Dir::XyNeg => 1,
            };
            let quad = quad::QuadInstance {
                translate: Vector3::new(
                    point[0] as f32 * 0.25,
                    point[1] as f32 * 0.25,
                    point[2] as f32 * 0.25,
                ),
                dir,
                color: Vector4::new(1.0, 1.0, 1.0, 1.0),
                index: i as u32,
            };
            world.create_entity().with(p).with(quad).build();
        }
        world.insert(planes);
        let rad_scene = crystal::rads::Scene::new(world);
        world.insert(rad_scene);
        println!("load done");
        Trans::Replace(Box::new(ExampleState))
    }
}

impl SimpleState for ExampleState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let prefab_handle = data.world.exec(|loader: PrefabLoader<'_, MyPrefabData>| {
            loader.load("prefab/fly_camera.ron", RonFormat, ())
        });
        let world = data.world;

        world
            .create_entity()
            .named("Fly Camera Scene")
            .with(prefab_handle)
            .build();

        // Add some triangles
        world
            .create_entity()
            .with(Triangle {
                points: [[0., 0.], [0., 1.], [1., 0.0]],
                colors: [[1., 0., 0., 1.], [0., 1., 0., 1.], [0., 0., 1., 1.]],
            })
            .build();
        world
            .create_entity()
            .with(Triangle {
                points: [[-2., -1.], [0., -1.], [-1., 1.0]],
                colors: [[1., 1., 0., 1.], [0., 1., 1., 1.], [1., 0., 1., 1.]],
            })
            .build();
        world
            .create_entity()
            .with(Triangle {
                points: [[0.2, -0.7], [0.4, -0.1], [0.8, -1.5]],
                colors: [[1., 0., 0., 1.], [0., 0., 0., 1.], [1., 1., 1., 1.]],
            })
            .build();

        world
            .create_entity()
            .with(Triangle {
                points: [[-0.2, 0.7], [-0.4, 0.1], [-0.8, 0.5]],
                colors: [
                    [0.337, 0.176, 0.835, 1.],
                    [0.337, 0.176, 0.835, 1.],
                    [0.337, 0.176, 0.835, 1.],
                ],
            })
            .build();
    }

    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        let StateData { world, .. } = data;
        let mut light_mode = WriteExpect::<LightMode>::fetch(world);

        if let StateEvent::Window(event) = &event {
            if is_key_down(&event, VirtualKeyCode::Escape) {
                let mut hide_cursor = world.write_resource::<HideCursor>();
                hide_cursor.hide = false;
            } else if is_mouse_button_down(&event, MouseButton::Left) {
                let mut hide_cursor = world.write_resource::<HideCursor>();
                hide_cursor.hide = true;
            } else if is_key_down(&event, VirtualKeyCode::Key1) {
                *light_mode = LightMode::RandomFlashing;
            } else if is_key_down(&event, VirtualKeyCode::Key2) {
                *light_mode = LightMode::Tron;
            } else if is_key_down(&event, VirtualKeyCode::Key3) {
                *light_mode = LightMode::LightSources;
            }
        }

        // println!("LightMode: {:?}", *light_mode);
        Trans::None
        // match &event {
        //     // Using the Mouse Wheel to control the scale
        //     StateEvent::Input(input) => {
        //         if let InputEvent::MouseWheelMoved(dir) = input {
        //             let mut scale = world.write_resource::<CustomUniformArgs>();
        //             match dir {
        //                 ScrollDirection::ScrollUp => (*scale).scale *= 1.1,
        //                 ScrollDirection::ScrollDown => (*scale).scale /= 1.1,
        //                 _ => {}
        //             }
        //         }
        //         Trans::None
        //     }
        //     _ => Trans::None,
        // }
    }
}

fn main() -> Result<(), Error> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let assets_dir = app_root.join("examples/assets");

    let display_config_path = app_root.join("examples/fly_camera/config/display.ron");

    let key_bindings_path = app_root.join("examples/fly_camera/config/input.ron");

    let game_data = GameDataBuilder::default()
        .with_system_desc(PrefabLoaderSystemDesc::<MyPrefabData>::default(), "", &[])
        // .with_system_desc(quad::DiscoSystemDesc::default(), "disco_system", &[])
        .with(
            // FIXME: create pausable system from SystemDesc?
            systems::RandomFlashingEmitSystem {}.pausable(LightMode::RandomFlashing),
            "random_flashing_emit_system",
            &[],
        )
        .with(
            systems::TronEmitSystem {}.pausable(LightMode::Tron),
            "tron_emit_system",
            &[],
        )
        .with(
            light::ApplyLightsSystem {}.pausable(LightMode::LightSources),
            "apply_lights_system",
            &[],
        )
        .with_system_desc(
            systems::RunRadSceneSystemDesc::default(),
            "run_rad_system",
            &[
                "random_flashing_emit_system",
                "tron_emit_system",
                "apply_lights_system",
            ],
        )
        .with_system_desc(
            systems::CopyRadFrontSystemDesc::default(),
            "copy_rad_front_system",
            &["run_rad_system"],
        )
        .with_bundle(
            FlyControlBundle::<StringBindings>::new(
                Some(String::from("move_x")),
                Some(String::from("move_y")),
                Some(String::from("move_z")),
            )
            .with_sensitivity(0.1, 0.1)
            .with_speed(10.0),
        )?
        .with_bundle(TransformBundle::new().with_dep(&["fly_movement"]))?
        .with_bundle(
            InputBundle::<StringBindings>::new().with_bindings_from_file(&key_bindings_path)?,
        )?
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?
                        .with_clear([0.34, 0.36, 0.52, 1.0]),
                )
                .with_plugin(RenderShaded3D::default())
                // Add our custom render plugin to the rendering bundle.
                .with_plugin(RenderQuad::default())
                .with_plugin(RenderSkybox::with_colors(
                    Srgb::new(0.82, 0.51, 0.50),
                    Srgb::new(0.18, 0.11, 0.85),
                )),
        )?;

    let mut game = Application::build(assets_dir, MapLoadState)?.build(game_data)?;
    game.run();
    Ok(())
}
