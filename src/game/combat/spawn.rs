use std::any::{Any, TypeId};

use amethyst::{
    assets::{
        Handle,
        Prefab,
    },
    core::{
        math::Vector3,
        Parent,
        Time,
        transform::Transform,
    },
    ecs::prelude::*,
    prelude::SystemDesc,
    shrev::{
        EventChannel,
        ReaderId,
    },
};

use crate::{
    core::Action,
    game::character::{
        Character,
        CharacterId,
        CharacterPrefabData,
        master::MasterDrone,
    },
};
use crate::core::activity::{Activity, ActivityAggregator, ActivityState};
use crate::game::character::{CharacterStore, UnassignedCharacter, CharacterData};
use crate::game::combat::ability::{Ability, AbilityList, UnassignedAbility};
use crate::game::combat::process::Principal;
use crate::game::combat::{Team, Rank};
use crate::game::ui::crosshair::UiCrosshair;
use crate::game::ui::status::UiStatus;

pub const ENV_SPAWN_TIME: f32 = 2.5;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CharacterSpawnedEvent {
    pub character_ent: Entity,
    pub action: SpawnAction,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SpawnSource {
    Enemy,
    Friendly,
    Master(Entity),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SpawnProcess {
    pub speed: f32,
    pub end: Vector3<f32>,
}

impl Component for SpawnProcess {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Debug, Copy, Clone, Default)]
pub struct Slots {
    occupied: [Option<Entity>; 7],
    max: usize,
}

impl Slots {
    // The master drone slot.
    pub const MASTER: usize = 0;

    // The primary slave drone slots - only these slots can be occupied by player drones.
    pub const PRIMARY_1: usize = 1;
    pub const PRIMARY_2: usize = 2;
    pub const PRIMARY_3: usize = 3;
    pub const PRIMARY_4: usize = 4;

    // These slots are additional slots that can be used by enemies, or filled when an enemy drone is hacked.
    pub const SECONDARY_1: usize = 5;
    pub const SECONDARY_2: usize = 6;
    pub const SECONDARY_3: usize = 7;

    pub fn new() -> Self {
        Self {
            occupied: [None; 7],
            max: 7,
        }
    }

    pub fn with_max(max: usize) -> Self {
        debug_assert!(max <= 7 && max > 0);
        Self {
            occupied: [None; 7],
            max,
        }
    }

    pub fn character(&self, idx: usize) -> Option<Entity> {
        self.occupied[idx]
    }

    pub fn is_occupied(&self, idx: usize) -> bool {
        debug_assert!(idx < self.max);
        self.occupied[idx].is_some()
    }

    pub fn occupied(&self) -> &[Option<Entity>] {
        &self.occupied
    }
    /// Cannot occupy a slot with an index greater than max - 1.
    pub fn occupy(&mut self, idx: usize, character_ent: Entity) -> Option<Entity> {
        debug_assert!(idx < self.max);
        let current = self.occupied[idx];
        self.occupied[idx] = Some(character_ent);
        current
    }

    /// Only occupy if vacant - returns true in this case.
    /// Return false if occupied.
    pub fn try_occupy(&mut self, idx: usize, character_ent: Entity) -> bool {
        if !self.is_occupied(idx) {
            self.occupy(idx, character_ent);
            true
        } else {
            false
        }
    }

    /// Returns the next available slot index.
    pub fn find_next(&self, fill_master: bool) -> Option<usize> {
        let start: usize = {
            if fill_master {
                0
            } else {
                1
            }
        };
        for i in start..self.max {
            if !self.occupied[i].is_some() {
                return Some(i);
            }
        }
        None
    }

    pub fn remove_entity(&mut self, entity: Entity) -> bool {
        for occupied in self.occupied.iter_mut() {
            if let Some(ent) = occupied {
                if *ent == entity {
                    *occupied = None;
                    return true;
                }
            }
        }
        false
    }

    pub fn count(&self) -> usize {
        let mut count: usize = 0;
        for occupied in self.occupied() {
            if let Some(_) = occupied {
                count += 1;
            }
        }
        count
    }

    pub fn is_full(&self) -> bool {
        for opt in self.occupied() {
            if opt.is_none() {
                return false;
            }
        }
        true
    }

    pub fn is_empty(&self) -> bool {
        self.occupied == [None; 7]
    }

    pub fn master(&self) -> Option<Entity> {
        self.character(0)
    }

    pub fn slave(&self, idx: usize) -> Option<Entity> {
        self.character(idx + 1)
    }

    /// The team local position of the slot.
    pub fn slot_position(&self, idx: usize) -> Vector3<f32> {
        match idx {
            0 => Vector3::new(0.0, 0.0, 0.0),

            1 => Vector3::new(16.0, 6.0, -6.0),
            2 => Vector3::new(-16.0, 6.0, -6.0),
            3 => Vector3::new(16.0, -6.0, -6.0),
            4 => Vector3::new(-16.0, -6.0, -6.0),

            5 => Vector3::new(30.0, 0.0, -16.0),
            6 => Vector3::new(-30.0, 0.0, -16.0),
            7 => Vector3::new(0.0, 12.0, -16.0),

            _ => panic!("SLOT INDEX OUT OF RANGE!"),
        }
    }

    pub fn iter(&self) -> SlotIterator {
        SlotIterator {
            counter: 0,
            slots: self,
        }
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct SlotManager {
    pub friendly: Slots,
    pub enemy: Slots,
}

impl SlotManager {
    pub fn new() -> Self {
        Self {
            friendly: Slots::new(),
            enemy: Slots::new(),
        }
    }
    pub fn for_team(&self, team: Team) -> &Slots {
        match team {
            Team::Friendly => &self.friendly,
            Team::Enemy => &self.enemy,
        }
    }

    pub fn for_team_mut(&mut self, team: Team) -> &mut Slots {
        match team {
            Team::Friendly => &mut self.friendly,
            Team::Enemy => &mut self.enemy,
        }
    }

    pub fn remove_entity(&mut self, entity: Entity) -> bool {
        for friendly in self.friendly.occupied.iter_mut() {
            if *friendly == Some(entity) {
                *friendly = None;
                return true;
            }
        }
        for enemy in self.enemy.occupied.iter_mut() {
            if *enemy == Some(entity) {
                *enemy = None;
                return true;
            }
        }
        false
    }
}

impl Component for SlotManager {
    type Storage = DenseVecStorage<Self>;
}

pub struct SlotIterator<'a> {
    counter: usize,
    slots: &'a Slots,
}

impl<'a> Iterator for SlotIterator<'a> {
    type Item = (usize, Entity);
    fn next(&mut self) -> Option<Self::Item> {
        while self.counter < 7 {
            if let Some(entity) = self.slots.occupied[self.counter] {
                self.counter += 1;
                return Some((self.counter - 1, entity));
            } else {
                self.counter += 1;
            }
        }
        None
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SpawnAction {
    pub slot_idx: usize,
    pub team: Team,
    pub rank: Rank,
    pub character_id: CharacterId,
    pub character_data: Option<CharacterData>,
    pub parent: Entity,
}

impl Component for SpawnAction {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Debug, Copy, Clone, Default, SystemDesc)]
#[system_desc(name(SpawnInvokeSystemDesc))]
pub struct SpawnInvokeSystem;

impl<'s> System<'s> for SpawnInvokeSystem {
    type SystemData = (
        Entities<'s>,
        Read<'s, Time>,
        WriteStorage<'s, Principal>,
        WriteStorage<'s, Parent>,
        WriteStorage<'s, Character>,
        WriteStorage<'s, UnassignedCharacter>,
        WriteStorage<'s, Handle<Prefab<CharacterPrefabData>>>,
        WriteStorage<'s, Team>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, SlotManager>,
        WriteStorage<'s, SpawnProcess>,
        WriteStorage<'s, SpawnAction>,
        Read<'s, CharacterStore>,
        WriteStorage<'s, Ability>,
        WriteStorage<'s, UnassignedAbility>,
        Read<'s, AbilityList>,
        Write<'s, EventChannel<CharacterSpawnedEvent>>,
    );

    fn run(&mut self, (entities, time, mut principals, mut parents, mut characters, mut unassigned_characters, mut character_prefabs, mut teams, mut transforms, mut slot_managers, mut spawn_processes, mut spawn_actions, character_store, mut abilities, mut unassigned_abilities, ability_list, mut spawn_events): Self::SystemData) {
        let mut to_remove: Vec<(Entity, SpawnAction)> = Vec::new();
        for (entity, action) in (&entities, &spawn_actions).join() {
            to_remove.push((entity, *action));
        }

        for (entity, action) in to_remove {
            let principal: bool = {
                if Principal::try_root_disengage(&parents, &mut principals, entity, TypeId::of::<Self>()) == Some(true) {
                    true
                } else {
                    false
                }
            };
            spawn_actions.remove(entity);

            let new_character: Entity = entity;

            match Character::spawn(
                &parents,
                &mut principals,
                &mut slot_managers,
                &mut characters,
                &mut unassigned_characters,
                &mut character_prefabs,
                &mut spawn_processes,
                &character_store,
                new_character, action.character_id, action.character_data, action.rank, action.team, action.slot_idx,
                principal,
            ) {
                Ok(_) => {
                    //populate character's abilities.
                    Character::populate_abilities(
                        &entities,
                        &mut parents,
                        &mut characters,
                        &mut abilities,
                        &mut unassigned_abilities,
                        new_character,
                        &ability_list,
                        None,
                    );

                    let character_name = characters.get(new_character).unwrap().name();

                    // Send the successful character spawn event.
                    spawn_events.single_write(
                        CharacterSpawnedEvent {
                            character_ent: new_character,
                            action,
                        }
                    );
                },
                Err(err) => {
                    println!("Failed to spawn character: {}", err);
                    parents.remove(new_character);
                    entities.delete(new_character);
                },
            }
        }
    }
}

#[derive(Debug, Copy, Clone, Default, SystemDesc)]
#[system_desc(name(SpawnSystemDesc))]
pub struct SpawnSystem;

impl<'s> System<'s> for SpawnSystem {
    type SystemData = (
        Entities<'s>,
        Read<'s, Time>,
        WriteStorage<'s, Principal>,
        ReadStorage<'s, Parent>,
        WriteStorage<'s, Character>,
        WriteStorage<'s, Team>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, SpawnProcess>,
    );

    fn run(&mut self, (entities, time, mut principals, parents, mut characters, mut teams, mut transforms, mut spawn_processes): Self::SystemData) {
        let mut to_remove: Vec<Entity> = Vec::new();
        for (entity, character, transform, mut spawn_process) in (&entities, &characters, &mut transforms, &mut spawn_processes).join() {
            let target: Vector3<f32> = spawn_process.end;
            let current: Vector3<f32> = *transform.translation();
            let delta: Vector3<f32> = target - current;
            let abs: f32 = delta.norm();
            let v: f32 = delta.norm() * spawn_process.speed + 3.0;
            let adj: f32 = v * time.delta_seconds();
            if adj >= abs {
                transform.set_translation(target);
                // We've arrived at the target point.

                to_remove.push(entity);
            } else {
                transform.prepend_translation(delta.normalize() * adj);
            }
        }

        for entity in to_remove {
            spawn_processes.remove(entity);
            // REMOVE FROM PRINCIPAL
            Principal::try_root_disengage(
                &parents, &mut principals,
                entity, TypeId::of::<Self>(),
            );
        }
    }
}

