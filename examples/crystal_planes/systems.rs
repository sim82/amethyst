use crate::{
    crystal,
    crystal::{rads::Scene, PlanesSep},
    math::prelude::*,
    quad,
    quad::QuadInstance,
};

#[allow(unused_imports)]
use amethyst::prelude::*;

use amethyst::core::ecs::{Join, ReadExpect, System, SystemData, Write, WriteExpect, WriteStorage};
use amethyst_derive::SystemDesc;
use rand::Rng; //prelude::*;

#[derive(SystemDesc)]
#[system_desc(name(RandomFlashingEmitSystemDesc))]
pub struct RandomFlashingEmitSystem;
impl<'a> System<'a> for RandomFlashingEmitSystem {
    type SystemData = WriteExpect<'a, Scene>;

    fn run(&mut self, mut rad_scene: Self::SystemData) {
        let mut rand = rand::thread_rng();
        use random_color::{Luminosity, RandomColor};
        let mut rc = RandomColor::new();
        rc.luminosity(Luminosity::Bright);
        for emit in &mut rad_scene.emit {
            let color = if rand.gen_bool(0.1) {
                rc.to_rgb_array()
            } else {
                [0; 3]
            };
            emit[0] = color[0] as f32 / 255.0;
            emit[1] = color[1] as f32 / 255.0;
            emit[2] = color[2] as f32 / 255.0;
        }
    }
}

#[derive(SystemDesc)]
#[system_desc(name(TronEmitSystemDesc))]
pub struct TronEmitSystem;
impl<'a> System<'a> for TronEmitSystem {
    type SystemData = WriteExpect<'a, Scene>;

    fn run(&mut self, mut rad_scene: Self::SystemData) {
        let mut rand = rand::thread_rng();
        use random_color::{Luminosity, RandomColor};
        let mut rc = RandomColor::new();
        rc.luminosity(Luminosity::Bright);
        let color = if rand.gen_bool(0.1) {
            rc.to_rgb_array()
        } else {
            [0; 3]
        };
        for emit in &mut rad_scene.emit {
            emit[0] = color[0] as f32 / 255.0;
            emit[1] = color[1] as f32 / 255.0;
            emit[2] = color[2] as f32 / 255.0;
        }
    }
}

#[derive(SystemDesc)]
#[system_desc(name(RunRadSceneSystemDesc))]
pub struct RunRadSceneSystem;
impl<'a> System<'a> for RunRadSceneSystem {
    type SystemData = WriteExpect<'a, Scene>;

    fn run(&mut self, mut rad_scene: Self::SystemData) {
        rad_scene.do_rad();
    }
}

#[derive(SystemDesc)]
#[system_desc(name(CopyRadFrontSystemDesc))]
pub struct CopyRadFrontSystem;
impl<'a> System<'a> for CopyRadFrontSystem {
    type SystemData = (
        WriteStorage<'a, QuadInstance>,
        ReadExpect<'a, Scene>,
        Write<'a, Option<quad::ColorGeneration>>,
    );

    fn run(&mut self, (mut quad_instances, rad_scene, mut color_generation): Self::SystemData) {
        for q in (&mut quad_instances).join() {
            if (q.index as usize) < rad_scene.rad_front.r.len() {
                q.color[0] = rad_scene.rad_front.r[q.index as usize];
                q.color[1] = rad_scene.rad_front.g[q.index as usize];
                q.color[2] = rad_scene.rad_front.b[q.index as usize];
            }
        }
        if let Some(ref mut color_generation) = *color_generation {
            color_generation.0 += 1;
        }
    }
}

#[derive(SystemDesc)]
#[system_desc(name(ApplyDiffuseColorSystemDesc))]
pub struct ApplyDiffuseColorSystem {
    up_to_date: bool,
}
impl Default for ApplyDiffuseColorSystem {
    fn default() -> Self {
        ApplyDiffuseColorSystem { up_to_date: false }
    }
}
impl<'a> System<'a> for ApplyDiffuseColorSystem {
    type SystemData = (WriteExpect<'a, PlanesSep>, WriteExpect<'a, Scene>);

    fn run(&mut self, (mut planes, mut scene): Self::SystemData) {
        if self.up_to_date {
            return;
        }
        let color1 = Vec3::new(1f32, 0.5f32, 0f32);
        // let color2 = hsv_to_rgb(rng.gen_range(0.0, 360.0), 1.0, 1.0);
        let color2 = Vec3::new(0f32, 1f32, 0f32);
        for (i, plane) in planes.planes_iter().enumerate() {
            if ((plane.cell.y) / 2) % 2 == 1 {
                continue;
            }
            scene.diffuse[i] = match plane.dir {
                crystal::Dir::XyPos => color1,
                crystal::Dir::XyNeg => color2,
                crystal::Dir::YzPos | crystal::Dir::YzNeg => Vec3::new(0.8f32, 0.8f32, 0.8f32),
                _ => Vec3::new(1f32, 1f32, 1f32),
                // let color = hsv_to_rgb(rng.gen_range(0.0, 360.0), 1.0, 1.0); //random::<f32>(), 1.0, 1.0);
                // scene.diffuse[i] = Vector3::new(color.0, color.1, color.2);
            }
        }
        self.up_to_date = true;
    }
}
