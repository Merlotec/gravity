use std::collections::HashMap;
use std::thread::current;

use amethyst::{
    core::{
        Parent,
        ParentHierarchy,
    },
    ecs::prelude::*,
    prelude::SystemDesc,
    shrev::EventChannel,
};

use crate::{
    core::activity::ActivityState,
    game::{
        character::{
            Character,
            CharacterId,
        },
        combat::{
            ability::Ability,
            CombatRoot,
            CombatState,
            spawn::SlotManager,
            tactical::AiAbilitySelectionQuery,
            Team,
            TickTurn,
        },
    },
};
use crate::core::{get_root, select_rng};
use crate::game::combat::ability::{AbilityPerform, perform_ability, AbilityTargetType, AbilityTargetArea};
use crate::game::combat::process::Principal;
use crate::game::combat::systems::delay::Delay;
use crate::game::combat::tactical::AiAbilitySelection;
use crate::game::combat::systems::enemy_wave::SpawnWaveEvent;
use crate::game::character::CharacterDefeatedEvent;

/// Contains 'live' data relating to the enemy team.
pub struct EnemyState {
    /// The current enemy wave.
    wave: usize,
}

#[derive(Debug, Copy, Clone, Default, Component)]
pub struct HasDelayedTag;

#[derive(Debug, new, SystemDesc)]
#[system_desc(name(EnemyControllerSystemDesc))]
pub struct EnemyControllerSystem {
    #[system_desc(event_channel_reader)]
    defeated_event_reader: ReaderId<CharacterDefeatedEvent>,
}

impl<'s> System<'s> for EnemyControllerSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Character>,
        ReadStorage<'s, Team>,
        ReadStorage<'s, Ability>,
        WriteStorage<'s, AiAbilitySelectionQuery>,
        ReadStorage<'s, SlotManager>,
        WriteStorage<'s, CombatRoot>,
        WriteStorage<'s, HasDelayedTag>,
        WriteStorage<'s, Principal>,
        WriteStorage<'s, Delay>,
        ReadStorage<'s, Parent>,
        ReadExpect<'s, ParentHierarchy>,
        Write<'s, EventChannel<SpawnWaveEvent>>,
        Read<'s, EventChannel<CharacterDefeatedEvent>>,
    );

    fn run(&mut self, (entities, mut characters, teams, abilities, mut ability_selections, slot_managers, mut combat_roots, mut delayed_tags, mut principals, mut delays, parents, hierarchy, mut spawn_wave_events, defeated_events): Self::SystemData) {
        for (entity, mut root, slot_manager) in (&entities, &mut combat_roots, &slot_managers).join() {
            // Only execute in no principals are running.
            if Principal::is_root_engaged(
                &parents,
                &principals,
                entity,
            ) != Some(true) {
                // Enemy turn!
                if root.current_state == CombatState::InTurn(Team::Enemy) {
                    if delayed_tags.contains(entity) {
                        for (i, entity_opt) in slot_manager.enemy.occupied().iter().enumerate() {
                            if let Some(character_ent) = *entity_opt {
                                if let Some(character) = characters.get(character_ent) {
                                    if character.has_turn() {
                                        // Perform ability.
                                        let children: BitSet = hierarchy.all_children(character_ent);
                                        let mut has_active_abilities: bool = false;
                                        for (entity, ability, bit) in (&entities, &abilities, &children).join() {
                                            // Check if ability can be performed.
                                            if Character::can_take_turn(
                                                &characters,
                                                character_ent,
                                                ability.data.charge.rated_charge(),
                                            ) {
                                                let target_entities = Ability::targets_for(
                                                    &entities,
                                                    &parents,
                                                    &abilities,
                                                    &characters,
                                                    &slot_managers,
                                                    &teams,
                                                    entity,
                                                );
//                                                let mut target_entities: Vec<Entity> = Vec::new();
//                                                if let Some(target_info) = ability.data.target_info {
//                                                    if target_info.ty == AbilityTargetType::Enemy || target_info.ty == AbilityTargetType::All {
//                                                        let count = slot_manager.friendly.count();
//                                                        for (i, friendly_opt) in slot_manager.friendly.occupied().iter().enumerate() {
//                                                            if i != 0 || count == 1 || target_info.area == AbilityTargetArea::All {
//                                                                if let Some(friendly) = *friendly_opt {
//                                                                    target_entities.push(friendly);
//                                                                }
//                                                            }
//                                                        }
//                                                    }
//                                                    if target_info.ty == AbilityTargetType::Friendly || target_info.ty == AbilityTargetType::All {
//                                                        for (i, enemy_opt) in slot_manager.enemy.occupied().iter().enumerate() {
//                                                            if let Some(enemy) = *enemy_opt {
//                                                                target_entities.push(enemy);
//                                                            }
//                                                        }
//                                                    }
//                                                }

                                                ability_selections.insert(entity, AiAbilitySelectionQuery::new(target_entities));
                                                has_active_abilities = true;
                                            }
                                        }
                                        if has_active_abilities {
                                            delayed_tags.remove(entity);
                                            break;
                                        } else {
                                            if let Some(character) = characters.get_mut(character_ent) {
                                                character.use_turns();
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        delayed_tags.insert(entity, HasDelayedTag::default());
                        Delay::principal_delay(
                            &parents,
                            &mut principals,
                            &mut delays,
                            entity,
                            0.4,
                        );
                    }
                } else if root.current_state == CombatState::DoneTurn(Team::Enemy) {
                    delayed_tags.remove(entity);
                }
            }
        }
    }
}

#[derive(Debug, Copy, Clone, Default, SystemDesc)]
#[system_desc(name(EnemyAbilityInvocationSystemDesc))]
pub struct EnemyAbilityInvocationSystem;

impl<'s> System<'s> for EnemyAbilityInvocationSystem {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, Parent>,
        WriteStorage<'s, Principal>,
        WriteStorage<'s, Character>,
        ReadStorage<'s, Ability>,
        WriteStorage<'s, AiAbilitySelectionQuery>,
        WriteStorage<'s, AbilityPerform>,
    );

    fn run(&mut self, (entities, parents, mut principals, mut characters, abilities, mut ability_selections, mut ability_performs): Self::SystemData) {
        let mut sets: HashMap<Entity, Vec<(AiAbilitySelection, Entity)>> = HashMap::new();
        let mut to_remove: Vec<Entity> = Vec::new();
        for (entity, ability, selection) in (&entities, &abilities, &ability_selections).join() {
            if let Some((_, character_ent)) = get_root::<Character, _, _>(&parents, &characters, entity) {
                if let Some(res) = selection.result.clone() {
                    if let Some(existing) = sets.get_mut(&character_ent) {
                        existing.push((res, entity));
                    } else {
                        sets.insert(character_ent, vec![(res, entity)]);
                    }
                }
            }
            to_remove.push(entity);
        }

        for ent in to_remove {
            ability_selections.remove(ent);
        }

        for (character_ent, selections) in sets {
            let mut values: Vec<f32> = Vec::with_capacity(selections.len());
            for (selection, _) in selections.iter() {
                values.push(selection.score);
            }

            if let Some(selection) = select_rng(&values) {
                // make principal
                perform_ability(
                    &parents, &mut principals, &abilities, &mut ability_performs,
                    selections[selection].1,
                    selections[selection].0.target.clone(),
                ).expect("No `Principal` in hierarchy!");
            } else {
                if let Some(character) = characters.get_mut(character_ent) {
                    character.use_turns();
                }
            }

        }
    }
}
