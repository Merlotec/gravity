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
    AbilityTarget
};
use crate::core::{get_root, select_rng};
use crate::game::combat::tactical::AiAbilitySelection;
use crate::game::character::{SupporterSpacebotDrone, CharacterStore};
use crate::game::combat::status::StatusType;
use crate::game::combat::ability::spawn::{all_rank_options, SpawnAbility};
use crate::game::combat::{Rank, CharacterSpawn};

#[derive(Debug, SystemDesc, new)]
#[system_desc(name(GuardianAiSystemDesc))]
pub struct GuardianAiSystem;

impl<'s> System<'s> for GuardianAiSystem {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, Parent>,
        ReadStorage<'s, Character>,
        ReadStorage<'s, SlotManager>,
        ReadStorage<'s, Team>,
        WriteStorage<'s, AiAbilitySelectionQuery>,
        WriteStorage<'s, SpawnAbility>,
        Read<'s, CharacterStore>,
    );

    fn run(&mut self, (entities, parents, characters, slot_managers, teams, mut ability_selections, mut spawn_abilities, character_store): Self::SystemData) {
        // Spawn
        for (ability_ent, selection, mut spawn_ability) in (&entities, &mut ability_selections, &mut spawn_abilities).join() {
            if let Some((team, _)) = Team::get_team(&parents, &teams, ability_ent) {
                if let Some((slot_manager, _)) = get_root::<SlotManager, _, _>(&parents, &slot_managers, ability_ent) {
                    if slot_manager.enemy.count() < 4 {
                        let mut chances: Vec<f32> = Vec::new();
                        let mut chars: Vec<CharacterId> = Vec::new();
                        for (character_id, _) in character_store.get_spawnable(team) {
                            let mut contains: usize = 0;
                            for existing in slot_manager.enemy.occupied() {
                                if let Some(character_ent) = *existing {
                                    if let Some(character) = characters.get(character_ent) {
                                        if character.id() == character_id {
                                            contains += 1;
                                        }
                                    }
                                }
                            }
                            chances.push(1.0 / ((contains + 1) as f32));
                            chars.push(character_id);
                        }

                        if let Some(char_idx) = select_rng(&chances) {
                            let mut rank_chances: Vec<f32> = Vec::new();
                            let mut ranks: Vec<Rank> = Vec::new();
                            if let Some((character, _)) = get_root::<Character, _, _>(&parents, &characters, ability_ent) {
                                for rank_opt in all_rank_options() {
                                    if character.charge() >= rank_opt.charge {
                                        rank_chances.push(rank_opt.charge);
                                        ranks.push(rank_opt.rank);
                                    }
                                }
                            }

                            if let Some(rank_idx) = select_rng(&rank_chances) {
                                spawn_ability.next_spawn = Some(CharacterSpawn {
                                    character_id: chars[char_idx],
                                    rank: ranks[rank_idx],
                                });
                                selection.result = Some(AiAbilitySelection {
                                    target: AbilityTarget::Multi(Vec::new()),
                                    score: 15.0 / (slot_manager.enemy.count() as f32),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}