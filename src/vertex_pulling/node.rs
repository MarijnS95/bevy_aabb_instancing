use bevy::{
    ecs::{query::QueryItem, world::World},
    render::{
        camera::ExtractedCamera,
        render_graph::{NodeRunError, RenderGraphContext, RenderLabel, ViewNode},
        render_phase::{RenderPhase, TrackedRenderPass},
        render_resource::{CommandEncoderDescriptor, RenderPassDescriptor, StoreOp},
        renderer::RenderContext,
        view::{ViewDepthTexture, ViewTarget},
    },
};

use super::phase::AabbOpaque3d;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct AabbOpaquePass3dLabel;

#[derive(Default)]
pub struct AabbOpaquePass3dNode;

impl ViewNode for AabbOpaquePass3dNode {
    type ViewQuery = (
        &'static ExtractedCamera,
        &'static RenderPhase<AabbOpaque3d>,
        &'static ViewTarget,
        &'static ViewDepthTexture,
    );

    fn run<'w>(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        (camera, aabb_opaque_phase, target, depth): QueryItem<'w, Self::ViewQuery>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let color_attachments = [Some(target.get_color_attachment())];
        let depth_stencil_attachment = Some(depth.get_attachment(StoreOp::Store));

        let view_entity = graph.view_entity();
        render_context.add_command_buffer_generation_task(move |render_device| {
            #[cfg(feature = "trace")]
            let _main_opaque_pass_3d_span = info_span!("main_opaque_pass_3d").entered();

            // Command encoder setup
            let mut command_encoder =
                render_device.create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("aabb_opaque_pass_3d_command_encoder"),
                });

            // Render pass setup
            let render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("aabb_opaque_pass_3d"),
                color_attachments: &color_attachments,
                depth_stencil_attachment,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            let mut render_pass = TrackedRenderPass::new(&render_device, render_pass);
            if let Some(viewport) = camera.viewport.as_ref() {
                render_pass.set_camera_viewport(viewport);
            }

            // Opaque draws
            if !aabb_opaque_phase.items.is_empty() {
                #[cfg(feature = "trace")]
                let _opaque_main_pass_3d_span = info_span!("aabb_main_pass_3d").entered();
                aabb_opaque_phase.render(&mut render_pass, world, view_entity);
            }

            drop(render_pass);
            command_encoder.finish()
        });

        Ok(())
    }
}
