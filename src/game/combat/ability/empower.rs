use amethyst::{
    core::SystemDesc,
    ecs::prelude::*,
};

use crate::game::character::*;

use super::*;

define_ability!(
    EmpowerAbility,
    EmpowerAbilitySystem,
    EmpowerAbilitySystemDesc,
    "Empower",
    "Empowers an ally. Empowered drones do 50% more damage.",
    50.0,
    0,
    AbilityTargetType::Friendly,
    AbilityTargetArea::Single,
    AbilityUsability::Unique(&[
    BlitzDrone::character_id(),
    GunnerSpacebotDrone::character_id(),
    ModelXDrone::character_id(),
    ]),
    [
        AbilityActionDesc::Wave(0, 0.1, None),
        AbilityActionDesc::Wait(0.1),
        AbilityActionDesc::Wave(0, 0.1, None),
        AbilityActionDesc::Wait(0.1),
        AbilityActionDesc::Wave(0, 0.1, None),
        AbilityActionDesc::InflictStatus(StatusInflictDesc {
            ty: StatusType::Empower,
            turns: 3,
            chance: 1.0,
        }),
        AbilityActionDesc::Wait(0.1)
    ]
);