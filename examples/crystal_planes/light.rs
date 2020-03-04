use crate::{
    crystal::{rads::Scene, BlockMap, PlanesSep},
    math::prelude::*,
};
use amethyst::core::{
    ecs::{
        Component, DenseVecStorage, Join, ReadExpect, ReadStorage, System, SystemData, WriteExpect,
    },
    math::Point3,
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
