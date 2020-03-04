use crate::{crystal::rads::Scene, light, quad, quad::QuadInstance};
use amethyst::{
    core::{
        ecs::{
            Component, DenseVecStorage, DispatcherBuilder, Join, ReadExpect, ReadStorage, System,
            SystemData, World, Write, WriteExpect, WriteStorage,
        },
        math::{convert, Matrix4, Point2, Point3, Vector2, Vector3, Vector4},
    },
    prelude::*,
    renderer::{
        bundle::{RenderOrder, RenderPlan, RenderPlugin, Target},
        pipeline::{PipelineDescBuilder, PipelinesBuilder},
        pod::{IntoPod, ViewArgs},
        rendy::{
            command::{QueueId, RenderPassEncoder},
            factory::Factory,
            graph::{
                render::{PrepareResult, RenderGroup, RenderGroupDesc},
                GraphContext, NodeBuffer, NodeImage,
            },
            hal::{self, device::Device, format::Format, pso, pso::ShaderStageFlags},
            mesh::{AsAttribute, AsVertex, Color, Mesh, Position, VertexFormat},
            shader::{Shader, SpirvShader},
        },
        submodules::{gather::CameraGatherer, DynamicUniform, DynamicVertexBuffer},
        types::Backend,
        util, ChangeDetection,
    },
};
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

    fn run(&mut self, (mut rad_scene): Self::SystemData) {
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

    fn run(&mut self, (mut quad_instances, mut rad_scene, mut color_generation): Self::SystemData) {
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
