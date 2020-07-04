use std::any::TypeId;
use std::collections::HashMap;

use amethyst::{
    ecs::prelude::*,
};

use super::action::{
    Action,
    Invoke,
};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Activity<A: Send + Sync> {
    pub data: A,
    pub system: TypeId,
    pub principal: bool,
    pub root: Entity,
}

impl<A: Send + Sync> Activity<A> {
    pub fn new(data: A, system: TypeId, principal: bool, root: Entity) -> Self {
        Self {
            data,
            system,
            principal,
            root,
        }
    }
}

impl<A: Send + Sync + 'static> Component for Activity<A> {
    type Storage = DenseVecStorage<Self>;
}

pub struct ActivityState {
    background: HashMap<TypeId, i32>,
    principals: HashMap<TypeId, i32>,
}

impl Component for ActivityState {
    type Storage = DenseVecStorage<Self>;
}

impl ActivityState {
    pub fn new() -> Self {
        Self {
            background: HashMap::new(),
            principals: HashMap::new(),
        }
    }

    /// Removes all registered activities.
    /// This should be done with the intention of recomputing all the active activities.
    pub fn clear(&mut self) {
        for (_, val) in self.background.iter_mut() {
            *val = 0;
        }
        for (_, val) in self.principals.iter_mut() {
            *val = 0;
        }
    }

    pub fn clear_for(&mut self, system: &TypeId) {
        if let Some(val) = self.background.get_mut(system) {
            *val = 0;
        }
        if let Some(val) = self.principals.get_mut(system) {
            *val = 0;
        }
    }

    pub fn set_background(&mut self, system: TypeId, count: i32) {
        self.background.insert(system, count);
    }

    pub fn set_principal(&mut self, system: TypeId, count: i32) {
        self.principals.insert(system, count);
    }

    pub fn increment_background(&mut self, system: TypeId) {
        if let Some(existing) = self.background.get_mut(&system) {
            *existing += 1;
        } else {
            self.background.insert(system, 1);
        }
    }

    pub fn increment_principal(&mut self, system: TypeId) {
        if let Some(existing) = self.principals.get_mut(&system) {
            *existing += 1;
        } else {
            self.principals.insert(system, 1);
        }
    }

    pub fn has_principal(&self) -> bool {
        for (_, val) in self.principals.iter() {
            if *val > 0 {
                return true;
            }
        }
        false
    }

    pub fn can_invoke<T: Send + Sync>(&self, action: &Action<T>) -> bool {
        match action.invoke {
            Invoke::AsBackground => true,
            Invoke::AsPrincipalUnchecked => true,
            Invoke::AsPrincipalUnique => !self.has_principal(),
        }
    }
}


/// Updates activity 'silently' on drop.
pub struct ActivityAggregator<'a, 's, A: Send + Sync + 'static> {
    system: TypeId,
    activities: &'a mut WriteStorage<'s, Activity<A>>,
    activity_states: &'a mut WriteStorage<'s, ActivityState>,
}

impl<'a, 's, A: Send + Sync + 'static> ActivityAggregator<'a, 's, A> {
    pub fn aggregate(system: TypeId, activities: &'a mut WriteStorage<'s, Activity<A>>, activity_states: &'a mut WriteStorage<'s, ActivityState>) -> Self {
        Self {
            system,
            activities,
            activity_states,
        }
    }

    pub fn can_invoke<T: Send + Sync>(&self, action: &Action<T>) -> bool {
        if let Some(activity_state) = self.activity_states.get(action.root) {
            activity_state.can_invoke(action)
        } else {
            false
        }
    }

    pub fn try_invoke<T: Send + Sync, F>(&mut self, action: &Action<T>, f: F)
        where F: FnOnce(&Action<T>) -> Option<(Entity, A)> {
        if self.can_invoke(&action) {
            if let Some((entity, activity_data)) = f(action) {
                let activity: Activity<A> = Activity::new(activity_data, self.system, action.is_principal(), action.root);
                self.activities.insert(entity, activity);
            }
        }
    }

    pub fn update_activity_state(&mut self) {
        for activity_state in self.activity_states.join() {
            activity_state.clear_for(&self.system);
        }
        for activity in self.activities.join() {
            if let Some(activity_state) = self.activity_states.get_mut(activity.root) {
                if activity.principal {
                    activity_state.increment_principal(self.system);
                } else {
                    activity_state.increment_background(self.system);
                }
            }
        }
    }
}