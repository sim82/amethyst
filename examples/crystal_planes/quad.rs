use crate::vertex::QuadInstanceArgsConst;
use amethyst::{
    core::{
        ecs::{
            Component, DenseVecStorage, DispatcherBuilder, Join, ReadStorage, SystemData, World,
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
use genmesh::{
    generators::{IndexedPolygon, SharedVertex},
    Triangulate,
};

#[derive(Clone)]
pub struct QuadInstance {
    pub translate: Vector3<f32>,
    pub dir: u32,
    pub color: Vector4<f32>,
    pub index: u32, // temporary for plane sorting
}

impl QuadInstance {
    pub fn get_args(&self) -> Color {
        let color: [f32; 4] = self.color.into();
        // QuadInstanceArgs {
        //     color: color.into(),
        // }
        color.into()
    }
    pub fn get_args_const(&self) -> QuadInstanceArgsConst {
        let translate: [f32; 3] = self.translate.into();
        // let color: [f32; 4] = self.color.into();
        QuadInstanceArgsConst {
            translate: translate.into(),
            dir: self.dir.into(),
            // color: color.into(),
        }
    }
}

impl Component for QuadInstance {
    type Storage = DenseVecStorage<Self>;
}

pub fn gen_quad_mesh<B: Backend>(queue: QueueId, factory: &Factory<B>) -> Mesh<B> {
    let icosphere = genmesh::generators::Plane::new();
    let indices: Vec<_> =
        genmesh::Vertices::vertices(icosphere.indexed_polygon_iter().triangulate())
            .map(|i| i as u32)
            .collect();

    println!("indices: {}", indices.len());
    let vertices: Vec<_> = icosphere
        .shared_vertex_iter()
        .map(|v| Position(v.pos.into()))
        .collect();
    println!("vertices: {}", vertices.len());
    // for v in &vertices {
    //     println!("vert: {:?}", v);
    // }
    println!("indices: {:?}", indices);
    println!("vertices: {:?}", vertices);
    let mesh = Mesh::<B>::builder()
        .with_indices(indices)
        .with_vertices(vertices)
        .build(queue, factory)
        .unwrap();

    println!("mesh: {:?}", mesh);
    mesh
}
