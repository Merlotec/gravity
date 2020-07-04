use amethyst::{
    assets::{
        AssetStorage,
        Handle,
        Loader,
    },
    core::{
        Parent,
        ParentHierarchy,
    },
    ecs::prelude::*,
    prelude::SystemDesc,
    renderer::ImageFormat,
    renderer::Texture,
    shrev::{
        EventChannel,
        ReaderId,
    },
    ui::{
        Anchor,
        Interactable,
        UiEvent,
        UiEventType,
        UiImage,
        UiText,
        UiTransform,
    },
    window::ScreenDimensions,
};

use crate::core::get_root;
use crate::game::character::{Character, CharacterId, CharacterStore};
use crate::game::combat::ability::Ability;
use crate::game::ui::crosshair::UiCrosshair;
use crate::game::ui::font::GameFonts;
use crate::game::ui::hud::{UiBase, UiCharacterBase};
use crate::game::ui::UiDisengageEvent;
use crate::game::combat::Team;

pub const CHARACTER_SELECT_WIDTH: f32 = 250.0;
pub const CHARACTER_SELECT_COUNT: i32 = 8;
pub const CHARACTER_SELECT_HEIGHT: f32 = 60.0;
pub const CHARACTER_SELECT_PANEL_HEIGHT: f32 = CHARACTER_SELECT_HEIGHT * (CHARACTER_SELECT_COUNT as f32);
pub const CHARACTER_SELECT_PANEL_OFFSET: f32 = 200.0;

#[derive(Debug, Copy, Clone, Eq, PartialEq, new, Component)]
pub struct UiCharacterSelectPanel {
    pub owner: Entity,
}

#[derive(Debug, Clone, PartialEq, Component)]
pub struct UiCharacterIcon {
    name: String,
    id: CharacterId,
    owner: Entity,
}

#[derive(Debug, Copy, Clone, new, PartialEq)]
pub struct SelectCharacterEvent {
    owner: Entity,
}

#[derive(Debug, Copy, Clone, new, PartialEq)]
pub struct CharacterSelectedEvent {
    pub(crate) id: Option<CharacterId>,
    pub(crate) owner: Entity,
}

#[derive(Debug, SystemDesc, new)]
#[system_desc(name(CharacterSelectSystemDesc))]
pub struct CharacterSelectSystem {
    #[system_desc(event_channel_reader)]
    abilities_reader: ReaderId<SelectCharacterEvent>,

    #[system_desc(event_channel_reader)]
    ui_reader: ReaderId<UiEvent>,

    #[system_desc(event_channel_reader)]
    disengage_reader: ReaderId<UiDisengageEvent>,
}

impl<'s> System<'s> for CharacterSelectSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Parent>,
        ReadStorage<'s, Character>,
        ReadStorage<'s, Team>,
        WriteStorage<'s, UiBase>,
        WriteStorage<'s, UiCharacterIcon>,
        WriteStorage<'s, UiCharacterSelectPanel>,
        WriteStorage<'s, UiTransform>,
        WriteStorage<'s, Interactable>,
        WriteStorage<'s, UiImage>,
        WriteStorage<'s, UiText>,
        ReadExpect<'s, ParentHierarchy>,
        Write<'s, EventChannel<SelectCharacterEvent>>,
        Write<'s, EventChannel<CharacterSelectedEvent>>,
        Read<'s, EventChannel<UiEvent>>,
        Read<'s, EventChannel<UiDisengageEvent>>,
        ReadExpect<'s, ScreenDimensions>,
        ReadExpect<'s, GameFonts>,
        Read<'s, CharacterStore>,
    );

    fn run(&mut self, (entities, mut parents, characters, teams, ui_bases, mut ui_character_icons, mut character_panels, mut ui_transforms, mut interactable, mut ui_images, mut ui_texts, hierarchy, mut evt_show_character_select, mut evt_character_selected, ui_events, disengage_events, dims, fonts, character_store): Self::SystemData) {
        for event in evt_show_character_select.read(&mut self.abilities_reader) {
            // Clear existing ability ui.
            for (entity, _) in (&entities, &character_panels).join() {
                entities.delete(entity);
            }

            let panel_ent: Entity = entities.create();
            let panel: UiCharacterSelectPanel = UiCharacterSelectPanel::new(event.owner);
            character_panels.insert(panel_ent, panel);
            let panel_img: UiImage = UiImage::SolidColor([0.005, 0.005, 0.006, 0.9]);
            ui_images.insert(panel_ent, panel_img);
            //parents.insert(panel_ent, Parent { entity: ui_ent });
            let id: String = String::from("select_character:") + &panel_ent.id().to_string();
            let transform: UiTransform = UiTransform::new(
                id,
                Anchor::BottomLeft,
                Anchor::Middle,
                -CHARACTER_SELECT_PANEL_OFFSET, 0.0, 10.0,
                CHARACTER_SELECT_WIDTH, CHARACTER_SELECT_PANEL_HEIGHT,
            );
            ui_transforms.insert(panel_ent, transform);
            if let Some((base, _)) = get_root::<UiBase, _, _>(&parents, &ui_bases, event.owner) {
                parents.insert(panel_ent, Parent { entity: base.entity() });
            }
            let mut offset_y: f32 = 0.0;

            // Add top text.
            let title_ent = entities.create();
            let title_text: UiText = UiText::new(fonts.ability().clone(), "Select Character".to_string(), [1.0; 4], 25.0);
            ui_texts.insert(title_ent, title_text);
            let id: String = String::from("select_character_title:") + &panel_ent.id().to_string();
            let transform: UiTransform = UiTransform::new(
                id,
                Anchor::TopLeft,
                Anchor::TopLeft,
                0.0, offset_y, 1.0,
                CHARACTER_SELECT_WIDTH, CHARACTER_SELECT_HEIGHT,
            );
            ui_transforms.insert(title_ent, transform);
            parents.insert(title_ent, Parent { entity: panel_ent });
            offset_y -= CHARACTER_SELECT_HEIGHT;

            if let Some((team, _)) = Team::get_team(&parents, &teams, event.owner) {
                for (id, character_data) in character_store.get_spawnable(team) {
                    let ui_character_icon_ent: Entity = entities.create();
                    let ui_character_icon: UiCharacterIcon = UiCharacterIcon {
                        name: character_data.name.to_owned(),
                        id,
                        owner: event.owner,
                    };
                    ui_character_icons.insert(ui_character_icon_ent, ui_character_icon);

                    let ability_text: UiText = UiText::new(fonts.ability().clone(), character_data.name.to_string(), [1.0; 4], 15.0);
                    ui_texts.insert(ui_character_icon_ent, ability_text);
                    let ability_img: UiImage = UiImage::SolidColor([0.0, 0.0, 0.0, 1.0]);
                    ui_images.insert(ui_character_icon_ent, ability_img);
                    let id: String = String::from("character_id:") + character_data.name;
                    let transform: UiTransform = UiTransform::new(
                        id,
                        Anchor::TopLeft,
                        Anchor::TopLeft,
                        0.0, offset_y, 1.0,
                        CHARACTER_SELECT_WIDTH, CHARACTER_SELECT_HEIGHT,
                    );
                    ui_transforms.insert(ui_character_icon_ent, transform);
                    interactable.insert(ui_character_icon_ent, Interactable::default());
                    parents.insert(ui_character_icon_ent, Parent { entity: panel_ent });

                    offset_y -= CHARACTER_SELECT_HEIGHT;
                }
            }
        }

        for ui_event in ui_events.read(&mut self.ui_reader) {
            if ui_event.event_type == UiEventType::Click {
                if let Some(ui_character_icon) = ui_character_icons.get(ui_event.target) {
                    evt_character_selected.single_write(CharacterSelectedEvent::new(Some(ui_character_icon.id), ui_character_icon.owner));
                    if let Some((_, panel_ent)) = get_root(&parents, &character_panels, ui_event.target) {
                        entities.delete(panel_ent);
                    }
                }
            } else if ui_event.event_type == UiEventType::HoverStart {
                if let Some(ui_character_icon) = ui_character_icons.get(ui_event.target) {
                    //ui_ability.hover = true;
                    if let Some(ui_image) = ui_images.get_mut(ui_event.target) {
                        *ui_image = UiImage::SolidColor([0.2, 0.05, 0.05, 1.0]);
                    }
                }
            } else if ui_event.event_type == UiEventType::HoverStop {
                if let Some(ui_character_icon) = ui_character_icons.get(ui_event.target) {
                    //ui_ability.hover = false;
                    if let Some(ui_image) = ui_images.get_mut(ui_event.target) {
                        *ui_image = UiImage::SolidColor([0.0, 0.0, 0.0, 1.0]);
                    }
                }
            }
        }

        for disengage_event in disengage_events.read(&mut self.disengage_reader) {
            for (entity, character_panel) in (&entities, &character_panels).join() {
                entities.delete(entity);
                evt_character_selected.single_write(CharacterSelectedEvent::new(None, character_panel.owner));
            }
        }
    }
}