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
        Named,
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
use crate::game::map::{MapRoot, CurrentState, MapStage, MapPoint};
use crate::game::combat::process::Principal;

#[derive(Debug, Clone, PartialEq, Component)]
pub struct UiMapNotification {
    pub root_ent: Entity,
}

#[derive(Debug, SystemDesc, new)]
#[system_desc(name(MapNotificationUiSystemDesc))]
pub struct MapNotificationUiSystem;

impl<'s> System<'s> for MapNotificationUiSystem {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, Parent>,
        ReadStorage<'s, Principal>,
        ReadStorage<'s, Team>,
        ReadStorage<'s, MapPoint>,
        ReadStorage<'s, Named>,
        WriteStorage<'s, UiText>,
        WriteStorage<'s, UiMapNotification>,
        WriteStorage<'s, UiTransform>,
        WriteStorage<'s, MapRoot>,
        ReadExpect<'s, GameFonts>,
        Read<'s, CurrentState>,
    );

    fn run(&mut self, (entities, parents, principals, teams, points, names, mut texts, mut map_notifications, mut ui_transforms, map_roots, fonts, current_state): Self::SystemData) {
        // Insertion of crosshair ui elements.
        let transform_mask = ui_transforms.mask().clone();
        for (entity, map_notification, _) in (&entities, &map_notifications, !transform_mask).join() {
            let id: String = String::from("map_notification:") + &entity.id().to_string();
            let mut ui_transform = UiTransform::new(
                id,
                Anchor::TopMiddle,
                Anchor::TopMiddle,
                0.0, -10.0, 0.0,
                1000.0, 60.0,
            );

            texts.insert(entity, UiText::new(fonts.ability().clone(), "".to_string(), [0.0; 4], 25.0));

            ui_transforms.insert(entity, ui_transform);
        }

        for (mut map_notification, mut text) in (&mut map_notifications, &mut texts).join() {
            if Principal::is_root_engaged(&parents, &principals, map_notification.root_ent) != Some(true) {
                if let Some(root) = map_roots.get(map_notification.root_ent) {
                    let pre: String = {
                        if let Some(name) = MapPoint::name_of(&points, &names, root.point_idx) {
                            name + ": "
                        } else {
                            String::new()
                        }
                    };
                    if root.point_idx == current_state.max_point {
                        match current_state.max_stage {
                            MapStage::Combat => {
                                text.text = pre + "Right Arrow Key to Enter Combat";
                                text.color = [1.0, 0.0, 0.0, 1.0];
                            },
                            MapStage::PreDialogue => {
                                text.text = pre + "Right Arrow Key to Enter Dialogue";
                                text.color = [1.0, 1.0, 0.0, 1.0];
                            },
                            MapStage::PostDialogue => {
                                text.text = pre + "Right Arrow Key to Enter Dialogue";
                                text.color = [1.0, 1.0, 0.0, 1.0];
                            },
                            MapStage::Complete => {
                                text.text = pre + "Complete - Use Right and Left Arrow Keys to Move";
                                text.color = [0.0, 1.0, 0.0, 1.0];
                            },
                        }
                    } else {
                        text.text = pre + "Complete - Use Right and Left Arrow Keys to Move";;
                        text.color = [0.0, 1.0, 0.0, 1.0];
                    }
                }
            } else {
                text.text = "".to_string();
                text.color = [0.0; 4];
            }
        }
    }
}
