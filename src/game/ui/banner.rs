use std::collections::HashMap;

use amethyst::{
    assets::{
        AssetStorage,
        Handle,
        Loader,
    },
    core::{
        math::{
            Vector2,
            Vector3,
            Vector4,
        },
        Parent,
        Transform,
    },
    ecs::prelude::*,
    prelude::SystemDesc,
    renderer::{
        ActiveCamera,
        Camera,
        formats::texture::ImageFormat,
        Sprite,
        Texture,
    },
    shrev::{
        EventChannel,
        ReaderId,
    },
    ui::{
        Anchor,
        FontAsset,
        FontHandle,
        Interactable,
        TtfFormat,
        UiButton,
        UiEvent,
        UiEventType,
        UiImage,
        UiText,
        UiTransform,
    },
    window::ScreenDimensions,
};

use crate::game::combat::{CombatRoot, CombatState, Team};
use crate::game::ui::font::GameFonts;
use crate::game::ui::hud::UiBase;

#[derive(Debug, Clone, PartialEq)]
pub struct ShowUiBannerDisplayEvent {
    pub owner: Entity,
    pub color: [f32; 4],
    pub text: String,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Component)]
pub struct UiBannerDisplay {
    pub owner: Entity,
}

#[derive(Debug, SystemDesc, new)]
#[system_desc(name(BannerUiSystemDesc))]
pub struct BannerUiSystem {
    #[system_desc(event_channel_reader)]
    show_banner_event_reader: ReaderId<ShowUiBannerDisplayEvent>,
}

impl<'s> System<'s> for BannerUiSystem {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, Parent>,
        ReadStorage<'s, Team>,
        WriteStorage<'s, UiText>,
        WriteStorage<'s, UiImage>,
        WriteStorage<'s, UiBannerDisplay>,
        WriteStorage<'s, UiTransform>,
        WriteStorage<'s, CombatRoot>,
        ReadExpect<'s, GameFonts>,
        Read<'s, EventChannel<ShowUiBannerDisplayEvent>>,
    );

    fn run(&mut self, (entities, parents, teams, mut texts, mut images, mut turn_notifications, mut ui_transforms, combat_roots, fonts, show_banner_events): Self::SystemData) {
        for event in show_banner_events.read(&mut self.show_banner_event_reader) {
            let banner_ent: Entity = entities.create();
            let id: String = String::from("banner:") + &banner_ent.id().to_string();
            let mut ui_transform = UiTransform::new(
                id,
                Anchor::Middle,
                Anchor::Middle,
                0.0, 0.0, 0.0,
                10000.0, 200.0,
            );

            let mut text = UiText::new(fonts.ability().clone(), event.text.clone(), event.color, 40.0);
            text.align = Anchor::Middle;
            texts.insert(banner_ent, text);
            images.insert(banner_ent, UiImage::SolidColor([0.005, 0.005, 0.006, 0.9]));
            ui_transforms.insert(banner_ent, ui_transform);
        }
    }
}
