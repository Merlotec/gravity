use amethyst::{
    ecs::prelude::*,
    core::{
        SystemDesc,
        Parent,
        Transform,
    },
    shrev::{
        EventChannel,
        ReaderId,
    }
};
use crate::game::character::{CharacterId, CharacterStore, Character, CharacterData};
use crate::game::combat::spawn::{SpawnAction, SlotManager};
use crate::game::combat::{Team, Wave, Rank, Difficulty};
use crate::game::combat::process::Principal;
use crate::core::get_root;
use crate::game::map::CurrentState;

#[derive(Debug, Clone, PartialEq)]
pub struct SpawnWaveEvent {
    pub wave: Wave,
    pub team_ent: Entity,
    pub idx: usize,
}

#[derive(Debug, new, SystemDesc)]
#[system_desc(name(EnemyWaveSystemDesc))]
pub struct EnemyWaveSystem {
    #[system_desc(event_channel_reader)]
    spawn_wave_event_reader: ReaderId<SpawnWaveEvent>,
}

impl<'s> System<'s> for EnemyWaveSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Principal>,
        WriteStorage<'s, Parent>,
        WriteStorage<'s, SpawnAction>,
        ReadStorage<'s, Team>,
        ReadStorage<'s, SlotManager>,
        Read<'s, CharacterStore>,
        Write<'s, CurrentState>,
        Read<'s, EventChannel<SpawnWaveEvent>>,
    );

    fn run(&mut self, (entities, mut principals, mut parents, mut spawn_actions, teams, slot_managers, character_store, current_state, spawn_wave_events): Self::SystemData) {
        for event in spawn_wave_events.read(&mut self.spawn_wave_event_reader) {
            if let Some(boss) = event.wave.master {
                let team = teams.get(event.team_ent).expect("No team!");
                let (slot_mgr, _) = get_root::<SlotManager, _, _>(&parents, &slot_managers, event.team_ent).expect("No slot manager!");
                if slot_mgr.for_team(*team).is_occupied(0) {
                    panic!("Master slot is occupied when trying to spawn a master drone!");
                }
                let character_data: CharacterData = {
                    let (mut base_data, _) = character_store.characters.get(&boss.character_id).expect("Invalid character id!");
                    match current_state.difficulty {
                        Difficulty::Easy => {
                            base_data.max_charge *= 0.75;
                            base_data.max_health *= 0.75;
                            base_data.initial_charge *= 0.75;
                            base_data.natural_charge *= 0.75;
                            base_data.artificial_charge *= 0.75;
                            base_data.base_dmg *= 0.75;
                            base_data.base_evade *= 0.75;
                            base_data.base_accuracy *= 0.75;
                        },
                        Difficulty::Normal => {},
                        Difficulty::Hard => {
                            base_data.max_charge *= 1.25;
                            base_data.max_health *= 1.25;
                            base_data.artificial_charge *= 1.25;
                            base_data.base_dmg *= 1.25;
                            base_data.base_evade *= 1.25;
                            base_data.base_accuracy *= 1.25;
                        },
                        Difficulty::Extreme => {
                            base_data.max_charge *= 1.5;
                            base_data.max_health *= 1.5;
                            base_data.initial_charge *= 1.5;
                            base_data.natural_charge *= 1.5;
                            base_data.artificial_charge *= 1.5;
                            base_data.base_dmg *= 1.5;
                            base_data.base_evade *= 1.5;
                            base_data.base_accuracy *= 1.5;
                        },
                    }
                    base_data
                };
                Character::invoke_spawn(
                    &entities,
                    &mut principals,
                    &mut parents,
                    &mut spawn_actions,
                    event.team_ent,
                    boss.character_id,
                    None,
                    boss.rank,
                    *team,
                    0,
                    false,
                );

            }
            for (i, character_spawn) in event.wave.characters.iter().enumerate() {
                let slot_idx = i + 1;
                let team = teams.get(event.team_ent).expect("No team!");
                let (slot_mgr, _) = get_root::<SlotManager, _, _>(&parents, &slot_managers, event.team_ent).expect("No slot manager!");
                if slot_mgr.for_team(*team).is_occupied(slot_idx) {
                    panic!("Drone slot occupied! Spawn wave should only be triggered when there are no drones!");
                }
                let character_data: CharacterData = {
                    let (mut base_data, _) = character_store.characters.get(&character_spawn.character_id).expect("Invalid character id!");
                    match current_state.difficulty {
                        Difficulty::Easy => {
                            base_data.max_charge *= 0.75;
                            base_data.max_health *= 0.75;
                            base_data.initial_charge *= 0.75;
                            base_data.natural_charge *= 0.75;
                            base_data.artificial_charge *= 0.75;
                            base_data.base_dmg *= 0.75;
                            base_data.base_evade *= 0.75;
                            base_data.base_accuracy *= 0.75;
                        },
                        Difficulty::Normal => {},
                        Difficulty::Hard => {
                            base_data.max_charge *= 1.25;
                            base_data.max_health *= 1.25;
                            base_data.artificial_charge *= 1.25;
                            base_data.base_dmg *= 1.25;
                            base_data.base_evade *= 1.25;
                            base_data.base_accuracy *= 1.25;
                        },
                        Difficulty::Extreme => {
                            base_data.max_charge *= 1.5;
                            base_data.max_health *= 1.5;
                            base_data.initial_charge *= 1.5;
                            base_data.natural_charge *= 1.5;
                            base_data.artificial_charge *= 1.5;
                            base_data.base_dmg *= 1.5;
                            base_data.base_evade *= 1.5;
                            base_data.base_accuracy *= 1.5;
                        },
                    }
                    base_data
                };
                Character::invoke_spawn(
                    &entities,
                    &mut principals,
                    &mut parents,
                    &mut spawn_actions,
                    event.team_ent,
                    character_spawn.character_id,
                    Some(character_data),
                    character_spawn.rank,
                    *team,
                    slot_idx,
                    false,
                );

            }
        }
    }
}