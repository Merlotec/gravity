use amethyst::{
    core::SystemDesc,
    ecs::prelude::*,
};

use crate::game::character::*;

use super::*;

define_ability!(
    FocusAbility,
    FocusAbilitySystem,
    FocusAbilitySystemDesc,
    "Focus",
    "Doubles the aim stat of an ally.",
    20.0,
    0,
    AbilityTargetType::Friendly,
    AbilityTargetArea::Single,
    AbilityUsability::Unique(&[
    SpanglesDrone::character_id(),
    SpacebotDrone::character_id(),
    SparkyDrone::character_id(),
    ChargeSpacebotDrone::character_id(),
    ModelXDrone::character_id(),
    ]),
    [
        AbilityActionDesc::Wave(0, 0.1, None),
        AbilityActionDesc::Wait(0.1),
        AbilityActionDesc::Wave(0, 0.1, None),
        AbilityActionDesc::Wait(0.1),
        AbilityActionDesc::Wave(0, 0.1, None),
        AbilityActionDesc::InflictStatus(StatusInflictDesc {
            ty: StatusType::Focus,
            turns: 3,
            chance: 1.0,
        }),
        AbilityActionDesc::Wait(0.1)
    ]
);