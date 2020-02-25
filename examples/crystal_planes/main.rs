//! Demonstrates how to use the fly camera
mod custom_pass;
mod quad_pass;
use crate::quad_pass::{RenderQuad, Triangle};

use amethyst::{
    assets::{PrefabLoader, PrefabLoaderSystemDesc, RonFormat},
    controls::{FlyControlBundle, HideCursor},
    core::transform::TransformBundle,
    ecs::WorldExt,
    input::{
        is_key_down, is_mouse_button_down, InputBundle, InputEvent, ScrollDirection, StringBindings,
    },
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

struct ExampleState;

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
        if let StateEvent::Window(event) = &event {
            if is_key_down(&event, VirtualKeyCode::Escape) {
                let mut hide_cursor = world.write_resource::<HideCursor>();
                hide_cursor.hide = false;
            } else if is_mouse_button_down(&event, MouseButton::Left) {
                let mut hide_cursor = world.write_resource::<HideCursor>();
                hide_cursor.hide = true;
            }
        }
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
        .with_bundle(
            FlyControlBundle::<StringBindings>::new(
                Some(String::from("move_x")),
                Some(String::from("move_y")),
                Some(String::from("move_z")),
            )
            .with_sensitivity(0.1, 0.1),
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

    let mut game = Application::build(assets_dir, ExampleState)?.build(game_data)?;
    game.run();
    Ok(())
}
