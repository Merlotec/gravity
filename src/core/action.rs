use amethyst::ecs::{Component, DenseVecStorage, Entity, ReadStorage};

/// Describes the invocation method to be used.
/// Principal actions are actions which block user input, thus only one can run at a time.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Invoke {
    /// Must be unique principal in at invoke time in order to be executed.
    AsPrincipalUnique,
    /// Set as a principal action but does not have to be principal.
    /// This should be used when 'delegating' - if one principal activity wishes to invoke another
    /// one, the action should be `AsPrincipalUnchecked` as the action will otherwise be blocked due
    /// to the invoking activity running as principal.
    AsPrincipalUnchecked,
    /// Executes as a background activity which does not 'block' high level control flow.
    AsBackground,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Action<T: Send + Sync> {
    pub data: T,
    pub owner: Option<Entity>,
    pub invoke: Invoke,
    pub root: Entity,
}

impl<T: Send + Sync> Action<T> {
    pub fn new(data: T, invoke: Invoke, root: Entity) -> Self {
        Self {
            data,
            owner: None,
            invoke,
            root,
        }
    }

    pub fn new_owned(data: T, owner: Option<Entity>, invoke: Invoke, root: Entity) -> Self {
        Self {
            data,
            owner,
            invoke,
            root,
        }
    }

    pub fn is_principal(&self) -> bool {
        match self.invoke {
            Invoke::AsPrincipalUnchecked => true,
            Invoke::AsPrincipalUnique => true,
            Invoke::AsBackground => false,
        }
    }
}

impl<T: Send + Sync + 'static> Component for Action<T> {
    type Storage = DenseVecStorage<Self>;
}