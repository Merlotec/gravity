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
    winit::VirtualKeyCode,
};
use crate::game::character::Character;
use crate::game::map::{CurrentState, MapPawn, MapRoot, MapPoint};
use failure::_core::mem::take;
use crate::core::{get_root, get_root_mut};
use crate::game::combat::process::Principal;

pub const MOVE_SPEED: f32 = 6.0;

#[derive(Debug, new, SystemDesc)]
#[system_desc(name(MapMovementSystemDesc))]
pub struct MapMovementSystem {
    #[system_desc(event_channel_reader)]
    input_event_reader: ReaderId<InputEvent<StringBindings>>,

    #[system_desc(event_channel_reader)]
    move_pawn_event_reader: ReaderId<MovePawnEvent>,
}

#[derive(Debug, Copy, Clone, Component)]
pub struct MoveTarget(pub Vector3<f32>);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct MovePawnEvent {
    pub point_idx: usize,
    pub pawn_ent: Entity,
}

impl<'s> System<'s> for MapMovementSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Parent>,
        ReadStorage<'s, Principal>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, MapRoot>,
        WriteStorage<'s, MapPawn>,
        ReadStorage<'s, MapPoint>,
        WriteStorage<'s, MoveTarget>,
        Write<'s, CurrentState>,
        Read<'s, EventChannel<InputEvent<StringBindings>>>,
        Write<'s, EventChannel<MovePawnEvent>>,
        Read<'s, Time>,
    );

    fn run(&mut self, (entities, parents, principals, mut transforms, mut map_roots, mut map_pawns, points, mut move_targets, mut current_state, input_events, mut move_pawn_events, time): Self::SystemData) {
        for (pawn_ent, mut map_pawn) in (&entities, &mut map_pawns).join() {
            if !map_pawn.0 {
                if let Some((root, _)) = get_root::<MapRoot, _, _>(&parents, &map_roots, pawn_ent) {
                    if let Some(pos) = MapPoint::position_of(&points, root.point_idx) {
                        let mut trans = Transform::from(pos);
                        trans.set_scale(Vector3::new(30.0, 30.0, 30.0));
                        trans.set_rotation_y_axis(-std::f32::consts::FRAC_PI_2);
                        transforms.insert(pawn_ent, trans);
                        map_pawn.0 = true;
                    }
                }
            }
        }

        for event in input_events.read(&mut self.input_event_reader) {
            for (mut map_root) in (&mut map_roots).join() {
                match event {
                    InputEvent::KeyPressed { key_code: VirtualKeyCode::Right, .. } => {
                        if map_root.point_idx < current_state.max_point {
                            move_pawn_events.single_write(
                                MovePawnEvent {
                                    pawn_ent: map_root.pawn_ent,
                                    point_idx: map_root.point_idx + 1,
                                }
                            );

                        }
                    },
                    InputEvent::KeyPressed { key_code: VirtualKeyCode::Left, .. } => {
                        if map_root.point_idx > 0 {
                            move_pawn_events.single_write(
                                MovePawnEvent {
                                    pawn_ent: map_root.pawn_ent,
                                    point_idx: map_root.point_idx - 1,
                                }
                            );
                        }
                    },
                    _ => {},
                }
            }
        }
        
        for event in move_pawn_events.read(&mut self.move_pawn_event_reader) {
            if Principal::is_root_engaged(&parents, &principals, event.pawn_ent) != Some(true) {
                if let Some(target) = MapPoint::position_of(&points, event.point_idx) {
                    if let Some((root, _)) = get_root_mut::<MapRoot, _, _>(&parents, &mut map_roots, event.pawn_ent) {
                        move_targets.insert(event.pawn_ent, MoveTarget(target));
                        root.point_idx = event.point_idx;
                    }
                }
            }
        }

        let mut to_remove: Vec<Entity> = Vec::new();
        for (pawn_ent, pawn, mut transform, target) in (&entities, &map_pawns, &mut transforms, &move_targets).join() {
            let target: Vector3<f32> = target.0;
            let current: Vector3<f32> = *transform.translation();
            let delta: Vector3<f32> = target - current;
            let abs: f32 = delta.norm();
            let v: f32 = delta.norm() * MOVE_SPEED + 500.0;
            let adj: f32 = v * time.delta_seconds();
            if adj >= abs {
                transform.set_translation(target);
                // We've arrived at the target point.
                to_remove.push(pawn_ent);
            } else {
                transform.prepend_translation(delta.normalize() * adj);
            }
        }

        for ent in to_remove {
            move_targets.remove(ent);
        }
    }
}