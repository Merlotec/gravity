use std::marker::PhantomData;

impl Component for StatusEffect {
    type Storage = DenseVecStorage<Self>;
}


use amethyst::{
    core::{
        math::Vector2,
        Named,
        Parent,
        ParentHierarchy,
        SystemDesc,
        Time,
        Transform,
    },
    ecs::prelude::*,
    shrev::{
        EventChannel,
        ReaderId,
    },
    ui::UiTransform,
};


#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum StatusType {
    Scramble,
    Overclocked,
    Unstable,
    Defend,
    Empower,
    Focus,
}

impl StatusType {
    pub fn all() -> Vec<StatusType> {
        vec![
            StatusType::Scramble,
            StatusType::Overclocked,
            StatusType::Unstable,
            StatusType::Defend,
            StatusType::Empower,
            StatusType::Focus,
        ]
    }
}

pub struct StatusData {
    name: String,
    desc: String,
    ty: StatusType,
}

pub struct StatusEffect {
    pub data: StatusData,
    pub turns: i32,
    pub strength: f32,
}

use crate::game::character::{Character, LastDamaged, WeaponSlot};
use crate::game::combat::{TickTurn, Team};

#[derive(Debug, new, SystemDesc)]
#[system_desc(name(StatusSystemDesc))]
pub struct StatusSystem {
    #[system_desc(event_channel_reader)]
    tick_turn_event_reader: ReaderId<TickTurn>,
}

impl<'s> System<'s> for StatusSystem {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, Parent>,
        ReadStorage<'s, Team>,
        WriteStorage<'s, Character>,
        Read<'s, EventChannel<TickTurn>>,
    );

    fn run(&mut self, (entities, parents, teams, mut characters, tick_turn_events): Self::SystemData) {
        for event in tick_turn_events.read(&mut self.tick_turn_event_reader) {
            for (character_ent, mut character) in (&entities, &mut characters).join() {
                if let Some((team, team_ent)) = Team::get_team(&parents, &teams, character_ent) {
                    if team == event.next_team.other() {
                        character.decrement_status(StatusType::Unstable);
                        character.decrement_status(StatusType::Scramble);
                        character.decrement_status(StatusType::Empower);
                        character.decrement_status(StatusType::Focus);
                    } else {
                        character.decrement_status(StatusType::Defend);
                    }
                }
            }
        }

    }
}