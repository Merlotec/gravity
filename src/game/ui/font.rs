use amethyst::{
    assets::{
        AssetStorage,
        Loader,
    },
    ecs::prelude::*,
    ui::{
        FontAsset,
        FontHandle,
        TtfFormat,
    },
};

pub struct GameFonts {
    pub standard: FontHandle,
}

impl GameFonts {
    pub fn load(world: &mut World) -> Self {
        if !world.has_value::<AssetStorage<FontAsset>>() {
            world.insert::<AssetStorage<FontAsset>>(AssetStorage::new())
        }
        let standard: FontHandle = world.fetch::<Loader>().load(
            "fonts/consola.ttf",
            TtfFormat,
            (),
            &world.fetch_mut::<AssetStorage<FontAsset>>(),
        );
        Self {
            standard,
        }
    }

    pub fn ability(&self) -> &FontHandle {
        &self.standard
    }

    pub fn status(&self) -> &FontHandle {
        &self.standard
    }
}