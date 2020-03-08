use crate::{
    crystal::{rads::Scene, BlockMap, PlanesSep},
    math::prelude::*,
};
use amethyst::{
    core::{
        ecs::{
            Component, DenseVecStorage, Join, ReadExpect, ReadStorage, System, SystemData,
            WriteExpect,
        },
        math::Point3,
        transform::Transform,
    },
    renderer::light::Light,
};
use amethyst_derive::SystemDesc;

pub struct PointLight {
    pos: Point3<f32>,
    color: Color,
}

impl Default for PointLight {
    fn default() -> PointLight {
        PointLight {
            pos: Point3::new(30.0, 30.0, 30.0),
            color: Color::new(1.0, 0.8, 0.8),
        }
    }
}

impl Component for PointLight {
    type Storage = DenseVecStorage<Self>;
}

#[derive(SystemDesc)]
#[system_desc(name(ApplyLightsSystemSystemDesc))]
pub struct ApplyLightsSystem;
impl<'a> System<'a> for ApplyLightsSystem {
    type SystemData = (
        WriteExpect<'a, Scene>,
        ReadExpect<'a, PlanesSep>,
        ReadExpect<'a, BlockMap>,
        ReadStorage<'a, PointLight>,
    );

    fn run(&mut self, (mut rad_scene, planes, blockmap, point_lights): Self::SystemData) {
        rad_scene.clear_emit();
        for light in point_lights.join() {
            rad_scene.apply_light(&planes, &blockmap, &light.pos, &light.color);
        }
    }
}

#[derive(SystemDesc)]
#[system_desc(name(ApplyRendyLightsSystemSystemDesc))]
pub struct ApplyRendyLightsSystem;
impl<'a> System<'a> for ApplyRendyLightsSystem {
    type SystemData = (
        WriteExpect<'a, Scene>,
        ReadExpect<'a, PlanesSep>,
        ReadExpect<'a, BlockMap>,
        ReadStorage<'a, Light>,
        ReadStorage<'a, Transform>,
    );

    fn run(&mut self, (mut rad_scene, planes, blockmap, light, transform): Self::SystemData) {
        rad_scene.clear_emit();
        for (_light, transform) in (&light, &transform).join() {
            // FIXME: this is broken, and much too complicated for just getting the light's global translation...
            let pos = transform.global_view_matrix().try_inverse().unwrap()
                * Vec4::new(0.0, 0.0, 0.0, 1.0);
            println!("transform: {:?}", transform);
            let pos = Point3::from_homogeneous(pos).unwrap();
            println!("pos: {:?}", pos);

            rad_scene.apply_light(&planes, &blockmap, &pos, &Color::new(1.0, 0.8, 0.8));
        }
    }
}
