use amethyst::ecs::prelude::*;

use crate::game::combat::ability::AbilityTarget;

pub struct TacticalSystem;

#[derive(Debug, Clone, PartialEq)]
pub struct AiAbilitySelectionQuery {
    pub result: Option<AiAbilitySelection>,
    pub targets: Vec<Entity>,
}

impl AiAbilitySelectionQuery {
    pub fn new(targets: Vec<Entity>) -> Self {
        Self {
            result: None,
            targets,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AiAbilitySelection {
    /// Set this to the absolute 'score' of this ability.
    /// The higher it is the more likely it is to be performed.
    pub score: f32,

    /// The target of this ability invocation.
    pub target: AbilityTarget,
}

impl Component for AiAbilitySelectionQuery {
    type Storage = DenseVecStorage<Self>;
}
