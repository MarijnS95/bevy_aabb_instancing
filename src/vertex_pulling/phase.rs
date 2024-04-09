use std::{cmp::Reverse, ops::Range};

use bevy::{ecs::entity::Entity, render::{render_phase::{CachedRenderPipelinePhaseItem, DrawFunctionId, PhaseItem}, render_resource::CachedRenderPipelineId}, utils::{nonmax::NonMaxU32, FloatOrd}};

pub struct AabbOpaque3d {
    pub distance: f32,
    pub pipeline: CachedRenderPipelineId,
    pub entity: Entity,
    pub draw_function: DrawFunctionId,
    pub batch_range: Range<u32>,
    pub dynamic_offset: Option<NonMaxU32>,
}

impl PhaseItem for AabbOpaque3d {
    // NOTE: Values increase towards the camera. Front-to-back ordering for opaque means we need a descending sort.
    type SortKey = Reverse<FloatOrd>;

    #[inline]
    fn entity(&self) -> Entity {
        self.entity
    }

    #[inline]
    fn sort_key(&self) -> Self::SortKey {
        Reverse(FloatOrd(self.distance))
    }

    #[inline]
    fn draw_function(&self) -> DrawFunctionId {
        self.draw_function
    }

    #[inline]
    fn sort(items: &mut [Self]) {
        // Key negated to match reversed SortKey ordering
        items.sort_by_key(|item| item.sort_key());
        // radsort::sort_by_key(items, |item| -item.distance);
    }

    #[inline]
    fn batch_range(&self) -> &Range<u32> {
        &self.batch_range
    }

    #[inline]
    fn batch_range_mut(&mut self) -> &mut Range<u32> {
        &mut self.batch_range
    }

    #[inline]
    fn dynamic_offset(&self) -> Option<NonMaxU32> {
        self.dynamic_offset
    }

    #[inline]
    fn dynamic_offset_mut(&mut self) -> &mut Option<NonMaxU32> {
        &mut self.dynamic_offset
    }
}

impl CachedRenderPipelinePhaseItem for AabbOpaque3d {
    #[inline]
    fn cached_pipeline(&self) -> CachedRenderPipelineId {
        self.pipeline
    }
}