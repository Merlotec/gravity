
use amethyst::{
    core::{
        Parent,
        ParentHierarchy,
        Transform,
        math::Vector3,
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
            CombatRoot,
            CombatState,
            spawn::{SlotManager, Slots},
            TickTurn,
        },
    },
};
use crate::game::combat::{Team};
use crate::game::combat::process::Principal;
use crate::game::combat::systems::delay::Delay;
use crate::game::combat::systems::enemy_wave::SpawnWaveEvent;
use crate::game::combat::ability::annihilate::AnnihilatePlusAbility;
use crate::game::character::{CharacterDefeatedEvent, MasterDrone};
use crate::game::ui::banner::ShowUiBannerDisplayEvent;
use crate::game::ui::crosshair::UiCrosshair;
use crate::game::combat::ability::hack::HackPerformedEvent;
use crate::core::get_root;
use crate::game::combat::ability::{DmgPackage, Element, Ability, UnassignedAbility};
use crate::game::combat::spawn::{CharacterSpawnedEvent, SpawnProcess};

#[derive(Debug, new, SystemDesc)]
#[system_desc(name(EarthCombatSystemDesc))]
pub struct EarthCombatSystem {
    #[system_desc(event_channel_reader)]
    pub character_spawned_event_reader: ReaderId<CharacterSpawnedEvent>,
}

impl<'s> System<'s> for EarthCombatSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Character>,
        WriteStorage<'s, SlotManager>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, CombatRoot>,
        WriteStorage<'s, Principal>,
        WriteStorage<'s, Ability>,
        WriteStorage<'s, UnassignedAbility>,
        WriteStorage<'s, SpawnProcess>,
        WriteStorage<'s, Parent>,
        ReadExpect<'s, ParentHierarchy>,
        Read<'s, EventChannel<CharacterSpawnedEvent>>,
    );

    fn run(&mut self, (entities, mut characters, mut slot_managers, mut transforms, mut combat_roots, mut principals, mut abilities, mut unassigned_abilities, mut spawn_processes, mut parents, hierarchy, character_spawned_events): Self::SystemData) {
        for event in character_spawned_events.read(&mut self.character_spawned_event_reader) {
            for (entity, mut root, slot_manager) in (&entities, &mut combat_roots, &slot_managers).join() {
                if root.data.name == "earth" {
                    if slot_manager.friendly.master() == Some(event.character_ent) {
                        if event.action.character_id == MasterDrone::character_id() {
                            if let Some(character) = characters.get_mut(event.character_ent) {
                                character.set_max_charge(1000.0);
                                character.set_charge(character.max_charge());
                            }
                            for (ent, _, parent) in (&entities, abilities.mask(), &parents).join() {
                                if parent.entity == event.character_ent {
                                    entities.delete(ent);
                                }
                            }
                            Character::insert_ability(
                                &entities,
                                &mut parents,
                                &mut abilities,
                                &mut unassigned_abilities,
                                event.character_ent,
                                AnnihilatePlusAbility::data(),
                            )
                        }
                    }
                    if slot_manager.enemy.master() == Some(event.character_ent) {
                        spawn_processes.remove(event.character_ent);
                    }
                }
            }
        }
    }
}