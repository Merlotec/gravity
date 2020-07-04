use amethyst::{
    core::{
        Parent,
        ParentHierarchy,
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
use crate::game::character::CharacterDefeatedEvent;
use crate::game::ui::banner::ShowUiBannerDisplayEvent;
use crate::game::ui::crosshair::UiCrosshair;
use crate::game::combat::ability::hack::HackPerformedEvent;
use crate::core::get_root;
use crate::game::combat::ability::{DmgPackage, Element};

pub const EXIT_TIMER: f32 = 3.0;

#[derive(Debug, new, SystemDesc)]
#[system_desc(name(StandardCombatSystemDesc))]
pub struct StandardCombatSystem {
    #[system_desc(event_channel_reader)]
    pub defeated_event_reader: ReaderId<CharacterDefeatedEvent>,

    #[system_desc(event_channel_reader)]
    pub hack_event_reader: ReaderId<HackPerformedEvent>,
}

#[derive(Debug, Copy, Clone, Default, Component)]
pub struct HasInitTag;

pub struct ExitCombat {
    pub timer: f32,
    pub(crate) winner: Option<Team>,
}

impl<'s> System<'s> for StandardCombatSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Character>,
        WriteStorage<'s, SlotManager>,
        WriteStorage<'s, CombatRoot>,
        WriteStorage<'s, HasInitTag>,
        WriteStorage<'s, Principal>,
        ReadStorage<'s, Parent>,
        ReadStorage<'s, Team>,
        ReadStorage<'s, UiCrosshair>,
        WriteStorage<'s, Delay>,
        ReadExpect<'s, ParentHierarchy>,
        Write<'s, Option<ExitCombat>>,
        Write<'s, EventChannel<TickTurn>>,
        Read<'s, EventChannel<CharacterDefeatedEvent>>,
        Write<'s, EventChannel<SpawnWaveEvent>>,
        Write<'s, EventChannel<ShowUiBannerDisplayEvent>>,
        Write<'s, EventChannel<DmgPackage>>,
        Read<'s, EventChannel<HackPerformedEvent>>,
    );

    fn run(&mut self, (entities, mut characters, mut slot_managers, mut combat_roots, mut has_init_tags, mut principals, parents, teams, crosshairs, mut delays, hierarchy, mut exit, mut tick_evt, defeated_events, mut spawn_wave_events, mut show_banners, mut dmg_events, hacked_events): Self::SystemData) {
        for (entity, mut root, slot_manager) in (&entities, &mut combat_roots, &slot_managers).join() {
            // Only execute in no principals are running.
            if Principal::is_root_engaged(
                &parents,
                &principals,
                entity,
            ) != Some(true) {
                // Perform state changes.
                match root.current_state {
                    CombatState::Init => {
                        if has_init_tags.contains(entity) {
                            root.current_state = CombatState::InTurn(Team::Friendly);
                        } else {
                            has_init_tags.insert(entity, HasInitTag::default());
                        }
                    }
                    CombatState::InTurn(team) => {
                        let mut active: bool = false;
                        // Check if all turns have been used.
                        for (i, entity_opt) in slot_manager.for_team(team).occupied().iter().enumerate() {
                            if let Some(character_ent) = *entity_opt {
                                if let Some(character) = characters.get(character_ent) {
                                    if character.has_turn() {
                                        active = true;
                                    }
                                }
                            }
                        }
                        if !active {
                            root.current_state = CombatState::DoneTurn(team);
                        }
                    }
                    CombatState::DoneTurn(team) => {
                        let new_team: Team = team.other();
                        // Tick
                        tick_evt.single_write(TickTurn {
                            count: root.turn_count,
                            next_team: new_team,
                        });

                        // Start new turn.
                        for (i, entity_opt) in slot_manager.for_team(new_team).occupied().iter().enumerate() {
                            if let Some(character_ent) = *entity_opt {
                                if let Some(character) = characters.get_mut(character_ent) {
                                    character.restore();
                                }
                            }
                        }

                        // Perform any tasks in between turns.
                        root.current_state = CombatState::InTurn(new_team);
                    }
                    _ => {}
                }
            }
            // Spawn in next wave.
            let mut test_state: bool = false;
            for event in defeated_events.read(&mut self.defeated_event_reader) {
                if let Some((team, team_ent)) = Team::get_team(&parents, &teams, event.character_ent) {
                    if let Some((slot_manager, _)) = get_root::<SlotManager, _, _>(&parents, &slot_managers, team_ent) {
                        if let Some(master) = slot_manager.for_team(team).master() {
                            let power: f32 = event.splash_dmg;

                            dmg_events.single_write(
                                DmgPackage {
                                    target: master,
                                    source: None,
                                    power,
                                    element: Element::Kinetic,
                                    status: None,
                                }
                            )
                        }
                    }
                }
                test_state = true;
            }
            for event in hacked_events.read(&mut self.hack_event_reader) {
                test_state = true;
            }
            if test_state {
                if slot_manager.enemy.is_empty() {
                    let idx: usize = root.current_wave + 1;
                    if let Some(wave) = root.data.waves.get(idx) {
                        let mut team_ent: Option<Entity> = None;
                        for (ent, team, _) in (&entities, &teams, hierarchy.all_children(entity)).join() {
                            if *team == Team::Enemy {
                                team_ent = Some(ent);
                                break;
                            }
                        }
                        if let Some(team_ent) = team_ent {
                            spawn_wave_events.single_write(
                                SpawnWaveEvent {
                                    wave: wave.clone(),
                                    idx,
                                    team_ent,
                                }
                            );
                        }
                        root.current_wave = idx;
                    } else {
                        show_banners.single_write(
                            ShowUiBannerDisplayEvent {
                                text: "Victory".to_string(),
                                color: [0.0, 1.0, 0.0, 1.0],
                                owner: entity,
                            }
                        );
                        root.current_state = CombatState::Victory(Team::Friendly);
                        *exit = Some(ExitCombat {
                            timer: EXIT_TIMER,
                            winner: Some(Team::Friendly),
                        });
                        for (crosshair_ent, _) in (&entities, crosshairs.mask()).join() {
                            entities.delete(crosshair_ent);
                        }
                    }
                }
                if slot_manager.friendly.master().is_none() {
                    show_banners.single_write(
                        ShowUiBannerDisplayEvent {
                            text: "Defeated".to_string(),
                            color: [1.0, 0.0, 0.0, 1.0],
                            owner: entity,
                        }
                    );
                    root.current_state = CombatState::Victory(Team::Enemy);
                    *exit = Some(ExitCombat {
                        timer: EXIT_TIMER,
                        winner: Some(Team::Enemy),
                    });
                    for (crosshair_ent, _) in (&entities, crosshairs.mask()).join() {
                        entities.delete(crosshair_ent);
                    }
                }
            }
        }
    }
}