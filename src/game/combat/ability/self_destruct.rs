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
use std::any::TypeId;

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
use crate::core::{activity::ActivityState, get_root, get_root_mut};
use crate::game::character::{CharacterPrefabData, CharacterStore, UnassignedCharacter, CharacterRole};
use crate::game::combat::{Team};
use crate::game::combat::ability::{AbilityData, AbilityPerform, AbilityTargetArea, AbilityTargetInfo, AbilityTargetType, UnassignedAbility, AbilityTarget, AbilityList, AbilityUsability, AbilityCharge};
use crate::game::combat::spawn::SpawnAction;
use crate::game::ui::marker::{MarkerUiCompletedEvent, ShowUiMarkerEvent};
use crate::game::ui::select_character::{CharacterSelectedEvent, SelectCharacterEvent};
use crate::game::combat::tactical::{AiAbilitySelection, AiAbilitySelectionQuery};
use crate::game::combat::status::StatusType;

#[derive(Debug, Copy, Clone, Default, Component)]
pub struct SelfDestructAbility;

#[derive(Debug, Copy, Clone, Default, Component)]
pub struct SelfDestructPerform;

impl SelfDestructAbility {
    pub fn data() -> AbilityData {
        AbilityData {
            name: "Self Destruct",
            desc: "Self destructs the drone.",
            id: TypeId::of::<Self>(),
            system: TypeId::of::<SelfDestructAbilitySystem>(),
            charge: AbilityCharge::Static(0.0),
            target_info: None,
            cooldown: 4,
        }
    }
}

#[derive(Debug, new, SystemDesc)]
#[system_desc(name(SelfDestructAbilitySystemDesc))]
pub struct SelfDestructAbilitySystem {
    #[system_desc(event_channel_reader)]
    completed_event_reader: ReaderId<MarkerUiCompletedEvent>,
}

impl<'s> System<'s> for SelfDestructAbilitySystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Principal>,
        WriteStorage<'s, Character>,
        WriteStorage<'s, Ability>,
        WriteStorage<'s, AbilityPerform>,
        WriteStorage<'s, UnassignedAbility>,
        WriteStorage<'s, AiAbilitySelectionQuery>,
        WriteStorage<'s, AbilityInvoke>,
        WriteStorage<'s, SelfDestructAbility>,
        WriteStorage<'s, Parent>,
        ReadStorage<'s, Team>,
        Write<'s, EventChannel<ShowUiMarkerEvent>>,
        Read<'s, EventChannel<MarkerUiCompletedEvent>>,
    );

    fn setup(&mut self, world: &mut World) {
        world.fetch_mut::<AbilityList>().register(SelfDestructAbility::data(), AbilityUsability::Role(CharacterRole::Slave));
    }

    fn run(&mut self, (entities, mut principals, mut characters, mut abilities, mut performs, mut unassigned_abilities, mut ability_selections, mut ability_invokes, mut self_destruct_abilities, mut parents, teams, mut marker_events, completed_events): Self::SystemData) {
        for (entity, ability, _, mut ability_selection) in (&entities, &abilities, self_destruct_abilities.mask(), &mut ability_selections).join() {
            if let Some((_, character_ent)) = get_root::<Character, _, _>(&parents, &characters, entity) {
                ability_selection.result = Some(
                    AiAbilitySelection {
                        score: 0.0,
                        target: AbilityTarget::Single(character_ent),
                    }
                );
            }
        }

        for (entity, mut ability, _) in (&entities, &mut abilities, unassigned_abilities.mask().clone()).join() {
            if ability.data.id == TypeId::of::<SelfDestructAbility>() {
                self_destruct_abilities.insert(entity, SelfDestructAbility::default());
                unassigned_abilities.remove(entity);
            }
        }

        // Check if the ability has been triggered.
        for (ent, ability, _, _) in (&entities, &abilities, performs.mask().clone() | ability_invokes.mask().clone(), self_destruct_abilities.mask()).join() {
            if let Some((_, character_ent)) = get_root::<Character, _, _>(&parents, &mut characters, ent) {
                if Character::try_take_turn(&mut characters, character_ent, 0.0) {
                    if let Some(character) = characters.get_mut(character_ent) {
                        character.set_health(0.0);
                        performs.remove(ent);
                        ability_invokes.remove(ent);
                        ability_invokes.remove(ent);
                    }
                } else {
                    panic!("[SelfDestructAbilitySystem] Unexpected failure to take turn.");
                }
            }
            Principal::try_root_disengage(&parents, &mut principals, ent, std::any::TypeId::of::<Self>());
        }
    }
}