use amethyst::{
    core::SystemDesc,
    ecs::prelude::*,
};

use crate::game::character::*;

use super::*;

define_ability!(
    SnipeAbility,
    SnipeAbilitySystem,
    SnipeAbilitySystemDesc,
    "Snipe",
    "Attack a single target. Never misses.",
    40.0,
    0,
    AbilityTargetType::Enemy,
    AbilityTargetArea::Single,
    AbilityUsability::Unique(&[GunnerSpacebotDrone::character_id(), BlitzDrone::character_id(), SpanglesDrone::character_id(), SpacebotDrone::character_id()]),
    [
       AbilityActionDesc::Fire(FireDesc::bullet(40.0, 1.0), 0),
       AbilityActionDesc::Wait(0.7)
    ]
);