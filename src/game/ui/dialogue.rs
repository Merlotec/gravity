
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
    input::{
        InputEvent,
        StringBindings,
    },
    winit::VirtualKeyCode,
    winit::MouseButton,
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
        LineMode,
    },
    window::ScreenDimensions,
};

use crate::game::combat::{CombatRoot, CombatState, Team, Rank};
use crate::game::ui::font::GameFonts;
use crate::game::ui::hud::UiBase;
use crate::game::combat::process::Principal;
use std::any::TypeId;
use crate::game::ui::UiDisengageEvent;

#[derive(Debug, Clone, PartialEq)]
pub struct Dialogue {
    pub segments: Vec<DialogueSegment>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DialogueSegment {
    pub header: Option<DialogueText>,
    pub body: DialogueText,
    pub icon_texture: Option<Handle<Texture>>,
}

impl DialogueSegment {
    pub fn new(header: &str, body: &str) -> Self {
        Self {
            header: Some(DialogueText {
                color: [1.0; 4],
                text: header.to_string(),
            }),
            body: DialogueText {
                color: [1.0; 4],
                text: body.to_string(),
            },
            icon_texture: None,
        }
    }

    pub fn with_icon(header: &str, body: &str, icon_texture: Handle<Texture>) -> Self {
        Self {
            header: Some(DialogueText {
                color: [1.0; 4],
                text: header.to_string(),
            }),
            body: DialogueText {
                color: [1.0; 4],
                text: body.to_string(),
            },
            icon_texture: Some(icon_texture),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DialogueText {
    pub  text: String,
    pub color: [f32; 4],
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShowDialogueDisplayEvent {
    pub owner: Option<Entity>,
    pub dialogue: Dialogue,
    pub start_idx: usize,
    pub principal: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NavigateDialogueEvent {
    pub dialogue_ent: Entity,
    pub index: usize,
}

#[derive(Debug, Clone, PartialEq, Component)]
pub struct UiDialogueDisplay {
    pub owner: Option<Entity>,
    pub dialogue: Dialogue,
    pub current_idx: usize,
    pub header_ent: Entity,
    pub body_ent: Entity,
    pub icon_ent: Entity,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DialogueCompletedEvent {
    pub owner: Option<Entity>,
}

#[derive(Debug, SystemDesc, new)]
#[system_desc(name(DialogueUiSystemDesc))]
pub struct DialogueUiSystem {
    #[system_desc(event_channel_reader)]
    show_dialogue_event_reader: ReaderId<ShowDialogueDisplayEvent>,

    #[system_desc(event_channel_reader)]
    disengage_event_reader: ReaderId<UiDisengageEvent>,

    #[system_desc(event_channel_reader)]
    navigate_dialogue_event_reader: ReaderId<NavigateDialogueEvent>,

    #[system_desc(event_channel_reader)]
    input_event_reader: ReaderId<InputEvent<StringBindings>>,
}

impl<'s> System<'s> for DialogueUiSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Principal>,
        WriteStorage<'s, Parent>,
        ReadStorage<'s, Team>,
        WriteStorage<'s, UiText>,
        WriteStorage<'s, UiImage>,
        WriteStorage<'s, UiDialogueDisplay>,
        WriteStorage<'s, UiTransform>,
        WriteStorage<'s, CombatRoot>,
        ReadExpect<'s, GameFonts>,
        Read<'s, EventChannel<ShowDialogueDisplayEvent>>,
        Write<'s, EventChannel<NavigateDialogueEvent>>,
        Write<'s, EventChannel<InputEvent<StringBindings>>>,
        Write<'s, EventChannel<DialogueCompletedEvent>>,
        Write<'s, EventChannel<UiDisengageEvent>>,
    );

    fn run(&mut self, (entities, mut principals, mut parents, teams, mut texts, mut images, mut dialogue_displays, mut ui_transforms, combat_roots, fonts, show_dialogue_events, mut navigate_dialogue_events, input_events, mut dialogue_completed_events, disengage_events): Self::SystemData) {
        for event in show_dialogue_events.read(&mut self.show_dialogue_event_reader) {
            if let Some(segment) = event.dialogue.segments.get(event.start_idx) {
                let dialogue_ent: Entity = entities.create();
                if event.principal {
                    if Principal::try_root_engage(&parents, &mut principals, event.owner.expect("No owner specified for principal dialogue!"), TypeId::of::<Self>()) != Some(true) {
                        entities.delete(dialogue_ent);
                        continue;
                    }
                }

                let id: String = String::from("dialogue:") + &dialogue_ent.id().to_string();
                let mut ui_transform = UiTransform::new(
                    id,
                    Anchor::BottomMiddle,
                    Anchor::TopMiddle,
                    0.0, 200.0, 0.0,
                    10000.0, 200.0,
                );
                ui_transforms.insert(dialogue_ent, ui_transform);
                images.insert(dialogue_ent, UiImage::SolidColor([0.005, 0.005, 0.006, 0.9]));

                let icon_ent: Entity = entities.create();
                if let Some(icon_texture) = segment.icon_texture.clone() {
                    images.insert(icon_ent, UiImage::Texture(icon_texture.clone()));
                } else {
                    images.insert(icon_ent, UiImage::SolidColor([0.0; 4]));
                }
                let id: String = String::from("dialogue_icon:") + &icon_ent.id().to_string();
                let mut ui_transform = UiTransform::new(
                    id,
                    Anchor::Middle,
                    Anchor::Middle,
                    -400.0, 0.0, 1.0,
                    100.0, 100.0,
                );
                ui_transforms.insert(icon_ent, ui_transform);
                parents.insert(icon_ent, Parent { entity: dialogue_ent });

                let header_ent: Entity = entities.create();
                let id: String = String::from("dialogue_header:") + &header_ent.id().to_string();
                let mut ui_transform = UiTransform::new(
                    id,
                    Anchor::TopMiddle,
                    Anchor::TopMiddle,
                    0.0, -10.0, 1.0,
                    600.0, 50.0,
                );
                ui_transforms.insert(header_ent, ui_transform);
                let (header_text, header_color) : (String, [f32; 4]) = match &segment.header {
                    Some(header) => (header.text.clone(), header.color),
                    None => (String::new(), [0.0; 4]),
                };
                let mut text = UiText::new(fonts.ability().clone(), header_text, header_color, 25.0);
                text.line_mode = LineMode::Single;
                text.align = Anchor::Middle;
                texts.insert(header_ent, text);
                parents.insert(header_ent, Parent { entity: dialogue_ent });

                let body_ent: Entity = entities.create();
                let id: String = String::from("dialogue_body:") + &body_ent.id().to_string();
                let mut ui_transform = UiTransform::new(
                    id,
                    Anchor::TopMiddle,
                    Anchor::TopMiddle,
                    0.0, -70.0, 1.0,
                    600.0, 300.0,
                );
                ui_transforms.insert(body_ent, ui_transform);
                let mut text = UiText::new(fonts.ability().clone(), segment.body.text.clone(), segment.body.color, 17.0);
                text.line_mode = LineMode::Wrap;
                text.align = Anchor::TopLeft;
                texts.insert(body_ent, text);
                parents.insert(body_ent, Parent { entity: dialogue_ent });

                dialogue_displays.insert(dialogue_ent, UiDialogueDisplay {
                    dialogue: event.dialogue.clone(),
                    owner: event.owner,
                    current_idx: event.start_idx,
                    header_ent,
                    body_ent,
                    icon_ent,
                });
            }
        }

        for event in input_events.read(&mut self.input_event_reader) {
            let dir: i32 = match event {
                InputEvent::KeyPressed { key_code: VirtualKeyCode::Return, .. } => {
                    1
                },
                InputEvent::KeyPressed { key_code: VirtualKeyCode::Space, .. } => {
                    1
                },
                InputEvent::KeyPressed { key_code: VirtualKeyCode::Right, .. } => {
                    1
                },
                InputEvent::KeyPressed { key_code: VirtualKeyCode::Right, .. } => {
                    -1
                },
                InputEvent::MouseButtonPressed(MouseButton::Left) => {
                    1
                },
                _ => 0,
            };

            match dir {
                -1 => {
                    for (dialogue_ent, dialogue_display) in (&entities, &dialogue_displays).join() {
                        if dialogue_display.current_idx > 0 {
                            navigate_dialogue_events.single_write(
                                NavigateDialogueEvent {
                                    dialogue_ent,
                                    index: dialogue_display.current_idx - 1,
                                }
                            );
                        }
                    }
                },
                1 => {
                    for (dialogue_ent, dialogue_display) in (&entities, &dialogue_displays).join() {
                        navigate_dialogue_events.single_write(
                            NavigateDialogueEvent {
                                dialogue_ent,
                                index: dialogue_display.current_idx + 1,
                            }
                        );
                    }
                },
                _ => {},
            }
        }

        for event in navigate_dialogue_events.read(&mut self.navigate_dialogue_event_reader) {
            if let Some(dialogue_display) = dialogue_displays.get_mut(event.dialogue_ent) {
                if let Some(segment) = dialogue_display.dialogue.segments.get(event.index) {
                    if let Some(text) = texts.get_mut(dialogue_display.header_ent) {
                        let (header_text, header_color) : (String, [f32; 4]) = match &segment.header {
                            Some(header) => (header.text.clone(), header.color),
                            None => (String::new(), [0.0; 4]),
                        };
                        text.text = header_text;
                        text.color = header_color;
                    }
                    if let Some(text) = texts.get_mut(dialogue_display.body_ent) {
                        text.text = segment.body.text.clone();
                        text.color = segment.body.color;
                    }

                    if let Some(icon) = images.get_mut(dialogue_display.icon_ent) {
                        if let Some(icon_texture) = segment.icon_texture.clone() {
                            *icon = UiImage::Texture(icon_texture.clone());
                        } else {
                            *icon = UiImage::SolidColor([0.0; 4]);
                        }
                    }

                    dialogue_display.current_idx = event.index;
                } else {
                    dialogue_completed_events.single_write(
                        DialogueCompletedEvent {
                            owner: dialogue_display.owner,
                        }
                    );
                    if let Some(owner) = dialogue_display.owner {
                        Principal::try_root_disengage(
                            &parents,
                            &mut principals,
                            owner,
                            TypeId::of::<Self>(),
                        );
                    }

                    entities.delete(event.dialogue_ent);

                }
            }
        }

        for disengage_event in disengage_events.read(&mut self.disengage_event_reader) {
            if *disengage_event == UiDisengageEvent::Cancel {
                for (entity, dialogue_display) in (&entities, &dialogue_displays).join() {
                    dialogue_completed_events.single_write(
                        DialogueCompletedEvent {
                            owner: dialogue_display.owner,
                        }
                    );
                    if let Some(owner) = dialogue_display.owner {
                        Principal::try_root_disengage(
                            &parents,
                            &mut principals,
                            owner,
                            TypeId::of::<Self>(),
                        );
                    }

                    entities.delete(entity);
                }
            }
        }
    }
}