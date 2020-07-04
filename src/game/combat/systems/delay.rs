use std::any::TypeId;

use amethyst::{
    core::{
        Parent,
        SystemDesc,
        Time,
    },
    ecs::{
        prelude::*,
        storage::{
            GenericReadStorage,
            GenericWriteStorage,
        },
    },
};

use crate::game::combat::process::Principal;

/// Must be added to the same entity which contains principal component.
#[derive(Debug, Copy, Clone, PartialEq, Component)]
pub struct Delay {
    pub time: f32,
}

impl Delay {
    pub fn principal_delay(
        parents: &impl GenericReadStorage<Component=Parent>,
        principals: &mut impl GenericWriteStorage<Component=Principal>,
        delays: &mut impl GenericWriteStorage<Component=Delay>,
        entity: Entity,
        delay: f32,
    ) -> Option<bool> {
        let eng = Principal::try_root_engage(
            parents,
            principals,
            entity,
            TypeId::of::<DelaySystem>(),
        );
        if eng == Some(true) {
            delays.insert(entity, Delay { time: delay });
            Some(true)
        } else {
            eng
        }
    }
}

#[derive(Debug, new, SystemDesc)]
#[system_desc(name(DelaySystemDesc))]
pub struct DelaySystem;

impl<'s> System<'s> for DelaySystem {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, Parent>,
        WriteStorage<'s, Delay>,
        WriteStorage<'s, Principal>,
        Read<'s, Time>,
    );

    fn run(&mut self, (entities, parents, mut delays, mut principals, time): Self::SystemData) {
        let mut to_remove: Vec<Entity> = Vec::new();
        for (entity, delay) in (&entities, &mut delays).join() {
            delay.time -= time.delta_seconds();
            if delay.time <= 0.0 {
                to_remove.push(entity);
            }
        }

        for entity in to_remove {
            delays.remove(entity);
            Principal::try_root_disengage(
                &parents,
                &mut principals,
                entity,
                TypeId::of::<Self>(),
            );
        }
    }
}