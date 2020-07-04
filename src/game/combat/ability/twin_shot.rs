use amethyst::{
    core::SystemDesc,
    ecs::prelude::*,
};

use crate::game::character::*;

use super::*;

define_ability!(
    TwinShotAbility,
    TwinShotAbilitySystem,
    TwinShotAbilitySystemDesc,
    "Twin Shot",
    "Fires two pairs of bullets with high accuracy and reasonable power.",
    20.0,
    0,
    AbilityTargetType::Enemy,
    AbilityTargetArea::Single,
    AbilityUsability::Unique(&[SpanglesDrone::character_id(), GunnerSpacebotDrone::character_id(), BlitzDrone::character_id(), ModelXDrone::character_id()]),
    [
        AbilityActionDesc::Fire(FireDesc::bullet(10.0, 0.8), 0),
        AbilityActionDesc::Fire(FireDesc::bullet(10.0, 0.8), 1),
        AbilityActionDesc::Wait(0.6),
        AbilityActionDesc::Fire(FireDesc::bullet(10.0, 0.8), 0),
        AbilityActionDesc::Fire(FireDesc::bullet(10.0, 0.8), 1),
        AbilityActionDesc::Wait(0.7)
    ]
);