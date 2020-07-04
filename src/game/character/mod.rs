use std::any::{
    Any, TypeId,
};
use std::collections::HashMap;
use std::path::Path;

use amethyst::{
    assets::{
        AssetPrefab,
        Handle,
        Prefab,
        PrefabData,
        PrefabLoader,
        ProgressCounter,
        RonFormat,
    },
    core::{
        SystemBundle,
        math::{
            Unit,
            Vector3,
        },
        SystemDesc,
        ParentHierarchy,
        Parent,
        Transform,
    },
    ecs::{
        prelude::*,
        storage::{
            GenericReadStorage,
            GenericWriteStorage,
        },
    },
    Error,
    gltf::{
        GltfSceneAsset,
        GltfSceneFormat,
    },
    renderer::light::LightPrefab,
};
use crate::amethyst::derive;
use crate::core::{get_root, get_root_mut, roll};
use crate::game::combat::{ability::{
    Ability,
    AbilityData,
    AbilityId,
    AbilityList,
    AbilityUsability,
    Element,
}, CombatRoot, status::{
    StatusEffect,
    StatusType,
}, Team, Rank};
use crate::game::combat::ability::{UnassignedAbility, StatusInflictDesc};
use crate::game::combat::process::Principal;
use crate::game::combat::spawn::{SlotManager, Slots, SpawnAction, SpawnProcess, SpawnSystem};

pub use {
    spacebot::{
        SpacebotDrone,
        GunnerSpacebotDrone,
        ChargeSpacebotDrone,
        SupporterSpacebotDrone,
        ModelXDrone,
    },
    master::MasterDrone,
    spangles::SpanglesDrone,
    sparky::SparkyDrone,
    blitz::BlitzDrone,
    defender::DefenderDrone,
    guardian::GuardianDrone,
    earth::EarthCharacter,
};

macro_rules! define_character {
    (
        $C:ident,
        $S:ident,
        $SD:ident,
        $prefab_path:literal,
        $name:literal,
        $description:literal,
        $max_charge:literal,
        $max_health:literal,
        $initial_charge:literal,
        $natural_charge:literal,
        $artificial_charge:literal,
        $hack_modifier:expr,
        $role:expr,
        $allegiance:expr,
        $turns:literal,
        $crosshair_scale:literal,
    ) => {
        #[derive(Debug, Copy, Clone, Default, Component)]
        pub struct $C;

        impl $C {
            pub fn character_id() -> CharacterId {
                std::any::TypeId::of::<Self>()
            }

            pub fn data() -> CharacterData {
                CharacterData {
                    role: $role,
                    max_charge: $max_charge,
                    max_health: $max_health,
                    initial_charge: $initial_charge,
                    natural_charge: $natural_charge,
                    artificial_charge: $artificial_charge,
                    turns: $turns,
                    attack: Stats::default(),
                    name: $name,
                    description: $description,
                    resistance: Stats::default(),
                    hack_modifier: $hack_modifier,
                    allegiance: $allegiance,
                    crosshair_scale: $crosshair_scale,
                    base_dmg: 1.0,
                    base_accuracy: 1.0,
                    base_evade: 1.0,
                }
            }
        }

         #[derive(Debug, Copy, Clone, Default, new, SystemDesc)]
        #[system_desc(name($SD))]
        pub struct $S;

        impl<'s> System<'s> for $S {
            type SystemData = (
                Entities<'s>,
                WriteStorage<'s, UnassignedCharacter>,
                WriteStorage<'s, $C>,
            );

            fn setup(&mut self, world: &mut World) {
                CharacterStore::register(world, $C::character_id(), $C::data(), $prefab_path);
            }

            fn run(&mut self, (entities, mut unassigned_characters, mut unique_drones): Self::SystemData) {
                let mut to_assign: Vec<Entity> = Vec::new();
                for (entity, unassigned) in (&entities, &unassigned_characters).join() {
                    if unassigned.id() == TypeId::of::<$C>() {
                        to_assign.push(entity);
                    }
                }
                for entity in to_assign {
                    unassigned_characters.remove(entity);
                    /// Set up our drone
                    unique_drones.insert(entity, $C::default());
                }
            }
        }
    }
}

pub mod master;
pub mod spangles;
pub mod sparky;
pub mod spacebot;
pub mod defender;
pub mod blitz;
pub mod guardian;
pub mod earth;


pub struct CharacterBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for CharacterBundle {
    fn build(
        self,
        world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {

        // Controller systems.
        builder.add(
            master::MasterDroneSystemDesc::default()
            .build(world),
        "master_drone",
        &[],
        );
        builder.add(
            spangles::SpanglesDroneSystemDesc::default()
                .build(world),
            "spangles_drone_system",
            &[],
        );
        builder.add(
            defender::DefenderDroneSystemDesc::default()
                .build(world),
            "defender_drone",
            &[],
        );
        builder.add(
            blitz::BlitzDroneSystemDesc::default()
                .build(world),
            "blitz_drone",
            &[],
        );
        builder.add(
            sparky::SparkyDroneSystemDesc::default()
                .build(world),
            "sparky_drone",
            &[],
        );

        builder.add(
            spacebot::SpacebotDroneSystemDesc::default()
                .build(world),
            "spacebot_drone",
            &[],
        );
        builder.add(
            spacebot::GunnerSpacebotDroneSystemDesc::default()
                .build(world),
            "gunner_spacebot_drone",
            &[],
        );
        builder.add(
            spacebot::SupporterSpacebotDroneSystemDesc::default()
                .build(world),
            "supporter_spacebot_drone",
            &[],
        );
        builder.add(
            spacebot::ChargeSpacebotDroneSystemDesc::default()
                .build(world),
            "charge_spacebot_drone",
            &[],
        );
        builder.add(
            spacebot::ModelXDroneSystemDesc::default()
                .build(world),
            "model_x_drone",
            &[],
        );
        builder.add(
            guardian::GuardianDroneSystemDesc::default()
                .build(world),
            "guardian_drone",
            &[],
        );
        builder.add(
            earth::EarthCharacterSystemDesc::default()
                .build(world),
            "earth_character",
            &[],
        );

        Ok(())
    }
}

pub type CharacterId = TypeId;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Component, Serialize, Deserialize, PrefabData)]
#[prefab(Component)]
pub struct WeaponSlot(pub usize);

impl WeaponSlot {
    pub fn index(&self) -> usize {
        self.0
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CharacterRole {
    Master,
    Slave,
    Independent, // Cannot be spawned, but is not a boss. (e.g. ModelX)
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Stats {
    // Elemental
    pub kinetic: f32,
    pub plasma: f32,
    pub ion: f32,
    pub quantum: f32,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            kinetic: 1.0,
            plasma: 1.0,
            ion: 1.0,
            quantum: 1.0,
        }
    }
}

impl Stats {
    pub fn element(&self, element: Element) -> f32 {
        match element {
            Element::Kinetic => self.kinetic,
            Element::Plasma => self.plasma,
            Element::Ion => self.ion,
            Element::Quantum => self.quantum,
        }
    }

    pub fn element_inflict(&self, element: Element) -> f32 {
        self.element(element)
    }

    pub fn element_receive(&self, element: Element) -> f32 {
         self.element(element)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Statuses {
    pub map: HashMap<StatusType, usize>,
}

impl Default for Statuses {
    fn default() -> Self {
        let statuses: Vec<StatusType> = StatusType::all();
        let mut map: HashMap<StatusType, usize> = HashMap::with_capacity(statuses.len());
        for status in statuses {
            map.insert(status, 0);
        }
        Self { map }
    }
}

impl Statuses {
    pub fn get(&self, status: StatusType) -> usize {
        *self.map.get(&status).unwrap()
    }

    pub fn append(&mut self, status: StatusType, count: usize) {
        *self.map.get_mut(&status).unwrap() += count;
    }

    pub fn decrement(&mut self, status: StatusType) {
        let status = self.map.get_mut(&status).unwrap();
        if *status != 0 {
            *status -= 1
        }
    }

    pub fn take(&mut self, status: StatusType) -> usize {
        let status = self.map.get_mut(&status).unwrap();
        let ret = *status;
        *status = 0;
        ret
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CharacterData {
    // Do not make owned string so that we can implement copy.
    pub name: &'static str,
    pub description: &'static str,

    pub resistance: Stats,
    pub turns: i32,
    pub attack: Stats,
    pub role: CharacterRole,
    pub max_health: f32,
    pub max_charge: f32,
    pub initial_charge: f32,
    pub natural_charge: f32,
    pub artificial_charge: f32,
    pub base_dmg: f32,
    pub base_accuracy: f32,
    pub base_evade: f32,
    pub hack_modifier: Option<f32>,
    pub allegiance: Team,
    pub crosshair_scale: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CharacterState {
    pub health: f32,
    pub charge: f32,

    pub statuses: Statuses,
}

pub struct Character {
    data: CharacterData,
    state: CharacterState,
    id: CharacterId,
    rank: Rank,

    // The number of turns the character currently has. Usually one.
    turns: i32,
}

impl Character {
    pub fn state(&self) -> &CharacterState {
        &self.state
    }

    pub fn try_upgrade(&mut self) -> bool {
        self.rank.try_upgrade()
    }

    pub fn accuracy(&self) -> f32 {
        let mut accuracy = self.data.base_accuracy * self.rank.accuracy_multiplier();
        if self.has_status(StatusType::Unstable) {
            accuracy /= 1.5;
        }
        if self.has_status(StatusType::Focus) {
            accuracy *= 2.0;
        }
        accuracy
    }

    pub fn evade(&self) -> f32 {
        let mut evade = self.data.base_evade * self.rank.evade_multiplier();
        evade
    }

    pub fn crosshair_scale(&self) -> f32 {
        self.data.crosshair_scale
    }

    pub fn set_max_charge(&mut self, max_charge: f32) {
        self.data.max_charge = max_charge;
    }

    pub fn status(&self, ty: StatusType) -> usize {
        self.state.statuses.get(ty)
    }

    pub fn has_status(&self, ty: StatusType) -> bool {
        self.status(ty) > 0
    }

    pub fn append_status(&mut self, ty: StatusType, count: usize) {
        self.state.statuses.append(ty, count);
    }

    pub fn decrement_status(&mut self, ty: StatusType) {
        self.state.statuses.decrement(ty);
    }

    pub fn take_status(&mut self, ty: StatusType) -> usize {
        self.state.statuses.take(ty)
    }

    /// Returns current health as a proportion of max hp.
    pub fn relative_health(&self) -> f32 {
        self.state().health / self.max_health()
    }

    pub fn damage_multiplier(&self) -> f32 {
        self.data.base_dmg * self.rank.base_multiplier()
    }

    /// Returns current health as a proportion of max hp.
    pub fn relative_charge(&self) -> f32 {
        self.state().charge / self.max_charge()
    }

    pub fn reset(&mut self) {
        self.turns = self.data.turns;
    }

    pub fn restore(&mut self) {
        self.reset();
        self.add_charge(self.natural_charge());
    }

    pub fn charge(&self) -> f32 {
        self.state.charge
    }

    pub fn try_take_charge(&mut self, charge: f32) -> bool {
        if self.charge() - charge >= 0.0 {
            self.set_charge(self.charge() - charge);
            true
        } else {
            false
        }
    }

    pub fn set_charge(&mut self, charge: f32) {
        self.state.charge = charge;
    }

    pub fn set_health(&mut self, health: f32) {
        self.state.health = health;
    }

    pub fn health(&self) -> f32 {
        self.state.health
    }

    pub fn max_health(&self) -> f32 {
        self.rank.health_multiplier() * self.data.max_health
    }

    pub fn max_charge(&self) -> f32 {
        self.rank.charge_multiplier() * self.data.max_charge
    }

    pub fn natural_charge(&self) -> f32 {
        self.rank.charge_multiplier() * self.data.natural_charge
    }

    pub fn artificial_charge(&self) -> f32 {
        self.rank.charge_multiplier() * self.data.artificial_charge
    }

    pub fn hack_modifier(&self) -> Option<f32> {
        if let Some(hack_modifier) = self.data.hack_modifier {
            Some(self.rank.base_multiplier() * hack_modifier)
        } else {
            None
        }
    }

    pub fn turns(&self) -> i32 {
        if self.has_status(StatusType::Scramble) {
            0
        } else {
            self.turns
        }
    }

    pub fn has_turn(&self) -> bool {
        self.turns() > 0 || self.has_status(StatusType::Overclocked)
    }

    pub fn use_turns(&mut self) {
        self.take_status(StatusType::Overclocked);
        self.turns = 0;
    }

    pub fn try_decrement_turn(&mut self) -> bool {
        if !self.has_turn() {
            return false;
        }
        if self.has_status(StatusType::Overclocked) {
            self.decrement_status(StatusType::Overclocked);
            true
        } else if self.turns > 0 {
            self.turns -= 1;
            true
        } else {
            false
        }
    }

    pub fn role(&self) -> CharacterRole {
        self.data.role
    }

    pub fn name(&self) -> &'static str {
        self.data.name
    }

    pub fn description(&self) -> &'static str {
        self.data.description
    }

}

#[derive(Debug, Copy, Clone, PartialEq, Component)]
pub struct LastDamaged {
    pub entity: Option<Entity>,
    pub dmg: f32,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Error)]
pub enum CharacterSpawnError {
    #[error(display = "no principal in the hierarchy")]
    NoPrincipal,
    #[error(display = "principal engaged")]
    PrincipalEngaged,
    #[error(display = "no slots available for the character")]
    NoSlots,
    #[error(display = "the slots are occupied")]
    SlotOccupied,
    #[error(display = "character id is invalid")]
    InvalidCharacterId,
}

impl Character {
    /// The character entity specified should be a newly created entity which will be populated with the necessary character components.
    /// If the spawn fails, it is recommended that you destroy the character entity.
    pub fn spawn<'s>(
        parents: &impl GenericReadStorage<Component=Parent>,
        principals: &mut impl GenericWriteStorage<Component=Principal>,
        slot_managers: &mut impl GenericWriteStorage<Component=SlotManager>,
        characters: &mut impl GenericWriteStorage<Component=Character>,
        unassigned_characters: &mut impl GenericWriteStorage<Component=UnassignedCharacter>,
        character_prefabs: &mut impl GenericWriteStorage<Component=Handle<Prefab<CharacterPrefabData>>>,
        spawn_processes: &mut impl GenericWriteStorage<Component=SpawnProcess>,
        character_store: &CharacterStore,
        character_ent: Entity,
        character_id: CharacterId,
        character_data: Option<CharacterData>,
        rank: Rank,
        team: Team,
        slot_idx: usize,
        principal: bool,
    ) -> Result<(), CharacterSpawnError> {
        let mut engage_result: Option<bool> = {
            if principal {
                Principal::try_root_engage(
                    parents, principals,
                    character_ent, TypeId::of::<SpawnSystem>(),
                )
            } else {
                Some(true)
            }
        };

        // If we have successfully engaged as principal, we spawn the character.
        match engage_result {
            Some(true) => {
                // Spawn character...
                if let Some((slot_manager, _)) = get_root_mut::<SlotManager, _, _>(
                    parents, slot_managers,
                    character_ent,
                ) {
                    if let Some((data, prefab)) = character_store.characters.get(&character_id) {
                        let slots: &mut Slots = slot_manager.for_team_mut(team);
                        if slots.try_occupy(slot_idx, character_ent) {
                            let character: Character = {
                                if let Some(character_data) = character_data {
                                    Character::new(character_data, character_id, rank)
                                } else {
                                    Character::new(*data, character_id, rank)
                                }
                            };
                            let spawn_process: SpawnProcess = SpawnProcess {
                                speed: 5.0,
                                end: slots.slot_position(slot_idx),
                            };
                            characters.insert(character_ent, character);
                            unassigned_characters.insert(character_ent, UnassignedCharacter::new(character_id, Option::clone(prefab)));
                            spawn_processes.insert(character_ent, spawn_process);
                            if let Some(prefab_handle) = Option::clone(prefab) {
                                character_prefabs.insert(character_ent, prefab_handle);
                            }
                            Ok(())
                        } else {
                            Err(CharacterSpawnError::SlotOccupied)
                        }
                    } else {
                        Err(CharacterSpawnError::InvalidCharacterId)
                    }
                } else {
                    Err(CharacterSpawnError::NoSlots)
                }
            }
            Some(false) => Err(CharacterSpawnError::PrincipalEngaged),
            None => Err(CharacterSpawnError::NoPrincipal),
        }
    }

    pub fn spawn_to_world<'s>(world: &mut World, parent_ent: Entity, character_id: CharacterId, character_data: Option<CharacterData>, rank: Rank, team: Team, slot_idx: usize, principal: bool) {
        world.exec(|(entities, mut principals, mut parents, mut spawn_actions): (Entities, WriteStorage<Principal>, WriteStorage<Parent>, WriteStorage<SpawnAction>)| {
            Self::invoke_spawn(
                &entities,
                &mut principals,
                &mut parents,
                &mut spawn_actions,
                parent_ent,
                character_id,
                character_data,
                rank,
                team,
                slot_idx,
                principal,
            );
        });
    }

    pub fn invoke_spawn<'s>(
        entities: &Entities,
        principals: &mut impl GenericWriteStorage<Component=Principal>,
        parents: &mut (impl GenericReadStorage<Component=Parent> + GenericWriteStorage<Component=Parent>),
        spawn_actions: &mut impl GenericWriteStorage<Component=SpawnAction>,
        parent_ent: Entity,
        character_id: CharacterId,
        character_data: Option<CharacterData>,
        rank: Rank,
        team: Team, slot_idx: usize,
        principal: bool,
    ) -> Option<bool> {
        let entity = entities.create();
        parents.insert(entity, Parent { entity: parent_ent });
        if principal {
            let res = Principal::try_root_engage(
                parents,
                principals,
                entity,
                TypeId::of::<SpawnSystem>(),
            );

            if let Some(true) = res {} else {
                entities.delete(entity);
                return res;
            }
        }
        spawn_actions.insert(entity, SpawnAction {
            character_id,
            team,
            rank,
            character_data,
            parent: parent_ent,
            slot_idx,
        });
        Some(true)
    }

    pub fn try_take_turn(
        characters: &mut impl GenericWriteStorage<Component=Character>,
        character_ent: Entity,
        charge: f32,
    ) -> bool {
        if let Some(character) = characters.get_mut(character_ent) {
            if character.try_take_charge(charge) {
                character.try_decrement_turn()
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn can_take_turn(
        characters: &impl GenericReadStorage<Component=Character>,
        character_ent: Entity,
        charge: f32,
    ) -> bool {
        if let Some(character) = characters.get(character_ent) {
            if character.has_turn() {
                if character.state.charge - charge >= 0.0 {
                    true
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn switch_team<'s>(
        entities: &Entities<'s>,
        parents: &mut (impl GenericReadStorage<Component=Parent> + GenericWriteStorage<Component=Parent>),
        roots: &impl GenericReadStorage<Component=CombatRoot>,
        teams: &impl GenericReadStorage<Component=Team>,
        transforms: &mut impl GenericWriteStorage<Component=Transform>,
        slot_managers: &mut impl GenericWriteStorage<Component=SlotManager>,
        hierarchy: &ParentHierarchy,
        character_ent: Entity,
        target_team: Team,
    ) {
        if let Some(team_ent) = Team::get_local(
            entities,
            parents,
            roots,
            teams,
            hierarchy,
            target_team,
            character_ent,
        ) {
            if let Some((slot_manager, _)) = get_root_mut(parents, slot_managers, character_ent) {
                if let Some(slot_idx) = slot_manager.for_team(target_team).find_next(false) {
                    if let Some((team, _)) = Team::get_team(
                        parents,
                        teams,
                        character_ent,
                    ) {
                        slot_manager.for_team_mut(team).remove_entity(character_ent);
                    }
                    slot_manager.for_team_mut(target_team).occupy(slot_idx, character_ent);
                    transforms.insert(character_ent, Transform::from(slot_manager.for_team(target_team).slot_position(slot_idx)));
                }
            }
            parents.insert(character_ent, Parent { entity: team_ent });
        }
    }
}

impl Component for Character {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Debug, Clone)]
pub struct UnassignedCharacter {
    pub id: CharacterId,
    pub prefab: Option<Handle<Prefab<CharacterPrefabData>>>,
}

impl UnassignedCharacter {
    pub fn new(id: CharacterId, prefab: Option<Handle<Prefab<CharacterPrefabData>>>) -> Self {
        Self {
            id,
            prefab,
        }
    }
}

impl Component for UnassignedCharacter {
    type Storage = DenseVecStorage<Self>;
}

impl UnassignedCharacter {
    #[inline]
    pub fn id(&self) -> CharacterId {
        self.id
    }

    #[inline]
    pub fn prefab(&self) -> Option<&Handle<Prefab<CharacterPrefabData>>> {
        self.prefab.as_ref()
    }

    #[inline]
    pub fn prefab_owned(self) -> Option<Handle<Prefab<CharacterPrefabData>>> {
        self.prefab
    }
}

impl Character {
    pub fn new(data: CharacterData, id: CharacterId, rank: Rank) -> Self {
        Self {
            data,
            state: CharacterState {
                health: data.max_health * rank.health_multiplier(),
                charge: data.initial_charge * rank.charge_multiplier(),
                statuses: Statuses::default(),
            },
            // Same as `TypeId::of<C>()`
            id,
            // Start with 1 turn by default.
            turns: 1,
            rank,
        }
    }

    #[inline]
    pub fn id(&self) -> CharacterId {
        self.id
    }

    #[inline]
    pub fn rank(&self) -> Rank {
        self.rank
    }

    pub fn dmg_output(&self, dmg: f32, element: Element) -> f32 {
        let element_multiplier: f32 = self.data.attack.element_inflict(element);
        let mut output: f32 = dmg * element_multiplier * self.damage_multiplier();
        if self.has_status(StatusType::Unstable) {
            output *= 1.5;
        }
        if self.has_status(StatusType::Empower) {
            output *= 1.5;
        }
        output
    }

    pub fn dmg_receive(&self, dmg: f32, element: Element) -> f32 {
        let element_multiplier: f32 = self.data.attack.element_receive(element);
        let mut output: f32 = dmg * element_multiplier;
        if self.has_status(StatusType::Defend) {
            output *= 0.5;
        }
        output
    }

    pub fn add_charge(&mut self, charge: f32) -> f32 {
        if self.state.charge == self.max_charge() && charge > 0.0 {
            return 0.0;
        }
        self.state.charge += charge;
        if self.state.charge > self.max_charge() {
            let delta_overflow = self.state.charge - self.max_charge();
            self.state.charge = self.max_charge();
            charge - delta_overflow
        } else {
            charge
        }
    }

    pub fn change_health(&mut self, delta_health: f32) -> f32 {
        if self.state.health == self.max_health() && delta_health > 0.0 {
            return 0.0;
        }
        self.state.health += delta_health;
        if self.state.health > self.max_health() {
            let delta_overflow = self.state.health - self.max_health();
            self.state.health = self.max_health();
            delta_health - delta_overflow
        } else {
            delta_health
        }
    }

    /// If `available_abilities` is set to `None` then the character will have all it's abilities.
    pub fn populate_abilities<'s>(
        entities: &Entities<'s>,
        parents: &mut impl GenericWriteStorage<Component=Parent>,
        characters: &impl GenericReadStorage<Component=Character>,
        abilities: &mut impl GenericWriteStorage<Component=Ability>,
        unassigned_abilities: &mut impl GenericWriteStorage<Component=UnassignedAbility>,
        character_ent: Entity,
        list: &AbilityList,
        specific: Option<&[AbilityId]>,
    ) {
        if let Some(character) = characters.get(character_ent) {
            let ability_map: HashMap<AbilityId, AbilityData> = list.abilities_for(character);
            for (id, ability_data) in ability_map {
                Self::insert_ability(
                    entities,
                    parents,
                    abilities,
                    unassigned_abilities,
                    character_ent,
                    ability_data,
                )
            }
        }
    }

    pub fn insert_ability<'s>(
        entities: &Entities<'s>,
        parents: &mut impl GenericWriteStorage<Component=Parent>,
        abilities: &mut impl GenericWriteStorage<Component=Ability>,
        unassigned_abilities: &mut impl GenericWriteStorage<Component=UnassignedAbility>,
        character_ent: Entity,
        ability_data: AbilityData,
    ) {
        let ability: Ability = Ability::new(ability_data);
        let ability_ent: Entity = entities.create();
        abilities.insert(ability_ent, ability);
        unassigned_abilities.insert(ability_ent, UnassignedAbility(ability_data.id));
        parents.insert(ability_ent, Parent { entity: character_ent });
    }
    pub fn inflict_dmg_silent<'s>(
        characters: &mut (impl GenericReadStorage<Component=Character> + GenericWriteStorage<Component=Character>),
        source_ent: Option<Entity>, target_ent: Entity, dmg: f32, element: Element,
    ) -> Result<f32, InflictError> {
        let mut dmg_to_inflict: f32 = 0.0;
        if let Some(source_ent) = source_ent {
            if let Some(source) = characters.get(source_ent) {
                let output: f32 = source.dmg_output(dmg, element);
                if let Some(target) = characters.get(target_ent) {
                    dmg_to_inflict = target.dmg_receive(output, element);
                } else {
                    return Err(InflictError::InvalidTargetEntity);
                }
            } else {
                return Err(InflictError::InvalidSourceEntity);
            }
        } else {
            if let Some(target) = characters.get(target_ent) {
                dmg_to_inflict = target.dmg_receive(dmg, element);
            } else {
                return Err(InflictError::InvalidTargetEntity);
            }
        }
        if dmg_to_inflict != 0.0 {
            if let Some(target) = characters.get_mut(target_ent) {
                target.change_health(-dmg_to_inflict);
            }
        }
        Ok(dmg_to_inflict)
    }

    pub fn check_hit<'s>(
        characters: &impl GenericReadStorage<Component=Character>,
        target_ent: Entity, source_ent: Entity, accuracy: f32,
    ) -> Result<bool, InflictError> {
        if let Some(source) = characters.get(source_ent) {
            if let Some(target) = characters.get(target_ent) {
                let exponent = source.accuracy() / target.evade();
                let net = accuracy.powf(1.0 / exponent);
                Ok(roll(net))
            } else {
                Err(InflictError::InvalidTargetEntity)
            }
        } else {
            Err(InflictError::InvalidSourceEntity)
        }
    }

    pub fn inflict_status_silent<'s>(
        characters: &mut impl GenericWriteStorage<Component=Character>,
        target_ent: Entity, status_inflict: StatusInflictDesc,
    ) -> Result<Option<usize>, InflictError> {
        if let Some(target) = characters.get_mut(target_ent) {
            if roll(status_inflict.chance) {
                target.append_status(status_inflict.ty, status_inflict.turns);
                return Ok(Some(status_inflict.turns));
            }
        } else {
            return Err(InflictError::InvalidTargetEntity);
        }
        Ok(None)
    }

    pub fn get_lowest_hp(
        characters: &impl GenericReadStorage<Component=Character>,
        character_ents: &[Entity],
    ) -> Option<Entity> {
        let mut current: Option<(Entity, f32)> = None;
        for ent in character_ents {
            if let Some(character) = characters.get(*ent) {
                if let Some((current_ent, current_hp)) = current.as_mut() {
                    if character.relative_health() < *current_hp {
                        *current_ent = *ent;
                        *current_hp = character.relative_health();
                    }
                } else {
                    current = Some((*ent, character.relative_health()));
                }
            }
        }
        if let Some((ent, _)) = current {
            Some(ent)
        } else {
            None
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Component)]
pub struct Defeated {
    pub drift: Vector3<f32>,
    pub rotation_axis: Unit<Vector3<f32>>,
    pub rotation_speed: f32,
    pub time: f32,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CharacterDefeatedEvent {
    pub character_ent: Entity,
    pub splash_dmg: f32,
    pub killer: Option<Entity>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum InflictError {
    InvalidSourceEntity,
    InvalidTargetEntity,
}

#[derive(Debug, Default, Clone)]
pub struct CharacterStore {
    pub characters: HashMap<CharacterId, (CharacterData, Option<Handle<Prefab<CharacterPrefabData>>>)>,
}

impl CharacterStore {
    pub fn register<P: Into<String>>(world: &mut World, character_id: CharacterId, data: CharacterData, prefab_path: P) {

        let handle = world.exec(
            |loader: PrefabLoader<'_, CharacterPrefabData>| {
                loader.load(
                    prefab_path,
                    RonFormat,
                    (),
                )
            },
        );
        world.write_resource::<Self>().characters.insert(character_id, (data, Some(handle)));
    }

    pub fn get_spawnable(&self, team: Team) -> Vec<(CharacterId, CharacterData)> {
        let mut res: Vec<(CharacterId, CharacterData)> = Vec::new();
        for (k, (character_data, prefab)) in self.characters.iter() {
            if character_data.role == CharacterRole::Slave && character_data.allegiance == team {
                res.push((*k, *character_data));
            }
        }
        res
    }

    pub fn prefab(&self, id: &CharacterId) -> Option<Handle<Prefab<CharacterPrefabData>>> {
        if let Some((_, prefab)) = self.characters.get(id) {
            prefab.clone()
        } else {
            None
        }
    }
}

#[derive(Default, Deserialize, Serialize, PrefabData)]
#[serde(default)]
pub struct CharacterPrefabData {
    transform: Option<Transform>,
    gltf: Option<AssetPrefab<GltfSceneAsset, GltfSceneFormat>>,
    light: Option<LightPrefab>,
    weapon_slot: Option<WeaponSlot>,
    // TODO: Add physics data.
}