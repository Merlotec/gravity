use amethyst::{
    core::SystemDesc,
    ecs::prelude::*,
};

use crate::game::character::*;

use super::*;

define_ability!(
    EnergyOverloadAbility,
    EnergyOverloadAbilitySystem,
    EnergyOverloadAbilitySystemDesc,
    "Energy Overload",
    "Charges all allies, also inflicting them with unstable.",
    100.0,
    0,
    AbilityTargetType::Friendly,
    AbilityTargetArea::All,
    AbilityUsability::Unique(&[ChargeSpacebotDrone::character_id(), SparkyDrone::character_id()]),
    [
        AbilityActionDesc::Wave(0, 0.5, None),
        AbilityActionDesc::Wait(0.2),
        AbilityActionDesc::Wave(0, 0.5, None),
       AbilityActionDesc::Charge(150.0, None),

        AbilityActionDesc::InflictStatus(StatusInflictDesc {
            ty: StatusType::Unstable,
            turns: 2,
            chance: 1.0,
        }),
       AbilityActionDesc::Wait(0.7)
    ]
);