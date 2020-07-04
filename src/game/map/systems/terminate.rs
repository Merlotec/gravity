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
use crate::game::ui::dialogue::{DialogueCompletedEvent, ShowDialogueDisplayEvent, NavigateDialogueEvent};
use crate::game::combat::process::Principal;
use crate::state::map_state::MapState;
use crate::game::map::systems::movement::MoveTarget;
use space_render::Star;

pub struct TerminationTimer(f32);

#[derive(Debug, new, SystemDesc)]
#[system_desc(name(TerminateSystemDesc))]
pub struct TerminateSystem {
    #[system_desc(event_channel_reader)]
    dialogue_completed_event_reader: ReaderId<DialogueCompletedEvent>,
}

impl<'s> System<'s> for TerminateSystem {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, Parent>,
        WriteStorage<'s, Principal>,
        WriteStorage<'s, Transform>,
        ReadStorage<'s, Star>,
        ReadStorage<'s, MapRoot>,
        Write<'s, CurrentState>,
        Read<'s, Time>,
        Write<'s, Option<TerminationTimer>>,
    );

    fn run(&mut self, (entities, parents, mut principals, mut transforms, mut stars, roots, mut current_state, time, mut termination): Self::SystemData) {
        for (ent, root) in (&entities, &roots).join() {
            if current_state.max_point > 10 && termination.is_none() {
                Principal::try_root_engage(&parents, &mut principals, ent, std::any::TypeId::of::<Self>());
                *termination = Some(TerminationTimer(3.0));
            }
        }
        if let Some(timer) = termination.as_mut(){
            timer.0 -= time.delta_seconds();
            if timer.0 < 0.0 {
                save_current(&CurrentState::default());
                unreachable!();
            }
            for (star, transform) in (&stars, &mut transforms).join() {
                transform.set_scale(*transform.scale() * (1.0 + time.delta_seconds()));
            }
        }
    }
}