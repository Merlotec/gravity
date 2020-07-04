use std::any::TypeId;
use std::collections::HashMap;

use amethyst::{
    core::Parent,
    ecs::{
        prelude::*,
        storage::{
            GenericReadStorage,
        }
    },
};
use rand::Rng;

use crate::core::{
    get_root,
    select_rng,
};
use crate::game::character::{
    Character,
    CharacterId,
    CharacterRole,
};
use crate::game::combat::{CombatRoot, Team};
use crate::game::combat::process::Principal;
use crate::game::combat::status::StatusType;
use crate::game::combat::tactical::AiAbilitySelection;
use crate::game::combat::tactical::AiAbilitySelectionQuery;
use std::ops::Range;
use crate::game::combat::spawn::SlotManager;

macro_rules! define_ability {
    (
        $C:ident,
        $S:ident,
        $SD:ident,
        $name:literal,
        $description:literal,
        $charge:literal,
        $cooldown:literal,
        $target_ty:expr,
        $target_area:expr,
        $usability:expr,
        [ $( $desc:expr ),* ]
    ) => {
        #[derive(Debug, Copy, Clone, Default, Component)]
        pub struct $C;

        #[derive(Debug, Copy, Clone, Default, new, SystemDesc)]
        #[system_desc(name($SD))]
        pub struct $S;

        impl $C {
            pub fn data() -> AbilityData {
                AbilityData {
                    name: $name,
                    desc: $description,
                    id: TypeId::of::<Self>(),
                    system: TypeId::of::<$S>(),
                    charge: AbilityCharge::Static($charge),
                    target_info: Some(AbilityTargetInfo {
                        ty: $target_ty,
                        area: $target_area,
                    }),
                    cooldown: $cooldown,
                }
            }
        }

        impl<'s> System<'s> for $S {
            type SystemData = (
                Entities<'s>,
                WriteStorage<'s, Principal>,
                WriteStorage<'s, CombatRoot>,
                ReadStorage<'s, Team>,
                ReadStorage<'s, SlotManager>,
                WriteStorage<'s, amethyst::core::Parent>,
                WriteStorage<'s, Character>,
                ReadStorage<'s, Ability>,
                WriteStorage<'s, AbilityPerform>,
                WriteStorage<'s, AbilityProgression>,
                WriteStorage<'s, AiAbilitySelectionQuery>,
                WriteStorage<'s, UnassignedAbility>,
                WriteStorage<'s, $C>,
                Read<'s, amethyst::core::Time>,
                Write<'s, amethyst::shrev::EventChannel<FireBulletEvent>>,
                Write<'s, amethyst::shrev::EventChannel<FireTorpedoEvent>>,
                Write<'s, amethyst::shrev::EventChannel<FireLaserEvent>>,
                Write<'s, amethyst::shrev::EventChannel<FireWaveEvent>>,
                Write<'s, amethyst::shrev::EventChannel<HealEvent>>,
                Write<'s, amethyst::shrev::EventChannel<ChargeEvent>>,
            );

            fn setup(&mut self, world: &mut World) {
                world.fetch_mut::<AbilityList>().register($C::data(), $usability);
            }
            fn run(&mut self, (entities, mut principals, combat_roots, teams, slot_managers, mut parents, mut characters, abilities, mut performs, mut progressions, mut ability_selections, mut unassigned, mut ability_components, time, mut fire_bullet_events, mut fire_torpedo_events, mut fire_laser_events, mut fire_wave_events, mut heal_events, mut charge_events): Self::SystemData) {
                for (entity, ability, _, mut ability_selection) in (&entities, &abilities, ability_components.mask(), &mut ability_selections).join() {
                    if ability_selection.result.is_none() {
                        let target: AbilityTarget = {
                            let target_info: AbilityTargetInfo = ability.data.target_info.expect("No target info for macro ability!");
                            if target_info.area == AbilityTargetArea::Single {
                                let mut chances: Vec<f32> = Vec::new();
                                let mut targets: Vec<Entity> = Vec::new();
                                for target in ability_selection.targets.iter() {
                                    if let Some(character) = characters.get(*target) {
                                        chances.push(2.0 - character.relative_health());
                                        targets.push(*target);
                                    }
                                }
                                AbilityTarget::Single(targets[select_rng(&chances).expect("Failed to get target for ability!")])
                            } else if target_info.area == AbilityTargetArea::Flexible {
                                let mut chances: Vec<f32> = Vec::new();
                                let mut targets: Vec<Entity> = Vec::new();
                                let mut multi: bool = false;
                                for target in ability_selection.targets.iter() {
                                    if let Some(character) = characters.get(*target) {
                                        if (character.health() < 10.0) {
                                            multi = true;
                                        } else {
                                            chances.push(2.0 - character.relative_health());
                                            targets.push(*target);
                                        }
                                    }
                                }
                                if multi {
                                    AbilityTarget::Multi(ability_selection.targets.clone())
                                } else {
                                    AbilityTarget::Single(targets[select_rng(&chances).expect("Failed to get target for ability!")])
                                }
                            } else {
                                AbilityTarget::Multi(ability_selection.targets.clone())
                            }
                        };
                        // The higher the charge, the greater the likelihood.
                        let score: f32 = (ability.data.charge.rated_charge() * 0.3).powf(0.3);
                        ability_selection.result = Some(
                            AiAbilitySelection {
                                score,
                                target,
                            }
                        );
                    }
                }
                for (entity, ability, _) in (&entities, &abilities, unassigned.mask().clone()).join() {
                    if ability.data.id == TypeId::of::<$C>() {
                        ability_components.insert(entity, $C::default());
                        unassigned.remove(entity);
                    }
                }

                for (entity, ability, _, _, _) in (&entities, &abilities, &ability_components, &performs, !progressions.mask().clone()).join() {
                    if let Some((_, character_ent)) = get_root::<Character, _, _>(&parents, &characters, entity) {
                        if Character::try_take_turn(&mut characters, character_ent, ability.data.charge.rated_charge()) {
                            progressions.insert(entity, AbilityProgression::new());
                        } else {
                            panic!("[define_ability!] Unexpected failure to take turn.");
                        }
                    }
                }

                let mut to_remove: Vec<Entity> = Vec::new();
                for (entity, _, perform, mut progression) in (&entities, &ability_components, &performs, &mut progressions).join() {
                    progression.time += time.delta_seconds();
                    let ability_actions = [$($desc,)*];
                    if let Some((character, character_ent)) = crate::core::get_root::<Character, _, _>(&parents, &characters, entity) {
                        loop {
                            if progression.stage < ability_actions.len() {
                                let desc: AbilityActionDesc = ability_actions[progression.stage].clone();
                                match desc {
                                    AbilityActionDesc::Fire(fire_desc, weapon_idx) => {
                                        match fire_desc.ty {
                                            FireType::Bullet => {
                                                fire_bullet_events.single_write(
                                                    FireBulletEvent {
                                                        source: character_ent,
                                                        weapon_idx,
                                                        target: perform.target.select_uniform(progression.fire_count),
                                                        accuracy: fire_desc.accuracy,
                                                        effect: fire_desc.effect,
                                                        power: fire_desc.power,
                                                    }
                                                );
                                            },
                                            FireType::Torpedo(element) => {
                                                fire_torpedo_events.single_write(
                                                    FireTorpedoEvent {
                                                        source: character_ent,
                                                        weapon_idx,
                                                        target: perform.target.select_uniform(progression.fire_count),
                                                        accuracy: fire_desc.accuracy,
                                                        effect: fire_desc.effect,
                                                        power: fire_desc.power,
                                                        element,
                                                    }
                                                );
                                            },
                                            FireType::Laser(element, time, should_multiply) => {
                                                let target_ent = perform.target.select_uniform(progression.fire_count);
                                                let dmg_mul: f32 = {
                                                    if should_multiply {
                                                        if let Some((slot_manager, _)) = get_root::<SlotManager, _, _>(&parents, &slot_managers, target_ent) {
                                                            if let Some((team, _)) = get_root::<Team, _, _>(&parents, &teams, target_ent) {
                                                                (slot_manager.for_team(*team).count() as f32)
                                                            } else {
                                                                1.0
                                                            }
                                                        } else {
                                                            1.0
                                                        }

                                                    } else {
                                                        1.0
                                                    }
                                                };
                                                fire_laser_events.single_write(
                                                    FireLaserEvent {
                                                        source: character_ent,
                                                        weapon_idx,
                                                        target: perform.target.select_uniform(progression.fire_count),
                                                        time,
                                                        accuracy: fire_desc.accuracy,
                                                        effect: fire_desc.effect,
                                                        power: fire_desc.power * dmg_mul,
                                                        element,
                                                    }
                                                );
                                            },
                                            _ => {
                                                unimplemented!();
                                            },
                                        }
                                        progression.fire_count += 1;
                                        progression.stage += 1;
                                    },
                                    AbilityActionDesc::Wave(weapon_idx, time, dmg) => {
                                        fire_wave_events.single_write(
                                            FireWaveEvent {
                                                source: character_ent,
                                                weapon_idx,
                                                targets: perform.target.to_vec(),
                                                time,
                                                dmg,
                                            }
                                        );
                                        progression.stage += 1;
                                    },
                                    AbilityActionDesc::InflictStatus(inflict) => {
                                        for target_ent in perform.target.to_vec() {
                                            Character::inflict_status_silent(&mut characters, target_ent, inflict);
                                        }
                                        progression.stage += 1;
                                    },
                                    AbilityActionDesc::InflictStatusFlexible(mut inflict, targeted) => {
                                        match perform.target.clone() {
                                            AbilityTarget::Single(target_ent) => {
                                                inflict.turns = targeted;
                                                Character::inflict_status_silent(&mut characters, target_ent, inflict);
                                            },
                                            AbilityTarget::Multi(targets) => {
                                                for target_ent in targets {
                                                    Character::inflict_status_silent(&mut characters, target_ent, inflict);
                                                }
                                            },
                                        };
                                        progression.stage += 1;
                                    }
                                    AbilityActionDesc::Heal(heal_value, target_ty) => {
                                        let targets = perform.target.to_vec();
                                        for target in targets.iter() {
                                            let mut invoke: bool = false;
                                            if let Some(ty) = target_ty {
                                                if let Some((team, _)) = Team::get_team(&parents, &teams, *target) {
                                                    if team.is_target_for(&ty) {
                                                        invoke = true;
                                                    }
                                                }
                                            } else {
                                                invoke = true;
                                            }
                                            if invoke {
                                                heal_events.single_write(
                                                    HealEvent {
                                                        owner: Some(entity),
                                                        source: Some(character_ent),
                                                        target: *target,
                                                        heal_value: heal_value / (targets.len() as f32),
                                                    }
                                                );
                                            }
                                        }
                                        progression.stage += 1;
                                    },
                                    AbilityActionDesc::Charge(charge_value, target_ty) => {
                                        let targets = perform.target.to_vec();
                                        for target in targets.iter() {
                                            let mut invoke: bool = false;
                                            if let Some(ty) = target_ty {
                                                if let Some((team, _)) = Team::get_team(&parents, &teams, *target) {
                                                    if team.is_target_for(&ty) {
                                                        invoke = true;
                                                    }
                                                }
                                            } else {
                                                invoke = true;
                                            }
                                            if invoke {
                                                charge_events.single_write(
                                                    ChargeEvent {
                                                        owner: Some(entity),
                                                        source: Some(character_ent),
                                                        target: *target,
                                                        charge_value: charge_value / (targets.len() as f32),
                                                    }
                                                );
                                            }
                                        }
                                        progression.stage += 1;
                                    },
                                    AbilityActionDesc::Wait(wait_time) => {
                                        if progression.stage_time >= wait_time {
                                            progression.stage_time = 0.0;
                                            progression.stage += 1;
                                        } else {
                                            progression.stage_time += time.delta_seconds();
                                            break;
                                        }
                                    },
                                }
                            } else {
                                to_remove.push(entity);
                                break;
                            }
                        }
                    }
                }

                for entity in to_remove {
                    Principal::try_root_disengage(&parents, &mut principals, entity, std::any::TypeId::of::<Self>());
                    performs.remove(entity);
                    progressions.remove(entity);
                }
            }
        }
    }
}

pub mod charge;
pub mod spawn;
pub mod hack;
pub mod solid_laser;
pub mod focused_charge;
pub mod shock;
pub mod nanobots;
pub mod scrambling_laser;
pub mod overclock;
pub mod twin_shot;
pub mod snipe;
pub mod barrage;
pub mod annihilate;
pub mod big_bullet;
pub mod jammer;
pub mod nuke;
pub mod overload;
pub mod corrupt;
pub mod sheild;
pub mod empower;
pub mod focus;
pub mod retribution;
pub mod self_destruct;
//pub mod upgrade;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Element {
    Kinetic,
    Plasma,
    Ion,
    Quantum,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DmgPackage {
    pub source: Option<Entity>,
    pub target: Entity,
    pub power: f32,
    pub element: Element,
    pub status: Option<StatusInflictDesc>,
}

#[derive(Debug, Copy, Clone, PartialEq, Component)]
pub struct DmgTimer {
    pub package: DmgPackage,
    /// Triggers the dmg when the timer reaches 0.
    pub timer: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Component)]
pub struct MissEvent {
    pub source: Option<Entity>,
    pub target: Entity,
}

#[derive(Debug, Copy, Clone, PartialEq, Component)]
pub struct HealEvent {
    /// The entity invoking this event.
    pub owner: Option<Entity>,
    pub source: Option<Entity>,
    pub target: Entity,

    pub heal_value: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Component)]
pub struct ChargeEvent {
    pub owner: Option<Entity>,
    pub source: Option<Entity>,
    pub target: Entity,

    pub charge_value: f32,
}


pub type AbilityId = std::any::TypeId;

/// Represents abilities associated with a drone.
/// Each ability is an entity, who's parent is the owner.
pub struct Abilities(Vec<Entity>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AbilityUsability<'a> {
    Common,
    Role(CharacterRole),
    Unique(&'a [CharacterId]),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AbilityCharge {
    Static(f32),
    Range(f32, f32),
}

impl AbilityCharge {
    pub fn rated_charge(&self) -> f32 {
        match self {
            AbilityCharge::Static(val) => *val,
            AbilityCharge::Range(lower, upper) => *lower,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct AbilityData {
    pub id: AbilityId,
    pub system: TypeId,
    pub cooldown: i32,
    pub charge: AbilityCharge,
    pub name: &'static str,
    pub desc: &'static str,
    /// This can be set to `None` for special abilities which do no take a target (e.g. spawn character).
    pub target_info: Option<AbilityTargetInfo>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum AbilityTargetType {
    Friendly,
    Enemy,
    All,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum AbilityTargetArea {
    Single,
    Flexible,
    All,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct AbilityTargetInfo {
    pub ty: AbilityTargetType,
    pub area: AbilityTargetArea,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AbilityTarget {
    Multi(Vec<Entity>),
    Single(Entity),
}

impl AbilityTarget {
    pub fn select(&self) -> Entity {
        match self {
            AbilityTarget::Multi(entities) => {
                debug_assert!(!entities.is_empty());
                if entities.len() == 1 {
                    entities[0]
                } else {
                    entities[rand::thread_rng().gen_range(0, entities.len())]
                }
            }
            AbilityTarget::Single(ent) => *ent,
        }
    }

    pub fn select_uniform(&self, counter: usize) -> Entity {
        match self {
            AbilityTarget::Multi(entities) => {
                debug_assert!(!entities.is_empty());
                let idx: usize = counter % entities.len();
                entities[idx]
            }
            AbilityTarget::Single(ent) => *ent,
        }
    }

    pub fn to_vec(&self) -> Vec<Entity> {
        match self {
            AbilityTarget::Multi(entities) => {
                entities.clone()
            }
            AbilityTarget::Single(ent) => vec![*ent],
        }
    }

    pub fn count(&self) -> usize {
        match self {
            AbilityTarget::Multi(entities) => {
                entities.len()
            }
            AbilityTarget::Single(ent) => 1,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Ability {
    pub data: AbilityData,
    pub locked: bool,
    pub current_cooldown: i32,
}

impl Component for Ability {
    type Storage = DenseVecStorage<Self>;
}

impl Ability {
    pub fn new(data: AbilityData) -> Self {
        Self {
            data,
            locked: false,
            current_cooldown: 0,
        }
    }
    pub fn apply_cooldown(&mut self) {
        self.current_cooldown = self.data.cooldown;
    }

    pub fn progress(&mut self, turns: i32) {
        self.current_cooldown -= turns;
        if self.current_cooldown > 0 {
            self.current_cooldown = 0;
        }
    }

    pub fn can_perform(&self) -> bool {
        self.current_cooldown == 0 && !self.locked
    }

    pub fn targets_for<'s>(
        entities: &Entities<'s>,
        parents: &impl GenericReadStorage<Component=Parent>,
        abilities: &impl GenericReadStorage<Component=Ability>,
        characters: &impl GenericReadStorage<Component=Character>,
        slot_managers: &impl GenericReadStorage<Component=SlotManager>,
        teams: &impl GenericReadStorage<Component=Team>,
        ability_ent: Entity,
    ) -> Vec<Entity> {
        let mut targets: Vec<Entity> = Vec::new();
        if let Some(ability) = abilities.get(ability_ent) {
            if let Some((ability_team, _)) = Team::get_team(parents, teams, ability_ent) {
                if let Some(target_info) = ability.data.target_info {
                    if target_info.ty == AbilityTargetType::Friendly || target_info.ty == AbilityTargetType::All {
                        if let Some((slot_manager, _)) = get_root::<SlotManager, _, _>(parents, slot_managers, ability_ent) {
                            for (i, character_ent) in slot_manager.for_team(ability_team).iter() {
                                targets.push(character_ent);
                            }
                        }
                    }
                    if target_info.ty == AbilityTargetType::Enemy || target_info.ty == AbilityTargetType::All {
                        if let Some((slot_manager, _)) = get_root::<SlotManager, _, _>(parents, slot_managers, ability_ent) {
                            for (i, character_ent) in slot_manager.for_team(ability_team.other()).iter() {
                                if i == 0 {
                                    if slot_manager.for_team(ability_team.other()).count() <= 1 {
                                        targets.push(character_ent);
                                    }
                                } else {
                                    targets.push(character_ent);
                                }
                            }
                        }
                    }
                }

            }
        }
        targets
    }
}


#[derive(Debug, Copy, Clone, Default, Component)]
pub struct Performing;

#[derive(Debug, Copy, Clone, Default, Component)]
pub struct AbilityInvoke;

pub struct AbilityPerform {
    pub target: AbilityTarget,
}

impl Component for AbilityPerform {
    type Storage = DenseVecStorage<Self>;
}

impl AbilityPerform {
    pub fn new(target: AbilityTarget) -> Self {
        Self {
            target,
        }
    }
}

#[derive(Debug, Copy, Clone, Error, PartialEq)]
pub enum AbilityPerformError {
    #[error(display = "ability is currently in cooldown")]
    InCooldown,
    #[error(display = "ability is locked")]
    Locked,
    #[error(display = "entity is invalid")]
    InvalidEntity,
}

pub struct AbilityAggregator<'a, 'b, 's> {
    pub abilities: &'a mut WriteStorage<'s, Ability>,
    pub ability_performs: &'b mut WriteStorage<'s, AbilityPerform>,
}

impl<'a, 'b, 's> AbilityAggregator<'a, 'b, 's> {
    pub fn new(abilities: &'a mut WriteStorage<'s, Ability>, ability_performs: &'b mut WriteStorage<'s, AbilityPerform>) -> Self {
        Self { abilities, ability_performs }
    }

    pub fn progress(&mut self, turns: i32) {
        for ability in self.abilities.join() {
            ability.progress(turns);
        }
    }

    /// Returns `None` if ability is invalid.
    pub fn can_perform(&self, ability_ent: Entity) -> Option<bool> {
        if let Some(ability) = self.abilities.get(ability_ent) {
            Some(ability.can_perform())
        } else {
            None
        }
    }

    pub fn get_ability(&self, ability_ent: Entity) -> Option<Ability> {
        if let Some(ability) = self.abilities.get(ability_ent) {
            Some(*ability)
        } else {
            None
        }
    }

    pub fn perform(&mut self, ability_ent: Entity, target: AbilityTarget, force_perform: bool) -> Result<(), AbilityPerformError> {
        if let Some(ability) = self.get_ability(ability_ent) {
            if ability.can_perform() || force_perform {
                self.ability_performs.insert(ability_ent, AbilityPerform::new(target));
            }

            Ok(())
        } else {
            Err(AbilityPerformError::InvalidEntity)
        }
    }
}

/// Contains a global list of abilities.
#[derive(Debug, Clone, Default)]
pub struct AbilityList {
    abilities: HashMap<AbilityId, AbilityData>,
    common: Vec<AbilityId>,
    role_specific: HashMap<CharacterRole, Vec<AbilityId>>,
    character_specific: HashMap<CharacterId, Vec<AbilityId>>,
}

impl AbilityList {
    /// Overwrites an existing ability with the same id.
    pub fn register(&mut self, ability: AbilityData, usability: AbilityUsability) {
        self.abilities.insert(ability.id, ability);
        match usability {
            AbilityUsability::Role(role) => {
                if let Some(abilities) = self.role_specific.get_mut(&role) {
                    abilities.push(ability.id);
                } else {
                    self.role_specific.insert(role, vec![ability.id]);
                }
            }
            AbilityUsability::Unique(character_ids) => {
                for character_id in character_ids {
                    if let Some(abilities) = self.character_specific.get_mut(character_id) {
                        abilities.push(ability.id);
                    } else {
                        self.character_specific.insert(*character_id, vec![ability.id]);
                    }
                }
            }
            AbilityUsability::Common => {
                self.common.push(ability.id);
            }
        }
    }

    #[inline]
    pub fn ability(&self, id: &AbilityId) -> Option<AbilityData> {
        self.abilities.get(id).copied()
    }

    pub fn abilities_for(&self, character: &Character) -> HashMap<AbilityId, AbilityData> {
        let mut res: HashMap<AbilityId, AbilityData> = HashMap::new();

        for id in self.common.iter() {
            if let Some(ability_data) = self.ability(id) {
                res.insert(*id, ability_data);
            }
        }
        if let Some(role_specific) = self.role_specific.get(&character.role()) {
            for id in role_specific {
                if let Some(ability_data) = self.ability(&id) {
                    res.insert(*id, ability_data);
                }
            }
        }
        if let Some(character_specific) = self.character_specific.get(&character.id()) {
            for id in character_specific {
                if let Some(ability_data) = self.ability(&id) {
                    res.insert(*id, ability_data);
                }
            }
        }
        res
    }
}

#[derive(Debug, Copy, Clone)]
pub struct UnassignedAbility(pub AbilityId);

impl Component for UnassignedAbility {
    type Storage = DenseVecStorage<Self>;
}

impl UnassignedAbility {
    #[inline]
    pub fn id(&self) -> AbilityId {
        self.0
    }
}

/// The 'base' function to invoke an ability.
pub fn perform_ability<'s>(
    parents: &ReadStorage<'s, Parent>, principals: &mut WriteStorage<'s, Principal>, abilities: &ReadStorage<'s, Ability>, ability_performs: &mut WriteStorage<'s, AbilityPerform>,
    ability_ent: Entity, target: AbilityTarget,
) -> Option<bool> {
    if !ability_performs.contains(ability_ent) {
        if let Some(ability) = abilities.get(ability_ent) {
            let engage_result = Principal::try_root_engage(
                parents, principals,
                ability_ent, ability.data.system,
            );
            if engage_result == Some(true) {
                ability_performs.insert(ability_ent, AbilityPerform::new(target));
                Some(true)
            } else {
                Some(false)
            }
        } else {
            Some(false)
        }
    } else {
        None
    }
}

#[derive(Debug, Copy, Clone, Default, Component)]
pub struct AbilityProgression {
    pub time: f32,
    pub stage_time: f32,
    pub stage: usize,
    pub fire_count: usize,
}

impl AbilityProgression {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FireDesc {
    pub ty: FireType,
    pub power: f32,
    pub accuracy: f32,
    pub effect: Option<StatusInflictDesc>,
}

impl FireDesc {
    pub const fn bullet(power: f32, accuracy: f32) -> Self {
        Self {
            ty: FireType::Bullet,
            power,
            accuracy,
            effect: None,
        }
    }
    pub const fn missile(power: f32, accuracy: f32) -> Self {
        Self {
            ty: FireType::Missile,
            power,
            accuracy,
            effect: None,
        }
    }
    pub const fn torpedo(power: f32, element: Element, accuracy: f32) -> Self {
        Self {
            ty: FireType::Torpedo(element),
            power,
            accuracy,
            effect: None,
        }
    }
    pub const fn laser(power: f32, element: Element, accuracy: f32, time: f32) -> Self {
        Self {
            ty: FireType::Laser(element, time, false),
            power,
            accuracy,
            effect: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FireType {
    Bullet,
    Missile,
    Torpedo(Element),
    Laser(Element, f32, bool),
    Wave(Option<WaveDmg>, f32),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AbilityActionDesc {
    Fire(FireDesc, usize),
    Wait(f32),
    Heal(f32, Option<AbilityTargetType>),
    Charge(f32, Option<AbilityTargetType>),
    Wave(usize, f32, Option<WaveDmg>),
    InflictStatus(StatusInflictDesc),
    InflictStatusFlexible(StatusInflictDesc, usize),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct StatusInflictDesc {
    pub ty: StatusType,
    pub turns: usize,
    pub chance: f32,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct FireBulletEvent {
    pub source: Entity,
    pub weapon_idx: usize,
    pub target: Entity,
    pub power: f32,
    pub accuracy: f32,
    pub effect: Option<StatusInflictDesc>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct FireTorpedoEvent {
    pub source: Entity,
    pub weapon_idx: usize,
    pub target: Entity,
    pub power: f32,
    pub accuracy: f32,
    pub element: Element,
    pub effect: Option<StatusInflictDesc>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct FireLaserEvent {
    pub source: Entity,
    pub weapon_idx: usize,
    pub target: Entity,
    pub time: f32,
    pub power: f32,
    pub accuracy: f32,
    pub element: Element,
    pub effect: Option<StatusInflictDesc>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct WaveDmg {
    pub accuracy: f32,
    pub power: f32,
    pub element: Element,
    pub effect: Option<StatusInflictDesc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FireWaveEvent {
    pub source: Entity,
    pub weapon_idx: usize,
    pub targets: Vec<Entity>,
    pub time: f32,
    pub dmg: Option<WaveDmg>,
}

#[derive(Debug, new, SystemDesc)]
#[system_desc(name(PerformingAbilitySystemDesc))]
pub struct PerformingAbilitySystem;

impl<'s> System<'s> for PerformingAbilitySystem {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, Parent>,
        ReadStorage<'s, Ability>,
        ReadStorage<'s, AbilityPerform>,
        ReadStorage<'s, Character>,
        WriteStorage<'s, Performing>,
    );

    fn run(&mut self, (entities, parents, abilities, ability_performs, characters, mut performings): Self::SystemData) {
        let mut existing: Vec<Entity> = Vec::new();
        for (ability_ent, _, _) in (&entities, abilities.mask(), ability_performs.mask()).join() {
            if let Some((_, character_ent)) = get_root::<Character, _, _>(&parents, &characters, ability_ent) {
                existing.push(character_ent);
                performings.insert(character_ent, Performing);
            }
        }

        for (ent, _) in (&entities, performings.mask().clone()).join() {
            if !existing.contains(&ent) {
                performings.remove(ent);
            }
        }
    }
}