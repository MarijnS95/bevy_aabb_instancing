use super::buffers::*;
use super::cuboid_cache::CuboidBufferCache;
use crate::clipping_planes::*;
use crate::cuboids::*;
use crate::CuboidMaterialId;
use crate::CuboidMaterialMap;

use bevy::{prelude::*, render::Extract};

#[allow(clippy::type_complexity)]
pub(crate) fn extract_cuboids(
    cuboids: Extract<
        Query<(
            Entity,
            Ref<Cuboids>,
            &GlobalTransform,
            &CuboidMaterialId,
            Option<&ViewVisibility>,
        )>,
    >,
    materials: Extract<Res<CuboidMaterialMap>>,
    mut materials_uniforms: ResMut<DynamicUniformBufferOfCuboidMaterial>,
    mut cuboid_buffers: ResMut<CuboidBufferCache>,
    mut transform_uniforms: ResMut<DynamicUniformBufferOfCuboidTransforms>,
) {
    transform_uniforms.clear();

    if materials.is_empty() {
        warn!("Cannot draw Cuboids with empty CuboidMaterialMap");
        return;
    }

    // First extract material so we can assign dynamic uniform indices to
    // cuboids.
    let materials_indices = materials.write_uniforms(&mut materials_uniforms);

    // Note that nothing is inserted into the render world here: `queue_cuboids` finds cuboids
    // through `RenderVisibleEntities` (driven by the main world's `VisibilityClass`) and looks
    // their buffers up in `CuboidBufferCache` by main-world entity, so a render-world copy of
    // `Cuboids` would go unread.
    for (entity, cuboids, transform, materials_id, maybe_visibility) in &cuboids {
        let instance_buffer_needs_update = cuboids.is_added() || cuboids.is_changed();
        // Filter all entities that don't have any instances. If an entity went
        // from non-empty to empty, then it will get culled from the buffer
        // cache.
        if cuboids.instances.is_empty() {
            continue;
        }

        let transform = CuboidsTransform::from_matrix(transform.to_matrix());

        let is_visible = maybe_visibility.is_none_or(|vis| vis.get());

        let entry = cuboid_buffers.entries.entry(entity).or_default();
        if instance_buffer_needs_update {
            entry.instance_buffer.set(cuboids.instances.clone());
        }
        entry.material_index = materials_indices[materials_id.0].0;
        entry.dirty = instance_buffer_needs_update;
        entry.enabled = is_visible;
        entry.keep_alive = true;
        entry.position = transform.position();
        entry.transform_index = transform_uniforms.push(&transform);
    }

    cuboid_buffers.cull_entities();
}

pub(crate) fn extract_clipping_planes(
    clipping_planes: Extract<Query<(&ClippingPlaneRange, &GlobalTransform)>>,
    mut clipping_plane_uniform: ResMut<UniformBufferOfGpuClippingPlaneRanges>,
) {
    let mut iter = clipping_planes.iter();
    let mut gpu_planes = GpuClippingPlaneRanges::default();
    for (range, transform) in iter.by_ref() {
        let (_, rotation, translation) = transform.to_scale_rotation_translation();
        gpu_planes.ranges[gpu_planes.num_ranges as usize] = GpuClippingPlaneRange {
            origin: translation,
            unit_normal: rotation * Vec3::X,
            min_sdist: range.min_sdist,
            max_sdist: range.max_sdist,
        };
        gpu_planes.num_ranges += 1;
        if gpu_planes.num_ranges as usize == MAX_CLIPPING_PLANES {
            break;
        }
    }
    if iter.next().is_some() {
        panic!(
            "Too many GpuClippingPlaneRanges entities, at most {MAX_CLIPPING_PLANES} are supported"
        );
    }
    clipping_plane_uniform.set(gpu_planes);
}
