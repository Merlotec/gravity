use std::any::TypeId;

use amethyst::{
    core::Parent,
    ecs::{
        prelude::*,
        storage::{
            GenericReadStorage,
            GenericWriteStorage,
        },
    },
};

use crate::core::{get_root, get_root_mut};

#[derive(Debug, Copy, Clone, )]
pub enum PrincipalProcess {
    Ability(Entity),
    Spawn(Entity),
    Other(Entity, TypeId),
}

/// Contains the current principal entity.
/// If this is populated, other abilities should not be used.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct Principal(Option<(Entity, TypeId)>);

impl Component for Principal {
    type Storage = DenseVecStorage<Self>;
}

impl Principal {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn current(&self) -> Option<(Entity, TypeId)> {
        self.0
    }

    #[inline]
    pub fn current_entity(&self) -> Option<Entity> {
        if let Some((entity, _)) = self.0 {
            Some(entity)
        } else {
            None
        }
    }

    #[inline]
    pub fn engage(&mut self, entity: Entity, system: TypeId) -> bool {
        if self.0.is_none() {
            self.0 = Some((entity, system));
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn force_engage(&mut self, entity: Entity, system: TypeId) -> Option<(Entity, TypeId)> {
        let old = self.0.take();
        self.0 = Some((entity, system));
        old
    }

    #[inline]
    pub fn is_engaged(&self) -> bool {
        self.0.is_some()
    }

    #[inline]
    pub fn disengage(&mut self, entity: Entity, system: TypeId) -> bool {
        if let Some(ent) = self.current() {
            if ent == (entity, system) {
                self.force_disengage();
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    #[inline]
    pub fn force_disengage(&mut self) -> Option<(Entity, TypeId)> {
        self.0.take()
    }

    /// Returns true if successful.
    pub fn try_root_engage<'s>(
        parents: &impl GenericReadStorage<Component=Parent>, principals: &mut impl GenericWriteStorage<Component=Principal>,
        entity: Entity, system: TypeId,
    ) -> Option<bool> {
        if let Some((principal, _)) = get_root_mut::<Principal, _, _>(
            parents, principals,
            entity,
        ) {
            Some(principal.engage(entity, system))
        } else {
            None
        }
    }

    pub fn is_root_engaged<'s>(
        parents: &impl GenericReadStorage<Component=Parent>, principals: &impl GenericReadStorage<Component=Principal>,
        entity: Entity,
    ) -> Option<bool> {
        if let Some((principal, _)) = get_root::<Principal, _, _>(
            parents, principals,
            entity,
        ) {
            Some(principal.is_engaged())
        } else {
            None
        }
    }

    /// Alerts the principal object that the action being performed on this entity is done.
    /// Th
    pub fn try_root_disengage<'s>(
        parents: &impl GenericReadStorage<Component=Parent>, principals: &mut impl GenericWriteStorage<Component=Principal>,
        entity: Entity, system: TypeId,
    ) -> Option<bool> {
        if let Some((principal, _)) = get_root_mut::<Principal, _, _>(
            parents, principals,
            entity,
        ) {
            Some(principal.disengage(entity, system))
        } else {
            None
        }
    }
}