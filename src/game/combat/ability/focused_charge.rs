use amethyst::{
    core::SystemDesc,
    ecs::prelude::*,
};

use crate::game::character::*;

use super::*;

define_ability!(
    FocusedChargeAbility,
    FocusedChargeAbilitySystem,
    FocusedChargeAbilitySystemDesc,
    "Focused Charge",
    "Accumulates Charge for selected allied drones. Less effective than Charge.",
    80.0,
    0,
    AbilityTargetType::Friendly,
    AbilityTargetArea::Flexible,
    AbilityUsability::Unique(&[SparkyDrone::character_id(), ChargeSpacebotDrone::character_id(), DefenderDrone::character_id(), SupporterSpacebotDrone::character_id()]),
    [
        AbilityActionDesc::Wave(0, 1.0, None),
        AbilityActionDesc::Charge(100.0, None),
        AbilityActionDesc::Wait(0.7)
    ]
);