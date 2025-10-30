use super::buffers::*;
use super::cuboid_cache::CuboidBufferCache;
use super::draw::{AuxiliaryMeta, DrawCuboids, TransformsMeta, ViewMeta};
use super::extract::{extract_clipping_planes, extract_cuboids};
use super::pipeline::{CuboidsPipelines, CuboidsShaderDefs};
use super::prepare::{
    prepare_auxiliary_bind_group, prepare_clipping_planes, prepare_cuboid_transforms,
    prepare_cuboids, prepare_cuboids_view_bind_group, prepare_materials,
};
use super::queue::queue_cuboids;
use crate::vertex_pulling::index_buffer::prepare_cuboids_index_buffer;
use crate::CuboidMaterialMap;
use bevy::asset::embedded_asset;
use bevy::core_pipeline::core_3d::Opaque3d;
use bevy::prelude::*;
use bevy::render::view::prepare_view_uniforms;
use bevy::render::{render_phase::AddRenderCommand, RenderApp};
use bevy::render::{Render, RenderSystems};

/// Renders the [`Cuboids`](crate::Cuboids) component using the "vertex pulling" technique.
#[derive(Default)]
pub struct VertexPullingRenderPlugin {
    pub outlines: bool,
}

impl Plugin for VertexPullingRenderPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "vertex_pulling.wgsl");

        app.init_resource::<CuboidMaterialMap>();

        // .add_plugins(ExtractComponentPlugin::<Cuboids>::default())
    }

    fn finish(&self, app: &mut App) {
        // let maybe_msaa = app.world.get_resource::<Msaa>().cloned();
        //
        let render_app = app.sub_app_mut(RenderApp);

        // TODO
        // if let Some(msaa) = maybe_msaa {
        //     render_app.insert_resource(msaa);
        // }
        let mut shader_defs = CuboidsShaderDefs::default();
        if self.outlines {
            shader_defs.enable_outlines();
        }
        render_app.insert_resource(shader_defs);

        render_app
            .add_render_command::<Opaque3d, DrawCuboids>()
            // TODO: pull from main app!
            .init_resource::<CuboidMaterialMap>()
            .init_resource::<AuxiliaryMeta>()
            .init_resource::<CuboidBufferCache>()
            .init_resource::<CuboidsPipelines>()
            .init_resource::<DynamicUniformBufferOfCuboidMaterial>()
            .init_resource::<DynamicUniformBufferOfCuboidTransforms>()
            .init_resource::<TransformsMeta>()
            .init_resource::<UniformBufferOfGpuClippingPlaneRanges>()
            .init_resource::<ViewMeta>()
            // TODO: Perform extraction differently
            // https://bevy.org/learn/migration-guides/0-15-to-0-16/#deprecate-insert-or-spawn-function-family
            // Or via extract_component? Now it just appends to the cache buffer...
            // https://docs.rs/bevy/0.17.2/src/custom_phase_item/custom_phase_item.rs.html#167
            .add_systems(ExtractSchedule, (extract_cuboids, extract_clipping_planes))
            .add_systems(
                Render,
                (
                    prepare_cuboids_index_buffer,
                    prepare_materials,
                    prepare_clipping_planes,
                    prepare_auxiliary_bind_group
                        .after(prepare_materials)
                        .after(prepare_clipping_planes),
                    prepare_cuboid_transforms,
                    prepare_cuboids,
                    prepare_cuboids_view_bind_group.after(prepare_view_uniforms),
                )
                    .in_set(RenderSystems::Prepare),
            )
            .add_systems(Render, queue_cuboids.in_set(RenderSystems::Queue));
    }
}
