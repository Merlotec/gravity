use amethyst::{
    core::SystemDesc,
    ecs::prelude::*,
};

use crate::game::character::Character;
use crate::game::character::*;

use super::*;

define_ability!(
    NukeAbility,
    NukeAbilitySystem,
    NukeAbilitySystemDesc,
    "Nuke",
    "Does massive damage to a single target.",
    900.0,
    0,
    AbilityTargetType::Enemy,
    AbilityTargetArea::Single,
    AbilityUsability::Role(CharacterRole::Master),
    [
       AbilityActionDesc::Fire(FireDesc::torpedo(300.0, Element::Plasma, 1.0),0),
       AbilityActionDesc::Wait(3.5)
    ]
);