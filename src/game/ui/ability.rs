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

use crate::game::character::Character;
use crate::game::combat::ability::{
    Ability,
    AbilityCharge,
};
use crate::game::combat::ability::charge::ChargeAbility;
use crate::game::combat::process::Principal;
use crate::game::ui::crosshair::UiCrosshair;
use crate::game::ui::font::GameFonts;
use crate::game::ui::hud::{UiBase, UiCharacterBase};
use crate::game::ui::status::STATUS_WIDTH;
use crate::game::ui::UiDisengageEvent;
use crate::game::ui::description::UiDescription;

pub const ABILITY_WIDTH: f32 = 250.0;
pub const ABILITY_COUNT: i32 = 8;
pub const ABILITY_HEIGHT: f32 = 60.0;
pub const ABILITY_PANEL_HEIGHT: f32 = ABILITY_HEIGHT * (ABILITY_COUNT as f32);
pub const ABILITY_PANEL_OFFSET: f32 = STATUS_WIDTH;

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, Component)]
pub struct UiAbilityTargetTag;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Component)]
pub struct UiAbilityPanel {
    character_ent: Entity,
    should_load: bool,
}

impl UiAbilityPanel {
    pub fn new(character_ent: Entity) -> Self {
        Self {
            character_ent,
            should_load: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Component)]
pub struct UiAbility {
    name: String,
    icon: Option<Handle<Texture>>,
    ability_ent: Entity,
    invokable: bool,
    hover: bool,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ShowAbilitiesEvent {
    character_ent: Entity,
}

impl ShowAbilitiesEvent {
    pub fn new(character_ent: Entity) -> Self {
        Self {
            character_ent
        }
    }
}

#[derive(Debug, SystemDesc, new)]
#[system_desc(name(AbilitySelectSystemDesc))]
pub struct AbilitySelectSystem {
    #[system_desc(event_channel_reader)]
    abilities_reader: ReaderId<ShowAbilitiesEvent>,

    #[system_desc(event_channel_reader)]
    ui_reader: ReaderId<UiEvent>,

    #[system_desc(event_channel_reader)]
    disengage_reader: ReaderId<UiDisengageEvent>,

    charge_icon_image: Option<Handle<Texture>>,
}

/// A marker struct that should be added to ability entities to inform the ui system that targeting should begin.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Component)]
pub struct UiAbilitySelectionTag;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct UiAbilitySelection {
    pub character_ent: Option<Entity>,
}

impl UiAbilitySelection {
    pub fn none() -> Self {
        Self {
            character_ent: None,
        }
    }

    pub fn with_character(character_ent: Entity) -> Self {
        Self {
            character_ent: Some(character_ent),
        }
    }
}

#[derive(Debug, Copy, Clone, new, PartialEq)]
pub struct UiAbilitySelectEvent {
    pub(crate) ability_ent: Entity,
}

impl<'s> System<'s> for AbilitySelectSystem {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, Principal>,
        WriteStorage<'s, Parent>,
        ReadStorage<'s, Ability>,
        ReadStorage<'s, Character>,
        WriteStorage<'s, UiBase>,
        WriteStorage<'s, UiAbility>,
        WriteStorage<'s, UiAbilitySelectionTag>,
        WriteStorage<'s, UiAbilityPanel>,
        WriteStorage<'s, UiTransform>,
        WriteStorage<'s, Interactable>,
        WriteStorage<'s, UiImage>,
        WriteStorage<'s, UiText>,
        WriteStorage<'s, UiDescription>,
        ReadExpect<'s, ParentHierarchy>,
        Read<'s, EventChannel<ShowAbilitiesEvent>>,
        Read<'s, EventChannel<UiEvent>>,
        Read<'s, EventChannel<UiDisengageEvent>>,
        Write<'s, EventChannel<UiAbilitySelectEvent>>,
        ReadExpect<'s, ScreenDimensions>,
        ReadExpect<'s, GameFonts>,
    );

    fn setup(&mut self, world: &mut World) {
        if !world.has_value::<AssetStorage::<Texture>>() {
            world.insert(AssetStorage::<Texture>::new());
        }
        let loader = world.read_resource::<Loader>();
        self.charge_icon_image = Some(loader.load(
            "ui/charge_icon.png",
            ImageFormat::default(),
            (),
            &world.read_resource::<AssetStorage<Texture>>(),
        ));
    }

    fn run(&mut self, (entities, principals, mut parents, abilities, characters, ui_bases, mut ui_abilities, mut ui_targets, mut ability_panels, mut ui_transforms, mut interactable, mut ui_images, mut ui_texts, mut descriptions, hierarchy, mut evt_show_ability, ui_events, disengage_events, mut ability_select_events, dims, fonts): Self::SystemData) {
        for event in evt_show_ability.read(&mut self.abilities_reader) {
            // Clear existing ability ui.

            for (entity, _) in (&entities, &ability_panels).join() {
                entities.delete(entity);
            }

            if let Some(character) = characters.get(event.character_ent) {
                if let Some(ui_ent) = ui_bases.get(event.character_ent).copied() {
                    let ui_ent = ui_ent.entity();
                    if let Some(current_transform) = ui_transforms.get(ui_ent) {
                        let panel_ent: Entity = entities.create();
                        let panel: UiAbilityPanel = UiAbilityPanel::new(event.character_ent);
                        ability_panels.insert(panel_ent, panel);
                        let panel_img: UiImage = UiImage::SolidColor([0.005, 0.005, 0.006, 0.9]);
                        ui_images.insert(panel_ent, panel_img);
                        //parents.insert(panel_ent, Parent { entity: ui_ent });
                        let id: String = String::from("ability_panel:") + &panel_ent.id().to_string();
                        let transform: UiTransform = UiTransform::new(
                            id,
                            Anchor::Middle,
                            Anchor::Middle,
                            ABILITY_PANEL_OFFSET, 0.0, 1.0,
                            ABILITY_WIDTH, ABILITY_PANEL_HEIGHT,
                        );
                        ui_transforms.insert(panel_ent, transform);
                        let mut offset_y: f32 = 0.0;

                        // Add top text.
                        let title_ent = entities.create();
                        let title_text: UiText = UiText::new(fonts.ability().clone(), character.name().to_string(), [1.0; 4], 25.0);
                        ui_texts.insert(title_ent, title_text);
                        let id: String = String::from("ability_panel_title:") + &title_ent.id().to_string();
                        let transform: UiTransform = UiTransform::new(
                            id,
                            Anchor::TopLeft,
                            Anchor::TopLeft,
                            0.0, offset_y, 1.0,
                            ABILITY_WIDTH, ABILITY_HEIGHT,
                        );
                        ui_transforms.insert(title_ent, transform);
                        parents.insert(title_ent, Parent { entity: panel_ent });
                        offset_y -= ABILITY_HEIGHT;

                        // Order the ability entities.
                        let mut ordered: Vec<(Entity, f32)> = Vec::new();

                        for (ability_ent, ability, _) in (&entities, &abilities, hierarchy.all_children(event.character_ent)).join() {
                            let new_charge: f32 = ability.data.charge.rated_charge();
                            if ordered.is_empty() {
                                ordered.push((ability_ent, new_charge));
                            } else {
                                let mut added: bool = false;
                                for (i, (ent, charge)) in ordered.clone().iter().enumerate() {
                                    if new_charge < *charge {
                                        ordered.insert(i, (ability_ent, new_charge));
                                        added = true;
                                        break;
                                    }
                                }
                                if !added {
                                    ordered.push((ability_ent, new_charge));
                                }
                            }
                        }

                        for (ability_ent, _) in ordered {
                            if let Some(ability) = abilities.get(ability_ent) {
                                let locked = !ability.can_perform();

                                let invokable: bool = {
                                    if Character::can_take_turn(&characters, event.character_ent, ability.data.charge.rated_charge()) && !locked {
                                        true
                                    } else {
                                        false
                                    }
                                };

                                let ui_ability_ent: Entity = entities.create();
                                let ui_ability: UiAbility = UiAbility {
                                    name: ability.data.name.to_owned(),
                                    ability_ent,
                                    icon: None,
                                    invokable,
                                    hover: false,
                                };
                                ui_abilities.insert(ui_ability_ent, ui_ability);

                                descriptions.insert(ui_ability_ent, UiDescription::new(ability.data.name.to_string(), ability.data.desc.to_string()));

                                // Change color depending on whether the ability can be performed.
                                let text_color: [f32; 4] = {
                                    if ability.data.charge.rated_charge() < 0.0 {
                                        // Charge ability.
                                        [0.0, 1.0, 1.0, 1.0]
                                    } else if invokable {
                                        [1.0; 4]
                                    } else if locked {
                                        [0.2, 0.2, 0.2, 1.0]
                                    } else {
                                        [0.4, 0.1, 0.1, 1.0]
                                    }
                                };

                                let mut ability_text: UiText = UiText::new(fonts.ability().clone(), "  ".to_string() + ability.data.name, text_color, 15.0);
                                ability_text.align = Anchor::MiddleLeft;
                                ui_texts.insert(ui_ability_ent, ability_text);
                                let ability_img: UiImage = UiImage::SolidColor([0.0, 0.0, 0.0, 1.0]);
                                ui_images.insert(ui_ability_ent, ability_img);

                                let id: String = String::from("ability:") + &ability_ent.id().to_string();
                                let transform: UiTransform = UiTransform::new(
                                    id,
                                    Anchor::TopLeft,
                                    Anchor::TopLeft,
                                    0.0, offset_y, 1.0,
                                    ABILITY_WIDTH, ABILITY_HEIGHT,
                                );
                                ui_transforms.insert(ui_ability_ent, transform);

                                // Charge Icon
                                let charge_ent: Entity = entities.create();
                                let charge_icon_img: UiImage = UiImage::Texture(self.charge_icon_image.clone().unwrap());
                                ui_images.insert(charge_ent, charge_icon_img);
                                let id: String = String::from("ability_charge_icon:") + &charge_ent.id().to_string();
                                let mut transform: UiTransform = UiTransform::new(
                                    id,
                                    Anchor::Middle,
                                    Anchor::Middle,
                                    ABILITY_WIDTH * 0.2, 0.0, 1.5,
                                    ABILITY_HEIGHT * 0.4, ABILITY_HEIGHT * 0.4,
                                );
                                transform.opaque = false;
                                ui_transforms.insert(charge_ent, transform);
                                parents.insert(charge_ent, Parent { entity: ui_ability_ent });

                                // Charge Text
                                let charge_text_data: String = {
                                    match ability.data.charge {
                                        AbilityCharge::Static(val) => format!("{:.0}", val.abs()),
                                        AbilityCharge::Range(low, high) => format!("{:.0}-{:.0}", low, high),
                                    }
                                };
                                let charge_text_ent: Entity = entities.create();
                                let mut charge_text = UiText::new(fonts.ability().clone(), charge_text_data, text_color, 15.0);
                                charge_text.align = Anchor::MiddleLeft;
                                ui_texts.insert(charge_text_ent, charge_text);
                                let id: String = String::from("ability_charge_text:") + &charge_text_ent.id().to_string();
                                let mut transform: UiTransform = UiTransform::new(
                                    id,
                                    Anchor::MiddleLeft,
                                    Anchor::MiddleLeft,
                                    25.0, 0.0, 1.5,
                                    60.0, ABILITY_HEIGHT * 0.4,
                                );
                                transform.opaque = false;
                                ui_transforms.insert(charge_text_ent, transform);
                                parents.insert(charge_text_ent, Parent { entity: charge_ent });

                                // Add main ability entity.
                                if invokable {
                                    interactable.insert(ui_ability_ent, Interactable::default());
                                }
                                parents.insert(ui_ability_ent, Parent { entity: panel_ent });

                                offset_y -= ABILITY_HEIGHT;
                            }
                        }
                    }
                }
            }
        }

        for (entity, ability_panel) in (&entities, &ability_panels).join() {
            if let Some(true) = Principal::is_root_engaged(&parents, &principals, ability_panel.character_ent) {
                entities.delete(entity);
            }
        }

        for ui_event in ui_events.read(&mut self.ui_reader) {
            if ui_event.event_type == UiEventType::Click {
                if let Some(ui_ability) = ui_abilities.get(ui_event.target) {
                    // Initiate ability target selection.
                    ability_select_events.single_write(UiAbilitySelectEvent::new(ui_ability.ability_ent));
                }
            } else if ui_event.event_type == UiEventType::HoverStart {
                if let Some(ui_ability) = ui_abilities.get_mut(ui_event.target) {
                    ui_ability.hover = true;
                    if let Some(ui_image) = ui_images.get_mut(ui_event.target) {
                        *ui_image = UiImage::SolidColor([0.2, 0.05, 0.05, 1.0]);
                    }

                }
            } else if ui_event.event_type == UiEventType::HoverStop {
                if let Some(ui_ability) = ui_abilities.get_mut(ui_event.target) {
                    ui_ability.hover = false;
                    if let Some(ui_image) = ui_images.get_mut(ui_event.target) {
                        *ui_image = UiImage::SolidColor([0.0, 0.0, 0.0, 1.0]);
                    }
                }
            }
        }

        for disengage_event in disengage_events.read(&mut self.disengage_reader) {
            if *disengage_event != UiDisengageEvent::TargetChanged {
                for (entity, _) in (&entities, &ability_panels).join() {
                    entities.delete(entity);
                }
            }
        }
    }
}