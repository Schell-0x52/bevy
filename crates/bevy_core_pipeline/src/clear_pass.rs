use std::collections::HashSet;

use crate::{ClearColor, RenderTargetClearColors};
use bevy_ecs::prelude::*;
use bevy_render::{
    camera::{ExtractedCamera, RenderTarget, ClearOp},
    prelude::Image,
    render_asset::RenderAssets,
    render_graph::{Node, NodeRunError, RenderGraphContext, SlotInfo},
    render_resource::{
        LoadOp, Operations, RenderPassColorAttachment, RenderPassDepthStencilAttachment,
        RenderPassDescriptor,
    },
    renderer::RenderContext,
    view::{ExtractedView, ExtractedWindows, ViewDepthTexture, ViewTarget},
};

pub struct ClearPassNode {
    query: QueryState<
        (
            &'static ExtractedCamera,
            Option<&'static ViewTarget>,
            Option<&'static ViewDepthTexture>,
        ),
        With<ExtractedView>,
    >,
}

impl ClearPassNode {
    pub fn new(world: &mut World) -> Self {
        Self {
            query: QueryState::new(world),
        }
    }
}

impl Node for ClearPassNode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![]
    }

    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
    }

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {

        for (camera, color_target, depth_target) in self.query.iter_manual(world) {
            let color_attachments = color_target.and_then(|target| {
                match camera.clear.color {
                    ClearOp::Value(v) =>
                        Some([target.get_color_attachment(Operations{
                            load: LoadOp::Clear(v.into()),
                            store: true,
                        })]),
                    _ => None,
                }
            });
            let depth_stencil_attachment = depth_target.and_then(|target| {
                match camera.clear.depth {
                    ClearOp::Value(v) =>
                        Some(RenderPassDepthStencilAttachment {
                            view: &target.view,
                            depth_ops: Some(Operations{
                                load: LoadOp::Clear(v),
                                store: true,
                            }),
                            stencil_ops: None,
                        }),
                    _ => None,
                }
            });

            if color_attachments.is_some() || depth_stencil_attachment.is_some() {
                let pass_descriptor = RenderPassDescriptor {
                    label: Some("clear_pass"),
                    color_attachments: match &color_attachments {
                        Some(color) => color,
                        None => &[],
                    },
                    depth_stencil_attachment,
                };

                render_context
                    .command_encoder
                    .begin_render_pass(&pass_descriptor);
            }
        }

        Ok(())
    }
}
