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

use crate::game::combat::Team;
use crate::game::ui::font::GameFonts;
use crate::game::ui::hud::UiBase;

#[derive(Debug, Copy, Clone, PartialEq, Component)]
pub struct UiSelectAllButton {
    pub team_ent: Entity,
    pub ability_ent: Entity,
    pub hover: bool,
    pub base_color: [f32; 4],
    pub hover_color: [f32; 4],
    pub x_position: f32,
}

impl UiSelectAllButton {
    pub fn new(ability_ent: Entity, team_ent: Entity, base_color: [f32; 4], hover_color: [f32; 4]) -> Self {
        Self {
            ability_ent,
            team_ent,
            hover: false,
            hover_color,
            base_color,
            x_position: 0.0
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct SelectAllClickedEvent {
    pub team_ent: Entity,
    pub ability_ent: Entity,
}

#[derive(Debug, SystemDesc, new)]
#[system_desc(name(SelectAllUiSystemDesc))]
pub struct SelectAllUiSystem {
    #[system_desc(event_channel_reader)]
    ui_reader: ReaderId<UiEvent>,
}

impl<'s> System<'s> for SelectAllUiSystem {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, Parent>,
        ReadStorage<'s, Team>,
        WriteStorage<'s, UiImage>,
        WriteStorage<'s, UiText>,
        WriteStorage<'s, UiSelectAllButton>,
        WriteStorage<'s, UiTransform>,
        WriteStorage<'s, Interactable>,
        ReadStorage<'s, UiBase>,
        Write<'s, EventChannel<SelectAllClickedEvent>>,
        Read<'s, EventChannel<UiEvent>>,
        ReadExpect<'s, GameFonts>,
    );

    fn run(&mut self, (entities, parents, teams, mut images, mut texts, mut buttons, mut ui_transforms, mut interactables, ui_bases, mut button_clicked_events, ui_events, fonts): Self::SystemData) {
        // Insertion of crosshair ui elements.
        let transform_mask = ui_transforms.mask().clone();
        let image_mask = images.mask().clone();
        for (entity, button, _) in (&entities, &buttons, !(image_mask & transform_mask)).join() {
            images.insert(entity, UiImage::SolidColor(button.base_color));

            let y: f32 = {
                if let Some(team) = teams.get(button.team_ent) {
                    match team {
                        Team::Enemy => 150.0,
                        Team::Friendly => 60.0,
                    }
                } else {
                    60.0
                }
            };

            let id: String = String::from("select_all_targets_button:") + &entity.id().to_string();
            let mut ui_transform = UiTransform::new(
                id,
                Anchor::BottomMiddle,
                Anchor::Middle,
                button.x_position, 60.0, 0.0,
                200.0, 60.0,
            );

            texts.insert(entity, UiText::new(fonts.ability().clone(), "Target All".to_string(), [1.0; 4], 25.0));

            ui_transforms.insert(entity, ui_transform);

            interactables.insert(entity, Interactable::default());
        }

        for ui_event in ui_events.read(&mut self.ui_reader) {
            if let Some(button) = buttons.get_mut(ui_event.target) {
                match ui_event.event_type {
                    UiEventType::Click => {
                        // Clicked on an ability.
                        let event: SelectAllClickedEvent = SelectAllClickedEvent {
                            ability_ent: button.ability_ent,
                            team_ent: button.team_ent,
                        };
                        button_clicked_events.single_write(event);
                    },
                    UiEventType::HoverStart => {
                        button.hover = true;
                        if let Some(image) = images.get_mut(ui_event.target) {
                            *image = UiImage::SolidColor(button.hover_color);
                        }
                    },
                    UiEventType::HoverStop => {
                        button.hover = false;
                        if let Some(image) = images.get_mut(ui_event.target) {
                            *image = UiImage::SolidColor(button.base_color);
                        }
                    }
                    _ => {},
                }
            }
        }
    }
}
