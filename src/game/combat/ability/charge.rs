use amethyst::{
    assets::{
        Handle,
        Prefab,
        AssetStorage,
        Loader,
    },
    audio::{
        Mp3Format,
        output::Output,
        Source,
        SourceHandle,
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
use crate::core::{activity::ActivityState, get_root};
use crate::game::character::{CharacterPrefabData, CharacterStore, UnassignedCharacter, CharacterRole};
use crate::game::combat::{Team};
use crate::game::combat::ability::{AbilityData, AbilityPerform, AbilityTargetArea, AbilityTargetInfo, AbilityTargetType, ChargeEvent, UnassignedAbility, AbilityCharge, AbilityTarget, AbilityList, AbilityUsability};
use crate::game::combat::spawn::SpawnAction;
use crate::game::ui::marker::MarkerUiCompletedEvent;
use crate::game::ui::select_character::{CharacterSelectedEvent, SelectCharacterEvent};
use crate::game::combat::tactical::{AiAbilitySelection, AiAbilitySelectionQuery};
use crate::game::combat::status::StatusType;

#[derive(Debug, Copy, Clone, Default, Component)]
pub struct ChargeAbility;

#[derive(Debug, Copy, Clone, Default, Component)]
pub struct ChargePerform;

impl ChargeAbility {
    pub fn data() -> AbilityData {
        AbilityData {
            name: "Charge",
            desc: "Charges the drone.",
            id: TypeId::of::<Self>(),
            system: TypeId::of::<ChargeAbilitySystem>(),
            charge: AbilityCharge::Static(-40.0),
            target_info: None,
            cooldown: 4,
        }
    }
}

#[derive(Debug, new, SystemDesc)]
#[system_desc(name(ChargeAbilitySystemDesc))]
pub struct ChargeAbilitySystem {
    sound_handle: Option<SourceHandle>,
    #[system_desc(event_channel_reader)]
    completed_event_reader: ReaderId<MarkerUiCompletedEvent>,
}

impl<'s> System<'s> for ChargeAbilitySystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Principal>,
        WriteStorage<'s, Character>,
        WriteStorage<'s, Ability>,
        WriteStorage<'s, AbilityPerform>,
        WriteStorage<'s, UnassignedAbility>,
        WriteStorage<'s, AiAbilitySelectionQuery>,
        WriteStorage<'s, AbilityInvoke>,
        WriteStorage<'s, ChargeAbility>,
        WriteStorage<'s, ChargePerform>,
        WriteStorage<'s, Parent>,
        ReadStorage<'s, Team>,
        Write<'s, EventChannel<ChargeEvent>>,
        Read<'s, EventChannel<MarkerUiCompletedEvent>>,
        Read<'s, AssetStorage<Source>>,
        ReadExpect<'s, Loader>,
        Option<Read<'s, Output>>,
    );

    fn setup(&mut self, world: &mut World) {
        self.sound_handle = Some(world.read_resource::<Loader>().load("music/charge.mp3", Mp3Format, (), &world.read_resource()));
        world.fetch_mut::<AbilityList>().register(ChargeAbility::data(), AbilityUsability::Common);
    }

    fn run(&mut self, (entities, mut principals, mut characters, mut abilities, mut performs, mut unassigned_abilities, mut ability_selections, mut ability_invokes, mut charge_abilities, mut charge_performs, mut parents, teams, mut charge_events, completed_events, audio_assets, loader, output): Self::SystemData) {
        for (entity, ability, _, mut ability_selection) in (&entities, &abilities, charge_abilities.mask(), &mut ability_selections).join() {
            if let Some((character, character_ent)) = get_root::<Character, _, _>(&parents, &characters,  entity) {
                // The higher the charge, the greater the likelihood.
                let mut score: f32 = 2.2 + 0.3 * (1.0 - character.relative_charge());
                if character.relative_charge() >= 1.0 || character.has_status(StatusType::Unstable){
                    score = 0.0;
                }
                ability_selection.result = Some(
                    AiAbilitySelection {
                        score,
                        target: AbilityTarget::Single(character_ent),
                    }
                );
            }
        }

        for (entity, mut ability, _) in (&entities, &mut abilities, unassigned_abilities.mask().clone()).join() {
            if ability.data.id == TypeId::of::<ChargeAbility>() {
                if let Some((character, _)) = get_root::<Character, _, _>(&parents, &characters, entity) {
                    ability.data.charge = AbilityCharge::Static(-character.artificial_charge());
                }
                charge_abilities.insert(entity, ChargeAbility::default());
                unassigned_abilities.remove(entity);
            }
        }

        for (entity, mut ability, _) in (&entities, &mut abilities, charge_abilities.mask()).join() {
            if let Some((character, _)) = get_root::<Character, _, _>(&parents, &characters, entity) {
                if character.has_status(StatusType::Unstable) {
                    ability.data.charge = AbilityCharge::Static(0.0);
                } else {
                    ability.data.charge = AbilityCharge::Static(-character.artificial_charge());
                }
            }
        }

        // Check if the ability has been triggered.
        for (ent, ability, _, mut charge_ability, _) in (&entities, &abilities, performs.mask().clone() | ability_invokes.mask().clone(), &mut charge_abilities, !charge_performs.mask().clone()).join() {
            if let Some((_, character_ent)) = get_root::<Character, _, _>(&parents, &characters, ent) {
                if Character::try_take_turn(&mut characters, character_ent, 0.0) {
                    Principal::try_root_engage(&parents, &mut principals, ent, std::any::TypeId::of::<Self>());

                    // Play sound
                    if let Some(ref handle) = self.sound_handle {
                        if let Some(ref output) = output {
                            if let Some(sound) = audio_assets.get(&handle) {
                                output.play_once(sound, 0.7);
                            }
                        }
                    }

                    charge_events.single_write(
                        ChargeEvent {
                            owner: Some(ent),
                            charge_value: ability.data.charge.rated_charge().abs(),
                            target: character_ent,
                            source: Some(character_ent),
                        }
                    );
                    charge_performs.insert(ent, ChargePerform);
                    performs.insert(ent, AbilityPerform{
                        target: AbilityTarget::Multi(Vec::new()),
                    });
                    continue;
                } else {
                    panic!("[ChargeAbilitySystem] Unexpected failure to take turn.");
                }
            }
            Principal::try_root_disengage(&parents, &mut principals, ent, std::any::TypeId::of::<Self>());
        }

        for event in completed_events.read(&mut self.completed_event_reader) {
            if let Some(owner) = event.owner {
                if charge_abilities.contains(owner) {
                    performs.remove(owner);
                    ability_invokes.remove(owner);
                    charge_performs.remove(owner);
                    Principal::try_root_disengage(&parents, &mut principals, owner, std::any::TypeId::of::<Self>());
                }
            }
        }
    }
}