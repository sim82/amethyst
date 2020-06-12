use crate::{
    crystal::{BlockMap, PlanesSep, Scene},
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
use std::sync::Arc;

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
        WriteExpect<'a, Arc<Scene>>,
        ReadExpect<'a, PlanesSep>,
        ReadExpect<'a, BlockMap>,
        ReadStorage<'a, PointLight>,
    );

    fn run(&mut self, (mut rad_scene, planes, blockmap, point_lights): Self::SystemData) {
        let mut frontend = rad_scene.lock_frontend();
        frontend.clear_emit();
        for light in point_lights.join() {
            frontend.apply_light(&planes, &blockmap, &light.pos, &light.color);
        }
    }
}

#[derive(SystemDesc)]
#[system_desc(name(ApplyRendyLightsSystemSystemDesc))]
pub struct ApplyRendyLightsSystem;
impl<'a> System<'a> for ApplyRendyLightsSystem {
    type SystemData = (
        WriteExpect<'a, Arc<Scene>>,
        ReadExpect<'a, PlanesSep>,
        ReadExpect<'a, BlockMap>,
        ReadStorage<'a, Light>,
        ReadStorage<'a, Transform>,
    );

    fn run(&mut self, (mut rad_scene, planes, blockmap, light, transform): Self::SystemData) {
        let mut frontend = rad_scene.lock_frontend();
        frontend.clear_emit();
        for (light, transform) in (&light, &transform).join() {
            if let Light::Point(point_light) = light {
                // FIXME: this is broken, and much too complicated for just getting the light's global translation...
                let pos = transform.global_view_matrix().try_inverse().unwrap()
                    * Vec4::new(0.0, 0.0, 0.0, 1.0);
                // println!("transform: {:?}", transform);
                let pos = Point3::from_homogeneous(pos).unwrap();
                // println!("pos: {:?}", pos);
                frontend.apply_light(
                    &planes,
                    &blockmap,
                    &pos,
                    &Color::new(
                        point_light.color.red,
                        point_light.color.green,
                        point_light.color.blue,
                    ),
                );
            }
        }
    }
}
