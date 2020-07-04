use amethyst::{
    core::SystemDesc,
    ecs::prelude::*,
};

use crate::game::character::Character;
use crate::game::character::*;

use super::*;

define_ability!(
    JammerAbility,
    JammerAbilitySystem,
    JammerAbilitySystemDesc,
    "Jammer",
    "Scrambles all enemy drones. Essentially skips the next enemy turn.",
    600.0,
    0,
    AbilityTargetType::Enemy,
    AbilityTargetArea::All,
    AbilityUsability::Role(CharacterRole::Master),
    [
        AbilityActionDesc::Wave(0, 0.5, None),
        AbilityActionDesc::Wait(0.1),
        AbilityActionDesc::Wave(0, 0.5, None),
        AbilityActionDesc::Wait(0.1),
        AbilityActionDesc::Wave(0, 0.5, None),
        AbilityActionDesc::Wait(0.1),
        AbilityActionDesc::Wave(0, 0.5, None),
        AbilityActionDesc::Wait(0.1),
        AbilityActionDesc::Wave(0, 0.5, None),
        AbilityActionDesc::Wait(0.1),
             AbilityActionDesc::InflictStatus(StatusInflictDesc {
            ty: StatusType::Scramble,
            turns: 1,
            chance: 1.0,
        }),
        AbilityActionDesc::Wait(0.7)
    ]
);