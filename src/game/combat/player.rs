use amethyst::{
    core::{
        Parent,
        Transform,
    },
    core::{
        ParentHierarchy,
        SystemDesc,
    },
    ecs::prelude::*,
    input::{
        Button,
        InputEvent,
        InputHandler,
        StringBindings,
    },
    shrev::{EventChannel, ReaderId},
    winit::{
        MouseButton,
        VirtualKeyCode,
    },
};

use crate::core::get_root;
use crate::game::character::Character;
use crate::game::combat::{CombatRoot, CombatState, Team};
use crate::game::combat::ability::{Ability, AbilityInvoke, AbilityPerform};
use crate::game::combat::process::Principal;
use crate::game::combat::spawn::{CharacterSpawnedEvent, SlotManager};
use crate::game::combat::systems::target_select::{SelectAbilityTarget, SelectAbilityTargetEvent};
use crate::game::control::camera::combat::CameraTargetPoint;
use crate::game::ui::ability::{ShowAbilitiesEvent, UiAbilitySelectEvent, UiAbilityTargetTag};
use crate::game::ui::crosshair::{CrosshairClickedEvent, CrosshairType, UiCrosshair};
use crate::game::ui::description::{ShowDescriptionPanelEvent, UiDescriptionPanel, UiDescription};
use crate::game::ui::hud::{UiBase, UiCharacterBase};
use crate::game::ui::status::UiStatus;
use crate::game::ui::UiDisengageEvent;
use crate::game::combat::ability::charge::ChargeAbility;
use crate::game::map::CurrentState;

#[derive(Debug, SystemDesc, new)]
#[system_desc(name(CombatUiSystemDesc))]
pub struct CombatUiSystem {
    #[system_desc(event_channel_reader)]
    spawn_reader: ReaderId<CharacterSpawnedEvent>,

    #[system_desc(event_channel_reader)]
    input_reader: ReaderId<InputEvent<StringBindings>>,

    #[system_desc(event_channel_reader)]
    ability_select_reader: ReaderId<UiAbilitySelectEvent>,
}

impl<'s> System<'s> for CombatUiSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Principal>,
        WriteStorage<'s, CombatRoot>,
        WriteStorage<'s, Parent>,
        WriteStorage<'s, Team>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Character>,
        ReadStorage<'s, Ability>,
        WriteStorage<'s, AbilityInvoke>,
        WriteStorage<'s, UiBase>,
        WriteStorage<'s, UiCharacterBase>,
        WriteStorage<'s, UiStatus>,
        WriteStorage<'s, UiCrosshair>,
        WriteStorage<'s, UiDescription>,
        ReadStorage<'s, SlotManager>,
        ReadStorage<'s, ChargeAbility>,
        ReadStorage<'s, SelectAbilityTarget>,
        ReadExpect<'s, ParentHierarchy>,
        Write<'s, CameraTargetPoint>,
        Read<'s, CurrentState>,
        Read<'s, EventChannel<CharacterSpawnedEvent>>,
        Write<'s, EventChannel<ShowAbilitiesEvent>>,
        Read<'s, EventChannel<InputEvent<StringBindings>>>,
        Write<'s, EventChannel<UiDisengageEvent>>,
        Write<'s, EventChannel<UiAbilitySelectEvent>>,
        Write<'s, EventChannel<SelectAbilityTargetEvent>>,
    );

    fn run(&mut self, (entities, mut principals, roots, mut parents, teams, transforms, mut characters, abilities, mut ability_invokes, mut ui_bases, mut ui_character_bases, mut ui_statuses, mut ui_crosshairs, mut descriptions, slot_managers, charge_abilities, select_ability_targets, hierarchy, mut target_point, current_state, spawned_events, mut show_abilities_events, input_events, mut disengage_events, mut ability_select_events, mut select_ability_target_events): Self::SystemData) {
        for event in input_events.read(&mut self.input_reader) {
            match event {
                InputEvent::MouseButtonPressed(btn) => {
                    if *btn == MouseButton::Right {
                        disengage_events.single_write(UiDisengageEvent::NegatingInput);
                    }
                }
                InputEvent::MouseWheelMoved(_) => {
                    disengage_events.single_write(UiDisengageEvent::NegatingInput);
                }
                InputEvent::KeyPressed { key_code: VirtualKeyCode::Space, .. } => {
                    if let Some(character_ent) = target_point.1 {
                        if let Some((root, _)) = get_root::<CombatRoot, _, _>(&parents, &roots, character_ent) {
                            if root.current_state == CombatState::InTurn(Team::Friendly) {
                                if let None = get_root(&parents, &select_ability_targets, character_ent) {
                                    if Principal::is_root_engaged(&parents, &principals, character_ent) != Some(true) {
                                        for (ability_ent, charge_ability, _) in (&entities, &charge_abilities, hierarchy.all_children(character_ent)).join() {
                                            ability_select_events.single_write(
                                                UiAbilitySelectEvent {
                                                    ability_ent,
                                                }
                                            );
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                _ => {}
            }
        }

        for event in spawned_events.read(&mut self.spawn_reader) {
            let character_ent = event.character_ent;

            if let Some(character) = characters.get(character_ent) {

                // Delete if we already have an entity.
                if let Some(existing_base) = ui_bases.get(character_ent) {
                    entities.delete(existing_base.entity());
                }

                let ui_ent = entities.create();
                ui_character_bases.insert(ui_ent, UiCharacterBase::new(character_ent));

                ui_bases.insert(character_ent, UiBase(ui_ent));

                let crosshair_ent = entities.create();
                let mut crosshair: UiCrosshair = UiCrosshair::new(CrosshairType::Passive(event.action.team));
                crosshair.uniform_scale = character.crosshair_scale();
                let description: UiDescription = UiDescription::new(character.name().to_string(), character.description().to_string());
                ui_crosshairs.insert(crosshair_ent, crosshair);
                descriptions.insert(crosshair_ent, description);
                parents.insert(crosshair_ent, Parent { entity: ui_ent });

                let status_ent = entities.create();
                let (team, team_ent) = Team::get_team(&parents, &teams, character_ent).expect("No team!");
                let status: UiStatus = UiStatus::new(character_ent, character.name().to_string(), team);
                ui_statuses.insert(status_ent, status);
                parents.insert(status_ent, Parent { entity: ui_ent });
            }
        }

        for event in ability_select_events.read(&mut self.ability_select_reader) {
            let ability_ent = event.ability_ent;
            if let Some(ability) = abilities.get(ability_ent) {
                if let Some((_, character_ent)) = get_root::<Character, _, _>(&parents, &characters, ability_ent) {
                    if Character::can_take_turn(&characters, character_ent, ability.data.charge.rated_charge()) {
                        ability_invokes.insert(ability_ent, AbilityInvoke::default());
                        if let Some(_) = ability.data.target_info {
                            select_ability_target_events.single_write(SelectAbilityTargetEvent {
                                ability_ent,
                                as_principal: false,
                                perform_on_select: true,
                            });
                        }
                        disengage_events.single_write(UiDisengageEvent::AbilitySelected);
                    }
                }
            }
        }


        for (root, slot_manager) in (&roots, &slot_managers).join() {
            if root.current_state == CombatState::Init {
                if let Some(master) = slot_manager.for_team(Team::Friendly).master() {
                    if let Some(character) = characters.get_mut(master) {
                        character.set_health(current_state.master_health * character.max_health());
                    }
                }
            }
            if root.current_state == CombatState::DoneTurn(Team::Enemy) {
                target_point.1 = None;
            }
            if target_point.1 == None {
                for opt_char in slot_manager.friendly.occupied().iter() {
                    if let Some(character_ent) = opt_char {
                        if let Some(character) = characters.get(*character_ent) {
                            if character.has_turn() {
                                if let Some(transform) = transforms.get(*character_ent) {
                                    *target_point = CameraTargetPoint(Some(*transform.translation()), Some(*character_ent));
                                    break;
                                }
                            }
                        }

                    }
                }
            }
        }


        // Check if the current target is valid.
        if let Some(current_ent) = target_point.1 {
            if Principal::is_root_engaged(&parents, &principals, current_ent) != Some(true) {
                if let Some(character) = characters.get(current_ent) {
                    if !character.has_turn() {
                        if let Some((_, team_ent)) = Team::get_team(&parents, &teams, current_ent) {
                            for (ent, character, transform, _) in (&entities, &characters, &transforms, hierarchy.all_children(team_ent)).join() {
                                if character.has_turn() {
                                    if ent != current_ent {
                                        *target_point = CameraTargetPoint(Some(*transform.translation()), Some(ent));
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug, SystemDesc, new)]
#[system_desc(name(CrosshairUiControllerSystemDesc))]
pub struct CrosshairUiControllerSystem {
    #[system_desc(event_channel_reader)]
    crosshair_click_reader: ReaderId<CrosshairClickedEvent>,

    #[system_desc(event_channel_reader)]
    input_event_reader: ReaderId<InputEvent<StringBindings>>,
}

impl<'s> System<'s> for CrosshairUiControllerSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Principal>,
        ReadStorage<'s, CombatRoot>,
        WriteStorage<'s, Parent>,
        WriteStorage<'s, Team>,
        WriteStorage<'s, Character>,
        WriteStorage<'s, Transform>,
        ReadStorage<'s, UiCharacterBase>,
        WriteStorage<'s, UiCrosshair>,
        WriteStorage<'s, SelectAbilityTarget>,
        Write<'s, CameraTargetPoint>,
        Read<'s, InputHandler<StringBindings>>,
        ReadExpect<'s, ParentHierarchy>,
        Read<'s, EventChannel<CrosshairClickedEvent>>,
        Write<'s, EventChannel<ShowAbilitiesEvent>>,
        Write<'s, EventChannel<UiDisengageEvent>>,
    );

    fn run(&mut self, (entities, principals, roots, parents, teams, characters, mut transforms, ui_character_bases, mut ui_crosshairs, select_ability_targets, mut camera_target_point, input, hierarchy, crosshair_clicked_events, mut show_abilities_events, mut disengage_events): Self::SystemData) {
        for (entity, mut crosshair) in (&entities, &mut ui_crosshairs).join() {
            if let Some((character_ui, _)) = get_root::<UiCharacterBase, _, _>(&parents, &ui_character_bases, entity) {
                if let Some((root, _)) = get_root::<CombatRoot, _, _>(&parents, &roots, character_ui.character_ent) {
                    if root.current_state == CombatState::InTurn(Team::Friendly) {
                        if let None = get_root(&parents, &select_ability_targets, character_ui.character_ent) {
                            if Principal::is_root_engaged(&parents, &principals, character_ui.character_ent) == Some(true) {
                                crosshair.visible = false;
                            } else {
                                if let Some((team, _)) = get_root::<Team, _, _>(&parents, &teams, character_ui.character_ent) {
                                    crosshair.ty.set_team(*team);
                                    if let Some(character) = characters.get(character_ui.character_ent) {
                                        if !character.has_turn() && *team == Team::Friendly {
                                            // Hide crosshair if the character has performed all abilities for this turn.
                                            crosshair.visible = false;
                                        } else {
                                            crosshair.visible = true;
                                            // Hover behaviour
                                            if let Some((ui_character, ui_ent)) = get_root::<UiCharacterBase, _, _>(&parents, &ui_character_bases, entity) {
                                                if camera_target_point.1 == Some(ui_character.character_ent) {
                                                    crosshair.ty = CrosshairType::Target(crosshair.ty.team());
                                                } else {
                                                    crosshair.ty = CrosshairType::Passive(crosshair.ty.team());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        for event in crosshair_clicked_events.read(&mut self.crosshair_click_reader) {
            if let Some((ui_character_base, ui_ent)) = get_root::<UiCharacterBase, _, _>(
                &parents,
                &ui_character_bases,
                event.entity(),
            ) {
                if let Some((root, root_ent)) = get_root::<CombatRoot, _, _>(&parents, &roots, ui_character_base.character_ent) {
                    if let None = get_root::<SelectAbilityTarget, _, _>(&parents, &select_ability_targets, ui_character_base.character_ent) {
                        if root.current_state == CombatState::InTurn(Team::Friendly) {
                            // Only perform ability selection if there are no principles.
                            if Principal::is_root_engaged(&parents, &principals, ui_character_base.character_ent) != Some(true) {
                                let character_ent = ui_character_base.character_ent;
                                if let Some(character) = characters.get(character_ent) {
                                    // Check if the character is on our team.
                                    if let Some((team, team_ent)) = Team::get_team(&parents, &teams, ui_character_base.character_ent) {
                                        if team == Team::Friendly {
                                            // Display our ability ui if the character is not spent.
                                            if character.has_turn() {
                                                // Since this is focused we can also change the camera pos.
                                                if let Some(transform) = transforms.get(character_ent) {
                                                    *camera_target_point = CameraTargetPoint(Some(*transform.translation()), Some(character_ent));
                                                }

//                                            // Clear any existing target crosshairs.
//                                            for mut crosshair in (&mut ui_crosshairs).join() {
//                                                crosshair.ty = CrosshairType::Passive(crosshair.ty.team());
//                                            }
//                                            // Set our new target crosshair.
//                                            if let Some(crosshair) = ui_crosshairs.get_mut(event.entity()) {
//                                                crosshair.ty = CrosshairType::Target(crosshair.ty.team());
//                                            }

                                                // We can use our character.
                                                show_abilities_events.single_write(ShowAbilitiesEvent::new(character_ent));
                                                disengage_events.single_write(UiDisengageEvent::TargetChanged);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}