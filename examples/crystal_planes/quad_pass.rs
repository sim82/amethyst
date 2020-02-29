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

use amethyst_error::Error;
use derivative::Derivative;
use genmesh::{
    generators::{IndexedPolygon, SharedVertex},
    Triangulate,
};
use glsl_layout::*;
pub type Triangle = crate::custom_pass::Triangle;

// lazy_static::lazy_static! {
//     // These uses the precompiled shaders.
//     // These can be obtained using glslc.exe in the vulkan sdk.
//     static ref VERTEX: SpirvShader = SpirvShader::from_bytes(
//         include_bytes!("../assets/shaders/compiled/vertex/custom.vert.spv"),
//         ShaderStageFlags::VERTEX,
//         "main",
//     ).unwrap();

//     static ref FRAGMENT: SpirvShader = SpirvShader::from_bytes(
//         include_bytes!("../assets/shaders/compiled/fragment/custom.frag.spv"),
//         ShaderStageFlags::FRAGMENT,
//         "main",
//     ).unwrap();
// }

use amethyst::renderer::rendy::shader::{PathBufShaderInfo, ShaderKind, SourceLanguage};
/// Example code of using a custom shader
///
/// Requires "shader-compiler" flag
///
/// ''' rust
use std::path::PathBuf;
lazy_static::lazy_static! {
    static ref VERTEX: SpirvShader = PathBufShaderInfo::new(
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/assets/shaders/src/vertex/quad.vert")),
        ShaderKind::Vertex,
        SourceLanguage::GLSL,
       "main",
    ).precompile().unwrap();
    static ref FRAGMENT: SpirvShader = PathBufShaderInfo::new(
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/examples/assets/shaders/src/fragment/quad.frag")),
        ShaderKind::Fragment,
        SourceLanguage::GLSL,
        "main",
    ).precompile().unwrap();
}
/// '''

/// Draw triangles.
#[derive(Clone, Debug, PartialEq, Derivative)]
#[derivative(Default(bound = ""))]
pub struct DrawQuadDesc;

impl DrawQuadDesc {
    /// Create instance of `DrawQuadDesc` render group
    pub fn new() -> Self {
        Default::default()
    }
}

impl<B: Backend> RenderGroupDesc<B, World> for DrawQuadDesc {
    fn build(
        self,
        _ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        _queue: QueueId,
        _world: &World,
        framebuffer_width: u32,
        framebuffer_height: u32,
        subpass: hal::pass::Subpass<'_, B>,
        _buffers: Vec<NodeBuffer>,
        _images: Vec<NodeImage>,
    ) -> Result<Box<dyn RenderGroup<B, World>>, failure::Error> {
        let env = DynamicUniform::new(factory, pso::ShaderStageFlags::VERTEX)?;
        let instance = DynamicVertexBuffer::new();
        let instance_const = DynamicVertexBuffer::new();

        let (pipeline, pipeline_layout) = build_custom_pipeline(
            factory,
            subpass,
            framebuffer_width,
            framebuffer_height,
            vec![env.raw_layout()],
        )?;

        Ok(Box::new(DrawQuad::<B> {
            pipeline,
            pipeline_layout,
            env,
            quad_mesh: None,
            instance,
            instance_const,
            instance_count: 0,
            change: Default::default(),
        }))
    }
}

/// Draws triangles to the screen.
#[derive(Debug)]
pub struct DrawQuad<B: Backend> {
    pipeline: B::GraphicsPipeline,
    pipeline_layout: B::PipelineLayout,
    env: DynamicUniform<B, ViewArgs>,
    quad_mesh: Option<Mesh<B>>,
    instance: DynamicVertexBuffer<B, Color>,
    instance_const: DynamicVertexBuffer<B, QuadInstanceArgsConst>,
    instance_count: usize,
    change: ChangeDetection,
}

impl<B: Backend> RenderGroup<B, World> for DrawQuad<B> {
    fn prepare(
        &mut self,
        factory: &Factory<B>,
        queue: QueueId,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        world: &World,
    ) -> PrepareResult {
        let (triangles,) = <(ReadStorage<'_, Triangle>,)>::fetch(world);

        // Get our scale value
        // let scale = world.read_resource::<QuadUniformArgs>();

        // Write to our DynamicUniform
        // self.env.write(factory, index, scale.std140());

        let projview = CameraGatherer::gather(world).projview;
        self.env.write(factory, index, projview);
        println!("prepare: {}", index);
        // println!("projview: {:?}", projview);
        let mut changed = false;
        if self.quad_mesh.is_none() {
            self.quad_mesh = Some(gen_quad_mesh(queue, &factory));
            changed = true;
        }

        if self.instance_count == 0 {
            let qi = [
                QuadInstance {
                    translate: Vector3::new(0.0, 0.0, 0.0),
                    color: Vector4::new(1.0, 0.0, 0.0, 1.0),
                    dir: 0,
                },
                QuadInstance {
                    translate: Vector3::new(0.0, 0.0, 0.0),
                    color: Vector4::new(0.0, 1.0, 0.0, 1.0),
                    dir: 1,
                },
                QuadInstance {
                    translate: Vector3::new(0.0, 0.0, 0.0),
                    color: Vector4::new(0.0, 0.0, 1.0, 1.0),
                    dir: 2,
                },
                QuadInstance {
                    translate: Vector3::new(0.0, 0.0, 0.0),
                    color: Vector4::new(1.0, 1.0, 0.0, 1.0),
                    dir: 3,
                },
                QuadInstance {
                    translate: Vector3::new(0.0, 0.0, 0.0),
                    color: Vector4::new(0.0, 1.0, 1.0, 1.0),
                    dir: 4,
                },
                QuadInstance {
                    translate: Vector3::new(0.0, 0.0, 0.0),
                    color: Vector4::new(1.0, 0.0, 1.0, 1.0),
                    dir: 5,
                },
            ];
            self.instance_count = qi.len();

            let instance_data_iter = qi.iter().map(|instance| instance.get_args());
            self.instance.write(
                factory,
                0,
                self.instance_count as u64,
                Some(instance_data_iter.collect::<Box<[Color]>>()),
            );
            let instance_data_const_iter = qi.iter().map(|instance| instance.get_args_const());

            self.instance_const.write(
                factory,
                0,
                self.instance_count as u64,
                Some(instance_data_const_iter.collect::<Box<[QuadInstanceArgsConst]>>()),
            );
            // println!("instance: {:?}", self.instance);
            changed = true;
        }
        self.change.prepare_result(index, changed)
    }

    fn draw_inline(
        &mut self,
        mut encoder: RenderPassEncoder<'_, B>,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        _world: &World,
    ) {
        // Don't worry about drawing if there are no vertices. Like before the state adds them to the screen.
        if self.quad_mesh.is_none() {
            return;
        }
        println!("draw");

        // Bind the pipeline to the the encoder
        encoder.bind_graphics_pipeline(&self.pipeline);

        // Bind the Dynamic buffer with the scale to the encoder
        self.env.bind(index, &self.pipeline_layout, 0, &mut encoder);
        println!("vertex format: {:?}", Position::vertex());

        let quad_mesh = &self.quad_mesh.as_ref().unwrap();
        quad_mesh
            .bind(0, &[Position::vertex()], &mut encoder)
            .unwrap();

        let bind2 = self.instance.bind(0, 1, 0, &mut encoder);

        let bind1 = self.instance_const.bind(0, 2, 0, &mut encoder);

        // println!("bind: {:?} {:?}", bind1, bind2);
        // Draw the vertices
        unsafe {
            // encoder.draw(0..self.vertex_count as u32, 0..self.instance_count as u32);
            encoder.draw_indexed(0..quad_mesh.len() as u32, 0, 0..self.instance_count as u32);
        }
    }

    fn dispose(self: Box<Self>, factory: &mut Factory<B>, _world: &World) {
        unsafe {
            factory.device().destroy_graphics_pipeline(self.pipeline);
            factory
                .device()
                .destroy_pipeline_layout(self.pipeline_layout);
        }
    }
}

fn build_custom_pipeline<B: Backend>(
    factory: &Factory<B>,
    subpass: hal::pass::Subpass<'_, B>,
    framebuffer_width: u32,
    framebuffer_height: u32,
    layouts: Vec<&B::DescriptorSetLayout>,
) -> Result<(B::GraphicsPipeline, B::PipelineLayout), failure::Error> {
    let pipeline_layout = unsafe {
        factory
            .device()
            .create_pipeline_layout(layouts, None as Option<(_, _)>)
    }?;

    // Load the shaders
    let shader_vertex = unsafe { VERTEX.module(factory).unwrap() };
    let shader_fragment = unsafe { FRAGMENT.module(factory).unwrap() };
    println!(
        "desc: {:?}",
        [
            (Position::vertex(), pso::VertexInputRate::Vertex),
            (
                QuadInstanceArgsConst::vertex(),
                pso::VertexInputRate::Instance(1),
            ),
            (Color::vertex(), pso::VertexInputRate::Instance(1)),
        ]
    );
    // Build the pipeline
    let pipes = PipelinesBuilder::new()
        .with_pipeline(
            PipelineDescBuilder::new()
                // This Pipeline uses our custom vertex description and does not use instancing
                .with_vertex_desc(&[
                    (Position::vertex(), pso::VertexInputRate::Vertex),
                    (Color::vertex(), pso::VertexInputRate::Instance(1)),
                    (
                        QuadInstanceArgsConst::vertex(),
                        pso::VertexInputRate::Instance(1),
                    ),
                ])
                .with_input_assembler(pso::InputAssemblerDesc::new(hal::Primitive::TriangleList))
                // Add the shaders
                .with_shaders(util::simple_shader_set(
                    &shader_vertex,
                    Some(&shader_fragment),
                ))
                .with_layout(&pipeline_layout)
                .with_subpass(subpass)
                .with_framebuffer_size(framebuffer_width, framebuffer_height)
                // We are using alpha blending
                .with_depth_test(pso::DepthTest {
                    fun: pso::Comparison::Less,
                    write: true,
                })
                .with_blend_targets(vec![pso::ColorBlendDesc {
                    mask: pso::ColorMask::ALL,
                    blend: None,
                }]),
        )
        .build(factory, None);

    // Destoy the shaders once loaded
    unsafe {
        factory.destroy_shader_module(shader_vertex);
        factory.destroy_shader_module(shader_fragment);
    }

    // Handle the Errors
    match pipes {
        Err(e) => {
            unsafe {
                factory.device().destroy_pipeline_layout(pipeline_layout);
            }
            Err(e)
        }
        Ok(mut pipes) => Ok((pipes.remove(0), pipeline_layout)),
    }
}

/// A [RenderPlugin] for our custom plugin
#[derive(Default, Debug)]
pub struct RenderQuad {}

impl<B: Backend> RenderPlugin<B> for RenderQuad {
    fn on_build<'a, 'b>(
        &mut self,
        world: &mut World,
        _builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        // Add the required components to the world ECS
        world.register::<Triangle>();
        // world.insert(QuadUniformArgs { scale: 1.0 });
        Ok(())
    }

    fn on_plan(
        &mut self,
        plan: &mut RenderPlan<B>,
        _factory: &mut Factory<B>,
        _world: &World,
    ) -> Result<(), Error> {
        plan.extend_target(Target::Main, |ctx| {
            // Add our Description
            ctx.add(RenderOrder::Transparent, DrawQuadDesc::new().builder())?;
            Ok(())
        });
        Ok(())
    }
}

// custom attributes
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct QuadDir(pub u32);
impl<T> From<T> for QuadDir
where
    T: Into<u32>,
{
    fn from(from: T) -> Self {
        QuadDir(from.into())
    }
}
impl AsAttribute for QuadDir {
    const NAME: &'static str = "dir";
    const FORMAT: Format = Format::R32Uint;
}

/// Type for position attribute of vertex.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Translate(pub [f32; 3]);
impl<T> From<T> for Translate
where
    T: Into<[f32; 3]>,
{
    fn from(from: T) -> Self {
        Translate(from.into())
    }
}
impl AsAttribute for Translate {
    const NAME: &'static str = "translate";
    const FORMAT: Format = Format::Rgb32Sfloat;
}

// #[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
// #[repr(C, align(4))]
// pub struct QuadArgs {
//     position: Position,
// }

// impl AsVertex for QuadArgs {
//     fn vertex() -> VertexFormat {
//         VertexFormat::new((
//             // position: vec3
//             Position::vertex(),
//         ))
//     }
// }

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[repr(C, align(16))]
pub struct QuadInstanceArgsConst {
    pub translate: Translate,
    pub dir: QuadDir,
    // pub color: Color,
}

impl AsVertex for QuadInstanceArgsConst {
    fn vertex() -> VertexFormat {
        VertexFormat::new((
            // color: vec3
            Translate::vertex(),
            // pad: u32
            QuadDir::vertex(),
            // Color::vertex(),
        ))
    }
}

#[derive(Clone)]
struct QuadInstance {
    translate: Vector3<f32>,
    dir: u32,
    color: Vector4<f32>,
}

impl QuadInstance {
    fn get_args(&self) -> Color {
        let color: [f32; 4] = self.color.into();
        // QuadInstanceArgs {
        //     color: color.into(),
        // }
        color.into()
    }
    fn get_args_const(&self) -> QuadInstanceArgsConst {
        let translate: [f32; 3] = self.translate.into();
        // let color: [f32; 4] = self.color.into();
        QuadInstanceArgsConst {
            translate: translate.into(),
            dir: self.dir.into(),
            // color: color.into(),
        }
    }
}

fn gen_quad_mesh<B: Backend>(queue: QueueId, factory: &Factory<B>) -> Mesh<B> {
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
