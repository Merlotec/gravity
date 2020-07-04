use amethyst::{
    core::SystemDesc,
    ecs::prelude::*,
};

use crate::game::character::*;

use super::*;

define_ability!(
    OverclockAbility,
    OverclockAbilitySystem,
    OverclockAbilitySystemDesc,
    "Overclock",
    "Overclocks an ally. Overclocked drones get an extra turn.",
    40.0,
    0,
    AbilityTargetType::Friendly,
    AbilityTargetArea::Single,
    AbilityUsability::Unique(&[
    SupporterSpacebotDrone::character_id(),
    DefenderDrone::character_id(),
    ]),
    [
        AbilityActionDesc::Wave(0, 0.5, None),
        AbilityActionDesc::InflictStatus(StatusInflictDesc {
            ty: StatusType::Overclocked,
            turns: 1,
            chance: 1.0,
        }),
        AbilityActionDesc::Wait(0.7)
    ]
);

define_ability!(
    SystemOverclockAbility,
    SystemOverclockAbilitySystem,
    SystemOverclockAbilitySystemDesc,
    "System Overclock",
    "Overclocks all allies.",
    150.0,
    0,
    AbilityTargetType::Friendly,
    AbilityTargetArea::All,
    AbilityUsability::Unique(&[
    SupporterSpacebotDrone::character_id(),
    DefenderDrone::character_id(),
    ]),
    [

        AbilityActionDesc::Wave(0, 0.5, None),
        AbilityActionDesc::Wait(0.7),
        AbilityActionDesc::Wave(0, 0.5, None),
        AbilityActionDesc::InflictStatus(StatusInflictDesc {
            ty: StatusType::Overclocked,
            turns: 1,
            chance: 1.0,
        }),
        AbilityActionDesc::Wait(0.7)
    ]
);