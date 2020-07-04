
use std::any::TypeId;

use amethyst::{
    assets::{
        Handle,
        Prefab,
    },
    core::{
        Parent,
        ParentHierarchy,
        Transform,
    },
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
use crate::game::combat::{Team};
use crate::game::combat::ability::{AbilityCharge, AbilityData, AbilityTargetArea, AbilityTargetInfo, AbilityTargetType, UnassignedAbility, AbilityPerform, AbilityTarget, AbilityList, AbilityUsability};
use crate::game::combat::spawn::SpawnAction;
use crate::game::ui::select_character::{CharacterSelectedEvent, SelectCharacterEvent};
use crate::game::ui::hack::{ShowHackUiEvent, HackSelectedEvent, UiHackOption};

pub const MAX_OPTIONS: i32 = 10;
pub const BASE_CHANCE: f32 = 0.1;

#[derive(Debug, Copy, Clone, Default, Component)]
pub struct HackAbility {
    displaying: bool,
}

impl HackAbility {
    pub fn data() -> AbilityData {
        AbilityData {
            name: "Hack",
            desc: "Attempts to hack an enemy drone, resulting in all of the hacked type joining our team.",
            id: TypeId::of::<Self>(),
            system: TypeId::of::<HackAbilitySystem>(),
            charge: AbilityCharge::Range(100.0, 700.0),
            target_info: Some(AbilityTargetInfo {
                ty: AbilityTargetType::Enemy,
                area: AbilityTargetArea::Single,
            }),
            cooldown: 4,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct HackPerformedEvent {
    pub target_ent: Entity,
}

#[derive(Debug, new, SystemDesc)]
#[system_desc(name(HackAbilitySystemDesc))]
pub struct HackAbilitySystem {
    #[system_desc(event_channel_reader)]
    hack_selected_reader: ReaderId<HackSelectedEvent>,
}

impl<'s> System<'s> for HackAbilitySystem {
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
        WriteStorage<'s, HackAbility>,
        WriteStorage<'s, Parent>,
        ReadStorage<'s, CombatRoot>,
        WriteStorage<'s, Transform>,
        ReadStorage<'s, Team>,
        Read<'s, CharacterStore>,
        ReadExpect<'s, ParentHierarchy>,
        Write<'s, EventChannel<ShowHackUiEvent>>,
        WriteStorage<'s, SpawnAction>,
        Read<'s, EventChannel<HackSelectedEvent>>,
        Write<'s, EventChannel<HackPerformedEvent>>,
    );

    fn setup(&mut self, world: &mut World) {
        world.fetch_mut::<AbilityList>().register(HackAbility::data(), AbilityUsability::Role(CharacterRole::Master));
    }

    fn run(&mut self, (entities, mut principals, mut slot_managers, mut characters, mut unassigned_characters, mut character_prefabs, mut spawn_processes, mut abilities, mut unassigned_abilities, mut ability_invokes, mut performs, mut hack_abilities, mut parents, combat_roots, mut transforms, teams, character_store, hierarchy, mut select_hack_events, mut spawn_actions, hack_selected_event, mut hack_performed_event): Self::SystemData) {
        let mut to_remove: Vec<Entity> = Vec::new();
        for (entity, ability, _) in (&entities, &abilities, &unassigned_abilities).join() {
            if ability.data.id == TypeId::of::<HackAbility>() {
                hack_abilities.insert(entity, HackAbility::default());
                to_remove.push(entity);
            }
        }

        for ent in to_remove {
            unassigned_abilities.remove(ent);
        }

        for (ent, mut ability, _) in (&entities, &mut abilities, hack_abilities.mask()).join() {
            if let Some((team, _)) = Team::get_team(&parents, &teams, ent) {
                if let Some((slot_manager, _)) = get_root::<SlotManager, _, _>(&parents, &slot_managers, ent) {
                    if slot_manager.for_team(team).is_full() {
                        ability.locked = true;
                    } else {
                        ability.locked = false;
                    }
                }
            }
        }

        // Check if the ability has been triggered.
        for (ent, perform, ability, mut hack_ability) in (&entities, &performs, &abilities, &mut hack_abilities).join() {
            if !hack_ability.displaying {
                if let AbilityTarget::Single(target_ent) = perform.target {
                    if let Some(target) = characters.get(target_ent) {
                        let mut exponent: f32 = target.relative_health();
                        if exponent < 0.1 {
                            exponent = 0.1;
                        }
                        let chance: f32 = BASE_CHANCE.powf(exponent);
                        let mut n: i32 = (1.0 / chance) as i32;
                        if n > MAX_OPTIONS {
                            n = MAX_OPTIONS;
                        }
                        let options: Vec<UiHackOption> = vec![
                            UiHackOption {
                                chance,
                                charge: 100.0,
                                level: "Basic Hack".to_string(),
                            },
                            UiHackOption {
                                chance: chance.powf(0.7),
                                charge: 300.0,
                                level: "Advanced Hack".to_string(),
                            },
                            UiHackOption {
                                chance: chance.powf(0.5),
                                charge: 500.0,
                                level: "Elite Hack".to_string(),
                            },
                            UiHackOption {
                                chance: chance.powf(0.3),
                                charge: 700.0,
                                level: "Legendary Hack".to_string(),
                            },
                        ];
                        select_hack_events.single_write(ShowHackUiEvent {
                            owner: ent,
                            target: target_ent,
                            options,
                        });
                    }
                }
                hack_ability.displaying = true;
            }
        }

        for selection_evt in hack_selected_event.read(&mut self.hack_selected_reader) {
            if performs.contains(selection_evt.data.owner) {
                let ability_ent = selection_evt.data.owner;
                if let Some(charge) = selection_evt.charge {
                    if let Some((_, character_ent)) = get_root::<Character, _, _>(&parents, &characters, selection_evt.data.owner) {
                        if let Some((team, _)) = Team::get_team(&parents, &teams, selection_evt.data.owner) {
                            if Character::try_take_turn(&mut characters, character_ent, charge) {
                                if selection_evt.succeeded {
                                    hack_performed_event.single_write(
                                        HackPerformedEvent {
                                            target_ent: selection_evt.data.target,
                                        }
                                    );
                                    Character::switch_team(
                                        &entities,
                                        &mut parents,
                                        &combat_roots,
                                        &teams,
                                        &mut transforms,
                                        &mut slot_managers,
                                        &hierarchy,
                                        selection_evt.data.target,
                                        team,
                                    );
                                    if let Some(character) = characters.get_mut(selection_evt.data.target) {
                                        character.restore();
                                    }
                                }
                            } else {
                                panic!("Failed to take turn!");
                            }
                        }
                    }
                }
                performs.remove(ability_ent);
                ability_invokes.remove(ability_ent);
                if let Some(hack_ability) = hack_abilities.get_mut(ability_ent) {
                    hack_ability.displaying = false;
                }
                Principal::try_root_disengage(
                    &parents,
                    &mut principals,
                    ability_ent,
                    TypeId::of::<Self>(),
                );
            }
        }
    }
}