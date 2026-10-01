use crate::clipping_planes::GpuClippingPlaneRanges;
use crate::{cuboids::CuboidsTransform, CuboidMaterial};

use bevy::asset::load_embedded_asset;
use bevy::mesh::PrimitiveTopology;
use bevy::shader::ShaderDefVal;
use bevy::{
    prelude::*,
    render::{
        render_resource::{
            BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
            BlendState, BufferBindingType, BufferSize, ColorTargetState, ColorWrites,
            CompareFunction, DepthBiasState, DepthStencilState, FragmentState, FrontFace,
            MultisampleState, PolygonMode, PrimitiveState, RenderPipelineDescriptor, ShaderStages,
            ShaderType, SpecializedRenderPipeline, StencilFaceState, StencilState, TextureFormat,
            VertexState,
        },
        renderer::RenderDevice,
        view::ViewUniform,
    },
};

/// Specialization key for [`CuboidsPipelines`].
///
/// MSAA is configured per camera (it is a `Msaa` component on the view, not a global resource),
/// and the sample count baked into a pipeline has to match the render pass it is used in, so it
/// cannot be fixed up front.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct CuboidsPipelineKey {
    pub target_format: TextureFormat,
    pub samples: u32,
}

#[derive(Resource)]
pub(crate) struct CuboidsPipelines {
    shader: Handle<Shader>,
    shader_defs: CuboidsShaderDefs,

    pub aux_layout: BindGroupLayout,
    pub cuboids_layout: BindGroupLayout,
    pub transforms_layout: BindGroupLayout,
    pub view_layout: BindGroupLayout,

    /// Descriptions of the four layouts above, in bind group order.
    ///
    /// `RenderPipelineDescriptor` describes its layouts rather than taking already-created ones,
    /// so keep both: the descriptors for the pipeline, and the concrete layouts for building the
    /// bind groups in `prepare`.
    layout_descriptors: Vec<BindGroupLayoutDescriptor>,
}

impl FromWorld for CuboidsPipelines {
    fn from_world(world: &mut World) -> Self {
        let vertex_pulling = load_embedded_asset!(world, "vertex_pulling.wgsl");

        let render_device = world.resource::<RenderDevice>();

        let view_layout_descriptor = BindGroupLayoutDescriptor::new(
            "cuboids_view_layout",
            &[
                // View
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: BufferSize::new(ViewUniform::min_size().get()),
                    },
                    count: None,
                },
            ],
        );
        let view_layout = render_device
            .create_bind_group_layout(Some("cuboids_view_layout"), &view_layout_descriptor.entries);

        let aux_layout_descriptor = BindGroupLayoutDescriptor::new(
            "aux_layout",
            &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: Some(CuboidMaterial::min_size()),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(GpuClippingPlaneRanges::min_size()),
                    },
                    count: None,
                },
            ],
        );
        let aux_layout = render_device
            .create_bind_group_layout(Some("aux_layout"), &aux_layout_descriptor.entries);

        let transforms_layout_descriptor = BindGroupLayoutDescriptor::new(
            "transforms_layout",
            &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    // We always have a single transform for each instance buffer.
                    min_binding_size: Some(CuboidsTransform::min_size()),
                },
                count: None,
            }],
        );
        let transforms_layout = render_device.create_bind_group_layout(
            Some("transforms_layout"),
            &transforms_layout_descriptor.entries,
        );

        let cuboids_layout_descriptor = BindGroupLayoutDescriptor::new(
            "cuboid_instances_layout",
            &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: BufferSize::new(0),
                },
                count: None,
            }],
        );
        let cuboids_layout = render_device.create_bind_group_layout(
            Some("cuboid_instances_layout"),
            &cuboids_layout_descriptor.entries,
        );

        let layout_descriptors = vec![
            view_layout_descriptor,
            aux_layout_descriptor,
            transforms_layout_descriptor,
            cuboids_layout_descriptor,
        ];

        let shader_defs = world.resource::<CuboidsShaderDefs>().clone();

        Self {
            layout_descriptors,
            shader: vertex_pulling,
            shader_defs,
            view_layout,
            aux_layout,
            cuboids_layout,
            transforms_layout,
        }
    }
}

impl SpecializedRenderPipeline for CuboidsPipelines {
    type Key = CuboidsPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: Some("cuboids_pipeline".into()),
            layout: self.layout_descriptors.clone(),
            vertex: VertexState {
                shader: self.shader.clone(),
                shader_defs: self.shader_defs.vertex.clone(),
                entry_point: Some("vertex".into()),
                buffers: vec![],
            },
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                shader_defs: self.shader_defs.fragment.clone(),
                entry_point: Some("fragment".into()),
                targets: vec![Some(ColorTargetState {
                    format: key.target_format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(CompareFunction::Greater),
                stencil: StencilState {
                    front: StencilFaceState::IGNORE,
                    back: StencilFaceState::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                },
                bias: DepthBiasState {
                    constant: 0,
                    slope_scale: 0.0,
                    clamp: 0.0,
                },
            }),
            multisample: MultisampleState {
                count: key.samples,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            immediate_size: 0,
            zero_initialize_workgroup_memory: false,
        }
    }
}

#[derive(Clone, Default, Resource)]
pub(crate) struct CuboidsShaderDefs {
    pub vertex: Vec<ShaderDefVal>,
    pub fragment: Vec<ShaderDefVal>,
}

impl CuboidsShaderDefs {
    pub fn enable_outlines(&mut self) {
        self.vertex.push("OUTLINES".into());
        self.fragment.push("OUTLINES".into());
    }
}
