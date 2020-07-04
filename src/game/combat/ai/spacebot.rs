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

use crate::game::combat::ability::{
    overclock::OverclockAbility,
    sheild::ReinforceAbility,
    nanobots::NanobotsAbility,
    empower::EmpowerAbility,
    focus::FocusAbility,
    AbilityTarget
};
use crate::core::{get_root, select_rng};
use crate::game::combat::tactical::AiAbilitySelection;
use crate::game::character::SupporterSpacebotDrone;
use crate::game::combat::status::StatusType;

#[derive(Debug, SystemDesc, new)]
#[system_desc(name(SpacebotAiSystemDesc))]
pub struct SpacebotAiSystem;

impl<'s> System<'s> for SpacebotAiSystem {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, Parent>,
        ReadStorage<'s, Character>,
        WriteStorage<'s, AiAbilitySelectionQuery>,
        ReadStorage<'s, OverclockAbility>,
        ReadStorage<'s, ReinforceAbility>,
        ReadStorage<'s, NanobotsAbility>,
        ReadStorage<'s, EmpowerAbility>,
        ReadStorage<'s, FocusAbility>,
    );

    fn run(&mut self, (entities, parents, characters, mut ability_selections, overclocks, reinforces, nanobots, empowers, focuses): Self::SystemData) {

        // Overclock
        for (ability_ent, selection, _) in (&entities, &mut ability_selections, overclocks.mask()).join() {
            if let Some((_, character_ent)) = get_root::<Character, _, _>(&parents, &characters, ability_ent) {
                let mut chances: Vec<f32> = Vec::new();
                let mut targets: Vec<Entity> = Vec::new();
                let mut max_charge: f32 = 0.0;
                for target in selection.targets.iter() {
                    if *target != character_ent {
                        if let Some(target_character) = characters.get(*target) {
                            if !target_character.has_status(StatusType::Scramble) {
                                chances.push(target_character.relative_charge());
                                targets.push(*target);
                                if target_character.relative_charge() > max_charge {
                                    max_charge = target_character.relative_charge();
                                }
                            }
                        }
                    }
                }
                if let Some(target_idx) = select_rng(&chances) {
                    selection.result = Some(AiAbilitySelection {
                        target: AbilityTarget::Single(targets[target_idx]),
                        score: max_charge * 3.0,
                    });
                }
            }
        }

        // Defend
        for (ability_ent, selection, _) in (&entities, &mut ability_selections, reinforces.mask()).join() {
            let mut chances: Vec<f32> = Vec::new();
            let mut targets: Vec<Entity> = Vec::new();
            let mut max_health: f32 = 1.0;
            for target in selection.targets.iter() {
                if let Some(character) = characters.get(*target) {
                    chances.push(1.5 - character.relative_health());
                    targets.push(*target);
                    if character.relative_health() < max_health {
                        max_health = character.relative_health();
                    }
                }
            }
            if let Some(target_idx) = select_rng(&chances) {
                selection.result = Some(AiAbilitySelection {
                    target: AbilityTarget::Single(targets[target_idx]),
                    score: (1.0 - max_health) * 3.0,
                });
            }
        }

        // Nanobots
        for (ability_ent, selection, _) in (&entities, &mut ability_selections, nanobots.mask()).join() {
            let mut min_health: f32 = 1.0;
            let mut min_target: Option<Entity> = None;
            let mut relative_total: f32 = 0.0;
            let mut count: usize = 0;
            for target in selection.targets.iter() {
                if let Some(character) = characters.get(*target) {
                    relative_total += character.relative_health();
                    count += 1;
                    if character.relative_health() < min_health {
                        min_health = character.relative_health();
                        min_target = Some(*target);
                    }
                }
            }
            let ave: f32 = relative_total / (count as f32);

            if min_health < 0.2 {
                selection.result = Some(AiAbilitySelection {
                    target: AbilityTarget::Single(min_target.unwrap()),
                    score: (1.0 - min_health) * 5.0,
                });
            } else {
                selection.result = Some(AiAbilitySelection {
                    target: AbilityTarget::Multi(selection.targets.clone()),
                    score: (1.0 - ave) * 3.0,
                });
            }

        }

        // Empower
        for (ability_ent, selection, _) in (&entities, &mut ability_selections, empowers.mask()).join() {
            if let Some((character, _)) = get_root::<Character, _, _>(&parents, &characters, ability_ent) {
                if !character.has_status(StatusType::Empower) {
                    let mut chances: Vec<f32> = Vec::new();
                    let mut targets: Vec<Entity> = Vec::new();
                    for target in selection.targets.iter() {
                        if let Some(character) = characters.get(*target) {
                            if !character.has_status(StatusType::Scramble) && !character.has_status(StatusType::Empower) {
                                if character.id() != SupporterSpacebotDrone::character_id() {
                                    chances.push(0.5 + character.relative_charge());
                                    targets.push(*target);
                                }
                            }
                        }
                    }

                    if let Some(target_idx) = select_rng(&chances) {
                        selection.result = Some(AiAbilitySelection {
                            target: AbilityTarget::Single(targets[target_idx]),
                            score: 3.0,
                        });
                    }
                } else {
                    selection.result = Some(AiAbilitySelection {
                        target: AbilityTarget::Multi(Vec::new()),
                        score: 0.0,
                    });
                }
            }
        }

        // Focus
        for (ability_ent, selection, _) in (&entities, &mut ability_selections, focuses.mask()).join() {
            if let Some((character, _)) = get_root::<Character, _, _>(&parents, &characters, ability_ent) {
                if !character.has_status(StatusType::Focus) {
                    let mut chances: Vec<f32> = Vec::new();
                    let mut targets: Vec<Entity> = Vec::new();
                    for target in selection.targets.iter() {
                        if let Some(character) = characters.get(*target) {
                            if !character.has_status(StatusType::Scramble) && !character.has_status(StatusType::Focus) {
                                if character.id() != SupporterSpacebotDrone::character_id() {
                                    chances.push(0.5 + character.relative_charge());
                                    targets.push(*target);
                                }
                            }
                        }
                    }

                    if let Some(target_idx) = select_rng(&chances) {
                        selection.result = Some(AiAbilitySelection {
                            target: AbilityTarget::Single(targets[target_idx]),
                            score: 3.0,
                        });
                    }
                } else {
                    selection.result = Some(AiAbilitySelection {
                        target: AbilityTarget::Multi(Vec::new()),
                        score: 0.0,
                    });
                }
            }
        }

    }
}