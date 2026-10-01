use crate::Cuboids;

use super::cuboid_cache::CuboidBufferCache;
use super::draw::DrawCuboids;
use super::pipeline::{CuboidsPipelineKey, CuboidsPipelines};

use bevy::core_pipeline::core_3d::{Opaque3d, Opaque3dBatchSetKey, Opaque3dBinKey};
use bevy::prelude::*;
use bevy::render::mesh::allocator::MeshSlabs;
use bevy::render::render_phase::{
    BinnedRenderPhaseType, DrawFunctions, InputUniformIndex, ViewBinnedRenderPhases,
};
use bevy::render::render_resource::{PipelineCache, SpecializedRenderPipelines};
use bevy::render::view::{ExtractedView, RenderVisibleEntities, ViewTarget};

pub(crate) fn queue_cuboids(
    cuboids_pipelines: Res<CuboidsPipelines>,
    mut specialized_pipelines: ResMut<SpecializedRenderPipelines<CuboidsPipelines>>,
    pipeline_cache: Res<PipelineCache>,
    opaque_3d_draw_functions: Res<DrawFunctions<Opaque3d>>,
    buffer_cache: Res<CuboidBufferCache>,
    mut opaque_render_phases: ResMut<ViewBinnedRenderPhases<Opaque3d>>,
    views: Query<(
        &ExtractedView,
        &ViewTarget,
        &RenderVisibleEntities,
        Option<&Msaa>,
    )>,
) {
    let draw_cuboids = opaque_3d_draw_functions
        .read()
        .get_id::<DrawCuboids>()
        .unwrap();

    for (view, view_target, view_visible_entities, msaa) in views.iter() {
        let Some(opaque_phase) = opaque_render_phases.get_mut(&view.retained_view_entity) else {
            continue;
        };

        // The pipeline has to agree with the render pass on both the colour format and the
        // sample count, and both are per camera. Note the format has to come from the view
        // target's main texture -- which is what the 3d main pass writes into -- and not from
        // `ExtractedView::target_format`, which is the format of the view's *final* output.
        let pipeline_id = specialized_pipelines.specialize(
            &pipeline_cache,
            &cuboids_pipelines,
            CuboidsPipelineKey {
                target_format: view_target.main_texture_format(),
                samples: msaa.copied().unwrap_or_default().samples(),
            },
        );

        let Some(visible_cuboids) = view_visible_entities.get::<Cuboids>() else {
            continue;
        };

        for &entity in &visible_cuboids.entities_cpu_culling {
            if let Some(entry) = buffer_cache.entries.get(&entity.1.id()) {
                if entry.enabled {
                    opaque_phase.add(
                        Opaque3dBatchSetKey {
                            draw_function: draw_cuboids,
                            pipeline: pipeline_id,
                            material_bind_group_index: None,
                            lightmap_slab: None,
                            slabs: MeshSlabs::default(),
                        },
                        Opaque3dBinKey {
                            asset_id: AssetId::<Mesh>::invalid().untyped(),
                        },
                        entity,
                        InputUniformIndex::default(),
                        BinnedRenderPhaseType::NonMesh,
                    );
                }
            }
        }
    }
}
