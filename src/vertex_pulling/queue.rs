use crate::Cuboids;

use super::cuboid_cache::CuboidBufferCache;
use super::draw::DrawCuboids;
use super::pipeline::CuboidsPipelines;

use bevy::camera::visibility::VisibleEntities;
use bevy::core_pipeline::core_3d::{Opaque3d, Opaque3dBatchSetKey, Opaque3dBinKey};
use bevy::ecs::component::Tick;
use bevy::prelude::*;
use bevy::render::mesh::allocator::SlabId;
use bevy::render::render_phase::{
    BinnedRenderPhase, BinnedRenderPhaseType, DrawFunctions, InputUniformIndex,
    PhaseItemExtraIndex, ViewBinnedRenderPhases,
};
use bevy::render::view::{ExtractedView, RenderVisibleEntities};

pub(crate) fn queue_cuboids(
    cuboids_pipelines: Res<CuboidsPipelines>,
    opaque_3d_draw_functions: Res<DrawFunctions<Opaque3d>>,
    buffer_cache: Res<CuboidBufferCache>,
    mut opaque_render_phases: ResMut<ViewBinnedRenderPhases<Opaque3d>>,
    views: Query<(&ExtractedView, &RenderVisibleEntities)>,
    mut next_tick: Local<Tick>,
) {
    let draw_cuboids = opaque_3d_draw_functions
        .read()
        .get_id::<DrawCuboids>()
        .unwrap();

    for (view, view_visible_entities) in views.iter() {
        let Some(opaque_phase) = opaque_render_phases.get_mut(&view.retained_view_entity) else {
            continue;
        };

        // TODO: add method so we can use this on a vector
        // let range_finder = view.rangefinder3d();
        let inverse_view_matrix = view.world_from_view.to_matrix().inverse();
        let inverse_view_row_2 = inverse_view_matrix.row(2);

        for &entity in view_visible_entities.get::<Cuboids>().iter() {
            // TODO: Re-enable extraction logic
            if let Some(entry) = buffer_cache.entries.get(&entity.0) {
                if entry.enabled {
                    // TODO: Replace with SpecializedRenderPipeline
                    let pipeline_id = if view.hdr {
                        cuboids_pipelines.hdr_pipeline_id
                    } else {
                        cuboids_pipelines.pipeline_id
                    };

                    // Bump the change tick in order to force Bevy to rebuild the bin.
                    let this_tick = next_tick.get() + 1;
                    next_tick.set(this_tick);

                    opaque_phase.add(
                        Opaque3dBatchSetKey {
                            draw_function: draw_cuboids,
                            pipeline: pipeline_id,
                            material_bind_group_index: None,
                            lightmap_slab: None,
                            vertex_slab: default(),
                            index_slab: None,
                        },
                        Opaque3dBinKey {
                            asset_id: AssetId::<Mesh>::invalid().untyped(),
                        },
                        entity,
                        InputUniformIndex::default(),
                        BinnedRenderPhaseType::NonMesh,
                        *next_tick,
                    );
                }
            } else {
                // warn!("Skip {entity:?} because extract_cuboids didn't add it to the cache yet");
            }
        }
    }
}
