use super::index_buffer::CUBE_INDICES;
use super::{cuboid_cache::CuboidBufferCache, index_buffer::CuboidsIndexBuffer};
use bevy::ecs::query::ROQueryItem;
use bevy::{
    ecs::system::{lifetimeless::*, SystemParamItem},
    prelude::*,
    render::{
        render_phase::{
            PhaseItem, RenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass,
        },
        render_resource::{BindGroup, IndexFormat},
        view::ViewUniformOffset,
    },
};

pub(crate) type DrawCuboids = (
    SetItemPipeline,
    SetCuboidsViewBindGroup<0>,
    SetAuxBindGroup<1>,
    SetGpuTransformBufferBindGroup<2>,
    SetGpuCuboidBuffersBindGroup<3>,
    DrawVertexPulledCuboids,
);

#[derive(Default, Resource)]
pub struct ViewMeta {
    pub cuboids_view_bind_group: Option<BindGroup>,
}

pub(crate) struct SetCuboidsViewBindGroup<const I: usize>;

impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetCuboidsViewBindGroup<I> {
    type Param = SRes<ViewMeta>;
    type ItemQuery = ();
    type ViewQuery = Read<ViewUniformOffset>;

    #[inline]
    fn render<'w>(
        _item: &P,
        view_uniform_offset: ROQueryItem<'w, '_, Self::ViewQuery>,
        _entity: Option<ROQueryItem<'w, '_, Self::ItemQuery>>,
        view_meta: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(bind_group) = view_meta.into_inner().cuboids_view_bind_group.as_ref() else {
            return RenderCommandResult::Skip;
        };
        pass.set_bind_group(I, bind_group, &[view_uniform_offset.offset]);
        RenderCommandResult::Success
    }
}

/// Holds the bind group for materials and clipping planes.
#[derive(Default, Resource)]
pub struct AuxiliaryMeta {
    pub bind_group: Option<BindGroup>,
}

pub(crate) struct SetAuxBindGroup<const I: usize>;

impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetAuxBindGroup<I> {
    type Param = (SRes<CuboidBufferCache>, SRes<AuxiliaryMeta>);
    type ItemQuery = ();
    type ViewQuery = ();

    #[inline]
    fn render<'w>(
        item: &P,
        _view: Self::ViewQuery,
        _entity: Option<Self::ItemQuery>,
        (buffer_cache, aux_meta): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        // Cuboids are queued as `BinnedRenderPhaseType::NonMesh`, which means they have no
        // render-world entity for `ItemQuery` to resolve against -- it would always be `None`.
        // The buffer cache is keyed by main-world entity (see `queue_cuboids`), so look it up
        // through the phase item instead.
        let Some(entry) = buffer_cache
            .into_inner()
            .entries
            .get(&item.main_entity().id())
        else {
            return RenderCommandResult::Skip;
        };
        let Some(bind_group) = aux_meta.into_inner().bind_group.as_ref() else {
            return RenderCommandResult::Skip;
        };
        pass.set_bind_group(I, bind_group, &[entry.material_index]);
        RenderCommandResult::Success
    }
}

#[derive(Default, Resource)]
pub struct TransformsMeta {
    pub transform_buffer_bind_group: Option<BindGroup>,
}

pub(crate) struct SetGpuTransformBufferBindGroup<const I: usize>;

impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetGpuTransformBufferBindGroup<I> {
    type Param = (SRes<CuboidBufferCache>, SRes<TransformsMeta>);
    type ItemQuery = ();
    type ViewQuery = ();

    #[inline]
    fn render<'w>(
        item: &P,
        _view: Self::ViewQuery,
        _entity: Option<Self::ItemQuery>,
        (buffer_cache, transforms_meta): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(entry) = buffer_cache
            .into_inner()
            .entries
            .get(&item.main_entity().id())
        else {
            return RenderCommandResult::Skip;
        };
        let Some(bind_group) = transforms_meta
            .into_inner()
            .transform_buffer_bind_group
            .as_ref()
        else {
            return RenderCommandResult::Skip;
        };
        pass.set_bind_group(I, bind_group, &[entry.transform_index]);
        RenderCommandResult::Success
    }
}

pub(crate) struct SetGpuCuboidBuffersBindGroup<const I: usize>;

impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetGpuCuboidBuffersBindGroup<I> {
    type Param = SRes<CuboidBufferCache>;
    type ItemQuery = ();
    type ViewQuery = ();

    #[inline]
    fn render<'w>(
        item: &P,
        _view: Self::ViewQuery,
        _entity: Option<Self::ItemQuery>,
        buffer_cache: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(entry) = buffer_cache
            .into_inner()
            .entries
            .get(&item.main_entity().id())
        else {
            return RenderCommandResult::Skip;
        };
        let Some(bind_group) = entry.instance_buffer_bind_group.as_ref() else {
            return RenderCommandResult::Skip;
        };
        pass.set_bind_group(I, bind_group, &[]);
        RenderCommandResult::Success
    }
}

pub(crate) struct DrawVertexPulledCuboids;

impl<P: PhaseItem> RenderCommand<P> for DrawVertexPulledCuboids {
    type Param = (SRes<CuboidBufferCache>, SRes<CuboidsIndexBuffer>);
    type ItemQuery = ();
    type ViewQuery = ();

    #[inline]
    fn render<'w>(
        item: &P,
        _view: Self::ViewQuery,
        _entity: Option<Self::ItemQuery>,
        (buffer_cache, index_buffer): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(entry) = buffer_cache
            .into_inner()
            .entries
            .get(&item.main_entity().id())
        else {
            return RenderCommandResult::Skip;
        };
        let num_cuboids = entry.instance_buffer.get().len().try_into().unwrap();
        pass.set_index_buffer(index_buffer.into_inner().slice(..), IndexFormat::Uint32);
        pass.draw_indexed(0..(CUBE_INDICES.len() as u32), 0, 0..num_cuboids);
        RenderCommandResult::Success
    }
}
