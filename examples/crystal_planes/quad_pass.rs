use amethyst::{
    core::ecs::{
        Component, DenseVecStorage, DispatcherBuilder, Join, ReadStorage, SystemData, World,
    },
    prelude::*,
    renderer::{
        bundle::{RenderOrder, RenderPlan, RenderPlugin, Target},
        pipeline::{PipelineDescBuilder, PipelinesBuilder},
        pod::ViewArgs,
        rendy::{
            command::{QueueId, RenderPassEncoder},
            factory::Factory,
            graph::{
                render::{PrepareResult, RenderGroup, RenderGroupDesc},
                GraphContext, NodeBuffer, NodeImage,
            },
            hal::{self, device::Device, format::Format, pso, pso::ShaderStageFlags},
            mesh::{AsVertex, VertexFormat},
            shader::{Shader, SpirvShader},
        },
        submodules::{gather::CameraGatherer, DynamicUniform, DynamicVertexBuffer},
        types::Backend,
        util, ChangeDetection,
    },
};

use amethyst_error::Error;
use derivative::Derivative;
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
        let vertex = DynamicVertexBuffer::new();

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
            vertex,
            vertex_count: 0,
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
    vertex: DynamicVertexBuffer<B, QuadArgs>,
    vertex_count: usize,
    change: ChangeDetection,
}

impl<B: Backend> RenderGroup<B, World> for DrawQuad<B> {
    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
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

        println!("projview: {:?}", projview);

        //Update vertex count and see if it has changed
        let old_vertex_count = self.vertex_count;
        self.vertex_count = triangles.join().count() * 3;
        let changed = old_vertex_count != self.vertex_count;

        // Create an iterator over the Triangle vertices
        let vertex_data_iter = triangles.join().flat_map(|triangle| triangle.get_args());

        // Write the vector to a Vertex buffer
        self.vertex.write(
            factory,
            index,
            self.vertex_count as u64,
            Some(vertex_data_iter.collect::<Box<[QuadArgs]>>()),
        );

        // Return with we can reuse the draw buffers using the utility struct ChangeDetection
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
        if self.vertex_count == 0 {
            return;
        }

        // Bind the pipeline to the the encoder
        encoder.bind_graphics_pipeline(&self.pipeline);

        // Bind the Dynamic buffer with the scale to the encoder
        self.env.bind(index, &self.pipeline_layout, 0, &mut encoder);

        // Bind the vertex buffer to the encoder
        self.vertex.bind(index, 0, 0, &mut encoder);

        // Draw the vertices
        unsafe {
            encoder.draw(0..self.vertex_count as u32, 0..1);
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

    // Build the pipeline
    let pipes = PipelinesBuilder::new()
        .with_pipeline(
            PipelineDescBuilder::new()
                // This Pipeline uses our custom vertex description and does not use instancing
                .with_vertex_desc(&[(QuadArgs::vertex(), pso::VertexInputRate::Vertex)])
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
                .with_blend_targets(vec![pso::ColorBlendDesc {
                    mask: pso::ColorMask::ALL,
                    blend: Some(pso::BlendState::ALPHA),
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

/// Vertex Arguments to pass into shader.
/// VertexData in shader:
/// layout(location = 0) out VertexData {
///    vec2 pos;
///    vec4 color;
/// } vertex;
// #[derive(Clone, Copy, Debug, PartialEq, PartialOrd, AsStd140)]
// #[repr(C, align(4))]
// pub struct QuadArgs {
//     /// vec2 pos;
//     pub pos: vec2,
//     /// vec4 color;
//     pub color: vec4,
// }

// /// Required to send data into the shader.
// /// These names must match the shader.
// impl AsVertex for QuadArgs {
//     fn vertex() -> VertexFormat {
//         VertexFormat::new((
//             // vec2 pos;
//             (Format::Rg32Sfloat, "pos"),
//             // vec4 color;
//             (Format::Rgba32Sfloat, "color"),
//         ))
//     }
// }

type QuadArgs = crate::custom_pass::CustomArgs;

// /// QuadUniformArgs
// /// A Uniform we pass into the shader containing the current scale.
// /// Uniform in shader:
// /// layout(std140, set = 0, binding = 0) uniform QuadUniformArgs {
// ///    uniform float scale;
// /// };

// #[derive(Clone, Copy, Debug, AsStd140)]
// #[repr(C, align(4))]
// pub struct QuadUniformArgs {
//     /// The value each vertex is scaled by.
//     pub scale: float,
// }
