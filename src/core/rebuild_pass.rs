use std::ops::Range;

use amethyst::{
    assets::{
        Loader,
        AssetStorage,
        Handle,
    },
    core::ecs::{
        DispatcherBuilder, World,
    },
    error::Error,
    renderer::{
        bundle::{RenderOrder, RenderPlan, RenderPlugin, Target},
        pipeline::{PipelineDescBuilder, PipelinesBuilder},
        rendy::{
            command::{QueueId, RenderPassEncoder},
            factory::{Factory, ImageState},
            graph::{
                GraphContext,
                NodeBuffer, NodeImage, render::{PrepareResult, RenderGroup, RenderGroupDesc},
            },
            hal::{self, device::Device,  pso, pso::ShaderStageFlags},
            mesh::{AsVertex, Position, TexCoord, PosTex},
            shader::{Shader, SpirvShader},
            texture::{TextureBuilder, pixel::Rgba8Srgb},
        },
        submodules::{
            FlatEnvironmentSub,
            TextureSub,
            TextureId,
        },
        Texture,
        types::Backend, util,
    },
};

use std::io::Cursor;
use super::*;

use amethyst::prelude::WorldExt;

pub struct RebuildRendering(pub bool);

/// A [RenderPlugin] for our custom plugin
#[derive(Debug, Default)]
pub struct RebuildPlugin;

impl<B: Backend> RenderPlugin<B> for RebuildPlugin {
    fn should_rebuild(&mut self, world: &World) -> bool {
        let mut should_rebuild = world.fetch_mut::<RebuildRendering>();
        if should_rebuild.0 {
            should_rebuild.0 = false;
            return true;
        } else {
            false
        }
    }

    fn on_build<'a, 'b>(
        &mut self,
        world: &mut World,
        _builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        world.insert(RebuildRendering(false));
        Ok(())
    }

    fn on_plan(
        &mut self,
        plan: &mut RenderPlan<B>,
        _factory: &mut Factory<B>,
        _world: &World,
    ) -> Result<(), Error> {
        Ok(())
    }
}