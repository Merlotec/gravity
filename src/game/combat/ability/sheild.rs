



use amethyst::{
    core::SystemDesc,
    ecs::prelude::*,
};

use crate::game::character::Character;
use crate::game::character::*;

use super::*;

define_ability!(
    ReinforceAbility,
    ReinforceAbilitySystem,
    ReinforceAbilitySystemDesc,
    "Reinforce",
    "Causes a single drone to reinforce itself, reducing the damage they receive by 50% by triggering the defend status.",
    20.0,
    0,
    AbilityTargetType::Friendly,
    AbilityTargetArea::Single,
    AbilityUsability::Unique(&[SupporterSpacebotDrone::character_id(), DefenderDrone::character_id()]),
    [
        AbilityActionDesc::Wave(0, 0.5, None),
        AbilityActionDesc::Wait(0.1),
        AbilityActionDesc::Wave(0, 0.5, None),
             AbilityActionDesc::InflictStatus(StatusInflictDesc {
            ty: StatusType::Defend,
            turns: 1,
            chance: 1.0,
        }),
        AbilityActionDesc::Wave(0, 0.5, None),
        AbilityActionDesc::Wait(0.5)
    ]
);

define_ability!(
    ShieldAbility,
    ShieldAbilitySystem,
    ShieldAbilitySystemDesc,
    "Energy Shield",
    "Causes the drones to shield up, reducing the damage they receive by 50%.",
    100.0,
    0,
    AbilityTargetType::Friendly,
    AbilityTargetArea::All,
    AbilityUsability::Unique(&[SupporterSpacebotDrone::character_id(), DefenderDrone::character_id()]),
    [
        AbilityActionDesc::Wave(0, 0.5, None),
        AbilityActionDesc::Wait(0.1),
        AbilityActionDesc::Wave(0, 0.5, None),
        AbilityActionDesc::Wait(0.1),
        AbilityActionDesc::Wave(0, 0.5, None),
        AbilityActionDesc::Wait(0.1),
             AbilityActionDesc::InflictStatusFlexible(StatusInflictDesc {
            ty: StatusType::Defend,
            turns: 1,
            chance: 1.0,
        }, 3),
        AbilityActionDesc::Wave(0, 0.5, None),
        AbilityActionDesc::Wait(0.1),
        AbilityActionDesc::Wave(0, 0.5, None),
        AbilityActionDesc::Wait(0.1),
        AbilityActionDesc::Wave(0, 0.5, None),
        AbilityActionDesc::Wait(0.5)
    ]
);