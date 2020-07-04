use std::any::TypeId;

use amethyst::{
    assets::{
        Handle,
        Prefab,
    },
    core::Parent,
    ecs::prelude::*,
    prelude::SystemDesc,
    shrev::EventChannel,
};

use crate::{
    core::action::{
        Action,
        Invoke,
    },
    game::{
        character::{Character, CharacterId},
        combat::{
            ability::{Ability, AbilityInvoke},
            CombatRoot,
            process::Principal,
            spawn::{
                SlotManager,
                SpawnProcess,
                SpawnSource,
            },
        },
    },
};
use crate::core::{activity::ActivityState, get_root};
use crate::game::character::{CharacterPrefabData, CharacterStore, UnassignedCharacter, CharacterRole};
use crate::game::combat::{Team, Rank, CharacterSpawn};
use crate::game::combat::ability::{AbilityCharge, AbilityData, AbilityTargetArea, AbilityTargetInfo, AbilityTargetType, UnassignedAbility, AbilityList, AbilityUsability, AbilityPerform};
use crate::game::combat::spawn::SpawnAction;
use crate::game::ui::select_character::{CharacterSelectedEvent, SelectCharacterEvent, UiCharacterSelectPanel};
use crate::game::ui::select_rank::{RankSelectedEvent, ShowSelectRankUiEvent, UiSelectRankOption, UiSelectRankData};
use crate::game::ui::ability::{UiAbilitySelection, UiAbilityPanel};

#[derive(Debug, Copy, Clone, Default, Component)]
pub struct SpawnAbility {
    displaying: bool,
    pub next_spawn: Option<CharacterSpawn>
}

impl SpawnAbility {
    pub fn data() -> AbilityData {
        AbilityData {
            name: "Spawn Drone",
            desc: "Spawns a new drone in your team.",
            id: TypeId::of::<Self>(),
            system: TypeId::of::<SpawnAbilitySystem>(),
            charge: AbilityCharge::Range(100.0, 700.0),
            target_info: None,
            cooldown: 4,
        }
    }
}

pub fn all_rank_options() -> Vec<UiSelectRankOption> {
    vec![
        UiSelectRankOption {
            rank: Rank::Basic,
            charge: 100.0,
        },
        UiSelectRankOption {
            rank: Rank::Advanced,
            charge: 400.0,
        },
        UiSelectRankOption {
            rank: Rank::Elite,
            charge: 700.0,
        }
    ]
}

#[derive(Debug, Copy, Clone, Default, Component)]
pub struct SpawnAbilityDisplayTag;

#[derive(Debug, new, SystemDesc)]
#[system_desc(name(SpawnAbilitySystemDesc))]
pub struct SpawnAbilitySystem {
    #[system_desc(event_channel_reader)]
    character_selected_reader: ReaderId<CharacterSelectedEvent>,

    #[system_desc(event_channel_reader)]
    rank_selected_reader: ReaderId<RankSelectedEvent>,
}

impl<'s> System<'s> for SpawnAbilitySystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Principal>,
        WriteStorage<'s, SlotManager>,
        WriteStorage<'s, Character>,
        WriteStorage<'s, UnassignedCharacter>,
        WriteStorage<'s, Handle<Prefab<CharacterPrefabData>>>,
        WriteStorage<'s, SpawnProcess>,
        WriteStorage<'s, Ability>,
        WriteStorage<'s, UnassignedAbility>,
        WriteStorage<'s, AbilityInvoke>,
        WriteStorage<'s, AbilityPerform>,
        WriteStorage<'s, SpawnAbility>,
        WriteStorage<'s, SpawnAbilityDisplayTag>,
        WriteStorage<'s, Parent>,
        ReadStorage<'s, Team>,
        Read<'s, CharacterStore>,
        Write<'s, EventChannel<SelectCharacterEvent>>,
        WriteStorage<'s, SpawnAction>,
        Read<'s, EventChannel<CharacterSelectedEvent>>,
        Write<'s, EventChannel<ShowSelectRankUiEvent>>,
        Write<'s, EventChannel<RankSelectedEvent>>,
    );

    fn setup(&mut self, world: &mut World) {
        world.fetch_mut::<AbilityList>().register(SpawnAbility::data(), AbilityUsability::Role(CharacterRole::Master));
    }

    fn run(&mut self, (entities, mut principals, mut slot_managers, mut characters, mut unassigned_characters, mut character_prefabs, mut spawn_processes, mut abilities, mut unassigned_abilities, mut ability_invokes, mut ability_performs, mut spawn_abilities, mut display_tags, mut parents, teams, character_store, mut select_character_events, mut spawn_actions, character_selected_events, mut show_select_rank, mut rank_selected_events): Self::SystemData) {
        for (entity, ability, _) in (&entities, &abilities, unassigned_abilities.mask().clone()).join() {
            if ability.data.id == TypeId::of::<SpawnAbility>() {
                spawn_abilities.insert(entity, SpawnAbility::default());
                unassigned_abilities.remove(entity);
            }
        }

        for (ability_ent, mut ability, _) in (&entities, &mut abilities, spawn_abilities.mask()).join() {
            if let Some((team, _)) = Team::get_team(&parents, &teams, ability_ent) {
                if let Some((slot_manager, _)) = get_root::<SlotManager, _, _>(&parents, &slot_managers, ability_ent) {
                    if slot_manager.for_team(team).count() >= 4 {
                        ability.locked = true;
                    } else {
                        ability.locked = false;
                    }
                }
            }
        }

        // Check if the ability has been triggered.
        for (ent, invoke, mut spawn_ability, _) in (&entities, &ability_invokes, &mut spawn_abilities, !display_tags.mask().clone()).join() {
            select_character_events.single_write(SelectCharacterEvent::new(ent));
            display_tags.insert(ent, SpawnAbilityDisplayTag);
        }

        for (ent, perform, spawn_ability) in (&entities, &ability_performs, &spawn_abilities).join() {
            if let Some(spawn) = spawn_ability.next_spawn {
                for rank in all_rank_options() {
                    if rank.rank == spawn.rank {
                        rank_selected_events.single_write(
                            RankSelectedEvent {
                                owner: ent,
                                character_id: spawn.character_id,
                                selection: Some(rank.clone()),
                            }
                        );
                    }
                }

            }

        }

        for selection_evt in character_selected_events.read(&mut self.character_selected_reader) {
            let ent = selection_evt.owner;
            if spawn_abilities.contains(ent) {
                if let Some(ability) = abilities.get(ent) {
                    if let Some((_, character_ent)) = get_root::<Character, _, _>(&parents, &characters, ent) {
                        if let Some(character_id) = selection_evt.id {
                            show_select_rank.single_write(
                                ShowSelectRankUiEvent {
                                    character_id,
                                    owner: ent,
                                    options: all_rank_options(),
                                }
                            );
                        } else {
                            display_tags.remove(ent);
                            ability_invokes.remove(ent);
                            ability_performs.remove(ent);
                        }
                    }
                }
            }
        }

        for event in rank_selected_events.read(&mut self.rank_selected_reader) {
            let ent = event.owner;

            display_tags.remove(ent);
            ability_invokes.remove(ent);
            ability_performs.remove(ent);
            if spawn_abilities.contains(ent) {
                if let Some(ability) = abilities.get(ent) {
                    if let Some((_, character_ent)) = get_root::<Character, _, _>(&parents, &characters, ent) {
                        Principal::try_root_disengage(&parents, &mut principals, ent, TypeId::of::<Self>());
                        // If we have selected a valid character, then spawn.
                        if let Some(selection) = event.selection {
                            if Character::try_take_turn(&mut characters, character_ent, selection.charge) {
                                let (team, team_ent) = Team::get_team(&parents, &teams, ent).expect("No team!");
                                let (slot_mgr, _) = get_root::<SlotManager, _, _>(&parents, &slot_managers, ent).expect("No slot manager!");
                                if let Some(slot_idx) = slot_mgr.for_team(team).find_next(false) {
                                    Character::invoke_spawn(
                                        &entities,
                                        &mut principals,
                                        &mut parents,
                                        &mut spawn_actions,
                                        team_ent,
                                        event.character_id,
                                        None,
                                        selection.rank,
                                        team,
                                        slot_idx,
                                        true,
                                    );
                                }
                            } else {
                                panic!("[SpawnAbilitySystem] Unexpected failure to take turn.");
                            }
                        }
                    }
                }
            }
        }
    }
}