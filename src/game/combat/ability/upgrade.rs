use amethyst::{
    assets::{
        Handle,
        Prefab,
    },
    core::{
        Parent,
        math::Vector2,
    },
    ecs::prelude::*,
    prelude::SystemDesc,
    shrev::EventChannel,
    ui::UiTransform,
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
use crate::game::character::{CharacterPrefabData, CharacterStore, UnassignedCharacter, CharacterRole, MasterDrone};
use crate::game::combat::{Team};
use crate::game::combat::ability::{AbilityData, AbilityPerform, AbilityTargetArea, AbilityTargetInfo, AbilityTargetType, UnassignedAbility, AbilityTarget, AbilityList, AbilityUsability, AbilityCharge};
use crate::game::combat::spawn::SpawnAction;
use crate::game::ui::marker::{MarkerUiCompletedEvent, ShowUiMarkerEvent};
use crate::game::ui::select_character::{CharacterSelectedEvent, SelectCharacterEvent};
use crate::game::combat::tactical::{AiAbilitySelection, AiAbilitySelectionQuery};
use crate::game::combat::status::StatusType;
use crate::game::ui::hud::UiBase;
use crate::game::ui::status::{UiStatus};

#[derive(Debug, Copy, Clone, Default, Component)]
pub struct UpgradeAbility;

#[derive(Debug, Copy, Clone, Default, Component)]
pub struct UpgradePerform;

impl UpgradeAbility {
    pub fn data() -> AbilityData {
        AbilityData {
            name: "Upgrade",
            desc: "Upgrades the drone.",
            id: TypeId::of::<Self>(),
            system: TypeId::of::<UpgradeAbilitySystem>(),
            charge: AbilityCharge::Static(100.0),
            target_info: Some(AbilityTargetInfo {
                ty: AbilityTargetType::Friendly,
                area: AbilityTargetArea::Single,
            }),
            cooldown: 4,
        }
    }
}

pub const UPGRADE_ANIM_TIME: f32 = 1.0;

#[derive(Debug, new, SystemDesc)]
#[system_desc(name(UpgradeAbilitySystemDesc))]
pub struct UpgradeAbilitySystem {
    #[system_desc(event_channel_reader)]
    completed_event_reader: ReaderId<MarkerUiCompletedEvent>,
}

impl<'s> System<'s> for UpgradeAbilitySystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Principal>,
        WriteStorage<'s, Character>,
        WriteStorage<'s, Ability>,
        ReadStorage<'s, UiBase>,
        WriteStorage<'s, UiStatus>,
        ReadStorage<'s, UiTransform>,
        WriteStorage<'s, AbilityPerform>,
        WriteStorage<'s, UnassignedAbility>,
        WriteStorage<'s, AiAbilitySelectionQuery>,
        WriteStorage<'s, AbilityInvoke>,
        WriteStorage<'s, UpgradeAbility>,
        WriteStorage<'s, UpgradePerform>,
        WriteStorage<'s, Parent>,
        ReadStorage<'s, Team>,
        Write<'s, EventChannel<ShowUiMarkerEvent>>,
        Read<'s, EventChannel<MarkerUiCompletedEvent>>,
    );

    fn setup(&mut self, world: &mut World) {
        world.fetch_mut::<AbilityList>().register(UpgradeAbility::data(), AbilityUsability::Unique(&[MasterDrone::character_id()]));
    }

    fn run(&mut self, (entities, mut principals, mut characters, mut abilities, ui_bases, mut ui_statuses, ui_transforms, mut performs, mut unassigned_abilities, mut ability_selections, mut ability_invokes, mut upgrade_abilities, mut upgrade_performs, mut parents, teams, mut marker_events, completed_events): Self::SystemData) {
        for (entity, ability, _, mut ability_selection) in (&entities, &abilities, upgrade_abilities.mask(), &mut ability_selections).join() {
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
            if ability.data.id == TypeId::of::<UpgradeAbility>() {
                upgrade_abilities.insert(entity, UpgradeAbility::default());
                unassigned_abilities.remove(entity);
            }
        }

        for (entity, mut ability, _) in (&entities, &mut abilities, upgrade_abilities.mask()).join() {
            if let Some((character, _)) = get_root::<Character, _, _>(&parents, &characters, entity) {
                let charge: f32 = {
//                    if character.relative_health() <= 0.5 {
//                        700.0
//                    } else if character.relative_health() <= 0.3 {
//                        500.0
//                    } else if character.relative_health() <= 0.2 {
//                        300.0
//                    } else if character.relative_health() <= 0.1 {
//                        100.0
//                    } else {
//                        1000.0
//                    }
                    100.0
                };
                ability.data.charge = AbilityCharge::Static(charge);
            }
        }

        // Check if the ability has been triggered.
        for (ent, ability, perform, mut charge_ability, _) in (&entities, &abilities, &performs, &mut upgrade_abilities, !upgrade_performs.mask().clone()).join() {
            if let Some((_, character_ent)) = get_root::<Character, _, _>(&parents, &characters, ent) {
                if let AbilityTarget::Single(target_ent) = perform.target {
                    if Character::try_take_turn(&mut characters, character_ent, 0.0) {
                        if let Some(target) = characters.get_mut(target_ent) {
                            if target.try_upgrade() {
                                Principal::try_root_engage(&parents, &mut principals, ent, std::any::TypeId::of::<Self>());
                                if let Some(ui_base) = ui_bases.get(target_ent) {
                                    if let Some(ui_transform) = ui_transforms.get(ui_base.entity()) {

                                        // Update status bar.
                                        let mut to_remove: Vec<(Entity, Option<Parent>, UiStatus)> = Vec::new();
                                        for (entity, status) in (&entities, &ui_statuses).join() {
                                            if status.character_ent == target_ent {
                                                to_remove.push((entity, parents.get(entity).cloned(), status.clone()));
                                            }
                                        }

                                        for (entity, parent, status) in to_remove {
                                            entities.delete(entity);
                                            let new_status_ent = entities.create();
                                            ui_statuses.insert(new_status_ent, status);
                                            if let Some(parent) = parent {
                                                parents.insert(new_status_ent, parent);
                                            }
                                        }

                                        marker_events.single_write(
                                            ShowUiMarkerEvent {
                                                owner: Some(ent),
                                                text: String::from("Upgrade"),
                                                text_color: [0.5, 0.0, 0.5, 1.0],
                                                character: Some(target_ent),
                                                position: Vector2::new(ui_transform.local_x, ui_transform.local_y),
                                                fade: true,
                                                anim_time: UPGRADE_ANIM_TIME,
                                                anim_vel: Some(Vector2::new(0.0, 100.0)),
                                            }
                                        );
                                    }
                                }
                                upgrade_performs.insert(ent, UpgradePerform);
                            }
                            continue;
                        }
                    } else {
                        panic!("[UpgradeAbilitySystem] Unexpected failure to take turn.");
                    }
                }

            }
            Principal::try_root_disengage(&parents, &mut principals, ent, std::any::TypeId::of::<Self>());
        }

        for event in completed_events.read(&mut self.completed_event_reader) {
            if let Some(owner) = event.owner {
                if upgrade_abilities.contains(owner) {
                    performs.remove(owner);
                    ability_invokes.remove(owner);
                    upgrade_performs.remove(owner);
                    Principal::try_root_disengage(&parents, &mut principals, owner, std::any::TypeId::of::<Self>());
                }
            }
        }
    }
}