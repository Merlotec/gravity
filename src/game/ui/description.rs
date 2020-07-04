use amethyst::{
    assets::{
        AssetStorage,
        Handle,
        Loader,
    },
    core::{
        Parent,
        ParentHierarchy,
        HiddenPropagate,
    },
    ecs::prelude::*,
    prelude::SystemDesc,
    renderer::ImageFormat,
    renderer::{
        Texture
    },
    input::{
        InputHandler,
        StringBindings,
    },
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
        LineMode,
    },
    window::ScreenDimensions,
    winit::VirtualKeyCode,
};

use crate::core::get_root;
use crate::game::character::{Character, CharacterId, CharacterStore};
use crate::game::combat::ability::Ability;
use crate::game::ui::crosshair::UiCrosshair;
use crate::game::ui::font::GameFonts;
use crate::game::ui::hud::{UiBase, UiCharacterBase};
use crate::game::ui::UiDisengageEvent;

pub const DESCRIPTION_PANEL_WIDTH: f32 = 250.0;
pub const DESCRIPTION_PANEL_COUNT: i32 = 8;
pub const DESCRIPTION_PANEL_HEIGHT: f32 = 400.0;
pub const DESCRIPTION_PANEL_OFFSET: f32 = 200.0;

#[derive(Debug, Clone, PartialEq, Component)]
pub struct UiDescription {
    pub title: String,
    pub description: String,
    pub panel_ent: Option<Entity>,
    pub hover: bool,
}

impl UiDescription {
    pub fn new(title: String, description: String) -> Self {
        Self {
            title,
            description,
            panel_ent: None,
            hover: false,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, new, Component)]
pub struct UiDescriptionPanel {
    pub owner: Entity,
}

#[derive(Debug, Copy, Clone, new, PartialEq)]
pub struct ShowDescriptionPanelEvent {
    pub owner: Entity,
}

#[derive(Debug, SystemDesc, new)]
#[system_desc(name(DescriptionPanelSystemDesc))]
pub struct DescriptionPanelSystem {

    #[system_desc(event_channel_reader)]
    ui_reader: ReaderId<UiEvent>,

    #[system_desc(event_channel_reader)]
    disengage_reader: ReaderId<UiDisengageEvent>,
}

impl<'s> System<'s> for DescriptionPanelSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Parent>,
        ReadStorage<'s, Character>,
        WriteStorage<'s, UiBase>,
        WriteStorage<'s, UiDescription>,
        WriteStorage<'s, UiDescriptionPanel>,
        WriteStorage<'s, UiTransform>,
        WriteStorage<'s, Interactable>,
        WriteStorage<'s, UiImage>,
        WriteStorage<'s, UiText>,
        WriteStorage<'s, HiddenPropagate>,
        ReadExpect<'s, ParentHierarchy>,
        Read<'s, EventChannel<UiEvent>>,
        ReadExpect<'s, ScreenDimensions>,
        ReadExpect<'s, GameFonts>,
        Read<'s, InputHandler<StringBindings>>,
    );

    fn run(&mut self, (entities, mut parents, characters, ui_bases, mut descriptions, mut description_panels, mut ui_transforms, mut interactable, mut ui_images, mut ui_texts, mut hiddens, hierarchy, ui_events, dims, fonts, input_handler): Self::SystemData) {

        for event in ui_events.read(&mut self.ui_reader) {
            if event.event_type == UiEventType::HoverStart {
                if let Some(description) = descriptions.get_mut(event.target) {
                    description.hover = true;
                }
            } else if event.event_type == UiEventType::HoverStop {
                if let Some(description) = descriptions.get_mut(event.target) {
                    description.hover = false;
                }
            }
        }

        let enable = input_handler.key_is_down(VirtualKeyCode::LShift);

        for (entity, mut description) in (&entities, &mut descriptions).join() {
            if description.panel_ent.is_none() {
                let panel_ent: Entity = entities.create();
                let panel: UiDescriptionPanel = UiDescriptionPanel::new(entity);
                description_panels.insert(panel_ent, panel);
                let panel_img: UiImage = UiImage::SolidColor([0.005, 0.005, 0.006, 1.0]);
                ui_images.insert(panel_ent, panel_img);
                //parents.insert(panel_ent, Parent { entity: ui_ent });
                let id: String = String::from("description:") + &panel_ent.id().to_string();
                let mut transform: UiTransform = UiTransform::new(
                    id,
                    Anchor::Middle,
                    Anchor::Middle,
                    DESCRIPTION_PANEL_OFFSET, 0.0, 10.0,
                    DESCRIPTION_PANEL_WIDTH, DESCRIPTION_PANEL_HEIGHT,
                );
                transform.opaque = false;
                ui_transforms.insert(panel_ent, transform);

                // Add top text.
                let title_ent = entities.create();
                let title_text: UiText = UiText::new(fonts.ability().clone(), description.title.clone(), [1.0; 4], 25.0);
                ui_texts.insert(title_ent, title_text);
                let id: String = String::from("description_title:") + &panel_ent.id().to_string();
                let mut transform: UiTransform = UiTransform::new(
                    id,
                    Anchor::TopLeft,
                    Anchor::TopLeft,
                    10.0, -10.0, 2.0,
                    DESCRIPTION_PANEL_WIDTH - 20.0, 50.0,
                );
                transform.opaque = false;
                ui_transforms.insert(title_ent, transform);
                parents.insert(title_ent, Parent { entity: panel_ent });

                // Add description text.
                let desc_ent = entities.create();
                let mut desc_text: UiText = UiText::new(fonts.ability().clone(), description.description.clone(), [1.0; 4], 15.0);
                desc_text.align = Anchor::TopLeft;
                desc_text.line_mode = LineMode::Wrap;
                ui_texts.insert(desc_ent, desc_text);
                let id: String = String::from("description_desc:") + &panel_ent.id().to_string();
                let mut transform: UiTransform = UiTransform::new(
                    id,
                    Anchor::TopLeft,
                    Anchor::TopLeft,
                    10.0, -60.0, 2.0,
                    DESCRIPTION_PANEL_WIDTH - 20.0, DESCRIPTION_PANEL_HEIGHT - 60.0,
                );
                transform.opaque = false;
                ui_transforms.insert(desc_ent, transform);
                parents.insert(desc_ent, Parent { entity: panel_ent });
                hiddens.insert(panel_ent, HiddenPropagate::new());
                parents.insert(panel_ent, Parent { entity });

                description.panel_ent = Some(panel_ent);
            }

            if let Some(panel_ent) = description.panel_ent {
                if description.hover && enable {
                    hiddens.remove(panel_ent);
                    if let Some(transform) = ui_transforms.get_mut(panel_ent) {
                        transform.local_y = 0.0;
                    }
                } else {
                    hiddens.insert(panel_ent, HiddenPropagate::new());
                    if let Some(transform) = ui_transforms.get_mut(panel_ent) {

                        transform.local_y = 10000.0;
                    }
                }
            }
        }
    }
}