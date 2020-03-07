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
        AnimationSet, AnimationSetPrefab, DeferStartRelation, EndControl, InterpolationFunction,
        Sampler, SamplerPrimitive, TransformChannel,
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

use serde::{Deserialize, Serialize};
type MyPrefabData = (
    Option<BasicScenePrefab<(Vec<Position>, Vec<Normal>, Vec<TexCoord>)>>,
    Option<AnimationSetPrefab<AnimationId, Transform>>,
);

#[derive(Eq, PartialOrd, PartialEq, Hash, Debug, Copy, Clone, Deserialize, Serialize)]
enum AnimationId {
    Scale,
    Rotate,
    Translate,
    Test,
}

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

struct ExampleState;

impl SimpleState for ExampleState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let prefab_handle = data.world.exec(|loader: PrefabLoader<'_, MyPrefabData>| {
            loader.load("prefab/crystal_planes.ron", RonFormat, ())
        });
        let world = data.world;

        let scene = world
            .create_entity()
            .named("Crystal Planes Scene")
            .with(prefab_handle)
            .build();

        {
            let (animation_set, animation) = {
                let loader = world.read_resource::<Loader>();

                let sampler = loader.load_from_data(
                    Sampler {
                        input: vec![0., 1.],
                        output: vec![
                            SamplerPrimitive::Vec3([0., 0., 0.]),
                            SamplerPrimitive::Vec3([0., 1., 0.]),
                        ],
                        function: InterpolationFunction::Step,
                    },
                    (),
                    &world.read_resource(),
                );

                let animation = loader.load_from_data(
                    Animation::new_single(0, TransformChannel::Translation, sampler),
                    (),
                    &world.read_resource(),
                );
                let mut animation_set: AnimationSet<AnimationId, Transform> = AnimationSet::new();
                animation_set.insert(AnimationId::Test, animation.clone());
                (animation_set, animation)
            };

            let entity = world.create_entity().with(animation_set).build();
            let mut storage = world.write_storage::<AnimationControlSet<AnimationId, Transform>>();
            let control_set = get_animation_set(&mut storage, entity).unwrap();
            control_set.add_animation(
                AnimationId::Test,
                &animation,
                EndControl::Loop(None),
                1.0,
                AnimationCommand::Start,
            );
        }
        add_animation(world, scene, AnimationId::Translate, 1.0, None, false);
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
            //  else if is_key_down(&event, VirtualKeyCode::P) {
            //     self.display_loaded_entities(world);
            // }
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
        .with_bundle(AnimationBundle::<AnimationId, Transform>::new(
            "animation_control_system",
            "sampler_interpolation_system",
        ))?
        .with_bundle(
            TransformBundle::new().with_dep(&["fly_movement", "sampler_interpolation_system"]),
        )?
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

fn add_animation(
    world: &mut World,
    entity: Entity,
    id: AnimationId,
    rate: f32,
    defer: Option<(AnimationId, DeferStartRelation)>,
    toggle_if_exists: bool,
) {
    let animation = world
        .read_storage::<AnimationSet<AnimationId, Transform>>()
        .get(entity)
        .and_then(|s| s.get(&id))
        .cloned()
        .unwrap();
    let mut sets = world.write_storage();
    let control_set = get_animation_set::<AnimationId, Transform>(&mut sets, entity).unwrap();
    match defer {
        None => {
            if toggle_if_exists && control_set.has_animation(id) {
                control_set.toggle(id);
            } else {
                control_set.add_animation(
                    id,
                    &animation,
                    EndControl::Normal,
                    rate,
                    AnimationCommand::Start,
                );
            }
        }

        Some((defer_id, defer_relation)) => {
            control_set.add_deferred_animation(
                id,
                &animation,
                EndControl::Normal,
                rate,
                AnimationCommand::Start,
                defer_id,
                defer_relation,
            );
        }
    }
}
