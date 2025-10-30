use super::buffers::*;
use super::cuboid_cache::CuboidBufferCache;
use crate::clipping_planes::*;
use crate::cuboids::*;
use crate::CuboidMaterialId;
use crate::CuboidMaterialMap;

use bevy::ecs::system::command::insert_batch;
use bevy::render::extract_component::ExtractComponent;
use bevy::{prelude::*, render::Extract};

#[allow(clippy::type_complexity)]
pub(crate) fn extract_cuboids(
    mut prev_extracted_entities_size: Local<usize>,
    mut commands: Commands,
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

    // TODO: Replace logic
    // https://bevy.org/learn/migration-guides/0-15-to-0-16/#deprecate-insert-or-spawn-function-family

    let mut extracted_entities = Vec::with_capacity(*prev_extracted_entities_size);
    for (entity, cuboids, transform, materials_id, maybe_visibility) in &cuboids {
        let instance_buffer_needs_update = cuboids.is_added() || cuboids.is_changed();
        // Filter all entities that don't have any instances. If an entity went
        // from non-empty to empty, then it will get culled from the buffer
        // cache.
        if cuboids.instances.is_empty() {
            continue;
        }

        if let Some(c) = Cuboids::extract_component(&cuboids) {
            extracted_entities.push((entity, c));
        } else {
            panic!()
        }
        let transform = CuboidsTransform::from_matrix(transform.to_matrix());

        let is_visible = maybe_visibility.is_none_or(|vis| vis.get());

        info!("Add {entity:?}");
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

    info!("{}", extracted_entities.len());

    *prev_extracted_entities_size = extracted_entities.len();
    // TODO: Panics on empty bundle
    // commands.insert_batch(extracted_entities);
    commands.try_insert_batch(extracted_entities);

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
