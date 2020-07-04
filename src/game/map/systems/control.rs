use amethyst::{
    core::{
        SystemDesc,
        Transform,
        math::Vector3,
        Time,
        Parent,
    },
    ecs::prelude::*,
    input::{
        InputEvent,
        StringBindings,
    },
    shrev::{
        EventChannel,
        ReaderId,
    },
    winit::{
        VirtualKeyCode,
        MouseButton,
    },
};
use crate::game::character::Character;
use crate::game::map::{CurrentState, MapPawn, MapRoot, MapPoint, MapStage, DialogueStore, CombatStore, EngageCombat, save_current};
use failure::_core::mem::take;
use crate::core::{get_root, get_root_mut};
use crate::game::ui::dialogue::{DialogueCompletedEvent, ShowDialogueDisplayEvent};
use crate::game::combat::process::Principal;
use crate::state::map_state::MapState;
use crate::game::map::systems::movement::MoveTarget;


#[derive(Debug, new, SystemDesc)]
#[system_desc(name(MapControlSystemDesc))]
pub struct MapControlSystem {
    #[system_desc(event_channel_reader)]
    input_event_reader: ReaderId<InputEvent<StringBindings>>,

    #[system_desc(event_channel_reader)]
    dialogue_completed_event_reader: ReaderId<DialogueCompletedEvent>,
}

impl<'s> System<'s> for MapControlSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Parent>,
        ReadStorage<'s, Principal>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, MapRoot>,
        WriteStorage<'s, MapPawn>,
        ReadStorage<'s, MapPoint>,
        ReadStorage<'s, MoveTarget>,
        Write<'s, CurrentState>,
        Read<'s, DialogueStore>,
        Read<'s, CombatStore>,
        Write<'s, Option<EngageCombat>>,
        Read<'s, EventChannel<InputEvent<StringBindings>>>,
        Read<'s, EventChannel<DialogueCompletedEvent>>,
        Write<'s, EventChannel<ShowDialogueDisplayEvent>>,
    );

    fn run(&mut self, (entities, parents, principals, mut transforms, mut map_roots, mut map_pawns, points, move_targets, mut current_state, dialogue_store, combat_store, mut engage_combat, input_events, dialogue_completed_events, mut show_dialogue_events): Self::SystemData) {

        let mut should_engage_combat: bool = false;
        for event in input_events.read(&mut self.input_event_reader) {
            match event {
                InputEvent::KeyPressed { key_code: VirtualKeyCode::Return, .. } => {
                    should_engage_combat = true;
                },
                InputEvent::KeyPressed { key_code: VirtualKeyCode::Space, .. } => {
                    should_engage_combat = true;
                },
                InputEvent::KeyPressed { key_code: VirtualKeyCode::Right, .. } => {
                    should_engage_combat = true;
                },
                InputEvent::MouseButtonPressed(MouseButton::Left) => {
                    should_engage_combat = true;
                },
                _ => {},
            }
        }

        if !move_targets.is_empty() {
            should_engage_combat = false;
        }

        for event in dialogue_completed_events.read(&mut self.dialogue_completed_event_reader) {
            if let Some(owner) = event.owner {
                if let Some(map_root) = map_roots.get(owner) {
                    if map_root.point_idx == current_state.max_point {
                        if current_state.max_stage == MapStage::PreDialogue {
                            current_state.max_stage = MapStage::Combat;
                        } else {
                            current_state.max_stage = MapStage::Complete;
                        }
                        should_engage_combat = false;
                    }
                }
            }
        }


        if !points.is_empty() {
            for (pawn_ent, mut pawn, _) in (&entities, &mut map_pawns, transforms.mask()).join() {
                if Principal::is_root_engaged(&parents, &principals, pawn_ent) != Some(true) {
                    if let Some((map_root, root_ent)) = get_root::<MapRoot, _, _>(&parents, &map_roots, pawn_ent) {
                        if map_root.point_idx == current_state.max_point {
                            if current_state.max_stage == MapStage::PreDialogue {
                                if let Some(pre_dialogue) = MapPoint::pre_dialogue(&points, &dialogue_store, map_root.point_idx) {
                                    show_dialogue_events.single_write(
                                        ShowDialogueDisplayEvent {
                                            dialogue: pre_dialogue.clone(),
                                            owner: Some(root_ent),
                                            start_idx: 0,
                                            principal: true,
                                        }
                                    );
                                    continue;
                                } else {
                                    current_state.max_stage = MapStage::Combat;
                                    save_current(&current_state);
                                }
                            }
                            if current_state.max_stage == MapStage::Combat {
                                if let Some(combat_data) = MapPoint::combat(&points, &combat_store, map_root.point_idx) {
                                    if should_engage_combat {
                                        *engage_combat = Some(EngageCombat { combat_data: combat_data.clone(), point_idx: map_root.point_idx });
                                    }
                                } else {
                                    current_state.max_stage = MapStage::PostDialogue;
                                    save_current(&current_state);
                                }
                            }
                            if current_state.max_stage == MapStage::PostDialogue {
                                if let Some(post_dialogue) = MapPoint::post_dialogue(&points, &dialogue_store, map_root.point_idx) {
                                    show_dialogue_events.single_write(
                                        ShowDialogueDisplayEvent {
                                            dialogue: post_dialogue.clone(),
                                            owner: Some(root_ent),
                                            start_idx: 0,
                                            principal: true,
                                        }
                                    );
                                    continue;
                                } else {
                                    current_state.max_stage = MapStage::Complete;
                                    save_current(&current_state);
                                }
                            }
                            if current_state.max_stage == MapStage::Complete {
                                current_state.max_point += 1;
                                current_state.max_stage = MapStage::PreDialogue;
                                current_state.master_health_mul += 0.2;
                                current_state.master_charge_mul += 0.2;
                                save_current(&current_state);
                            }
                        }
                    }
                }
            }
        }
    }
}