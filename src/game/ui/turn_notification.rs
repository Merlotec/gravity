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

#[derive(Debug, Clone, PartialEq, Component)]
pub struct UiTurnNotification {
    pub root_ent: Entity,
    pub team: Team,
    pub precursor_text: String,
}

#[derive(Debug, SystemDesc, new)]
#[system_desc(name(TurnNotificationUiSystemDesc))]
pub struct TurnNotificationUiSystem;

impl<'s> System<'s> for TurnNotificationUiSystem {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, Parent>,
        ReadStorage<'s, Team>,
        WriteStorage<'s, UiText>,
        WriteStorage<'s, UiTurnNotification>,
        WriteStorage<'s, UiTransform>,
        WriteStorage<'s, CombatRoot>,
        ReadExpect<'s, GameFonts>,
    );

    fn run(&mut self, (entities, parents, teams, mut texts, mut turn_notifications, mut ui_transforms, combat_roots, fonts): Self::SystemData) {
        // Insertion of crosshair ui elements.
        let transform_mask = ui_transforms.mask().clone();
        for (entity, turn_notification, _) in (&entities, &turn_notifications, !transform_mask).join() {
            let id: String = String::from("turn_notification:") + &entity.id().to_string();
            let mut ui_transform = UiTransform::new(
                id,
                Anchor::TopMiddle,
                Anchor::TopMiddle,
                0.0, -10.0, 0.0,
                200.0, 60.0,
            );

            texts.insert(entity, UiText::new(fonts.ability().clone(), "Your Turn".to_string(), match turn_notification.team {
                Team::Friendly => [0.0, 1.0, 0.0, 1.0],
                Team::Enemy => [1.0, 0.0, 0.0, 1.0],
            }, 25.0));

            ui_transforms.insert(entity, ui_transform);
        }

        for (mut turn_notification, mut text) in (&mut turn_notifications, &mut texts).join() {
            if let Some(root) = combat_roots.get(turn_notification.root_ent) {
                if let CombatState::InTurn(team) = root.current_state {
                    if team != turn_notification.team {
                        turn_notification.team = team;
                        match team {
                            Team::Friendly => {
                                text.color = [0.0, 1.0, 0.0, 1.0];
                                text.text = "Your Turn".to_string();
                            },
                            Team::Enemy => {
                                text.color = [1.0, 0.0, 0.0, 1.0];
                                text.text = "Enemy Turn".to_string();
                            },
                        };
                    }
                }
            }
        }
    }
}
