use amethyst::{
    core::SystemDesc,
    ecs::prelude::*,
};

use crate::game::character::*;

use super::*;

define_ability!(
    CorruptAbility,
    CorruptAbilitySystem,
    CorruptAbilitySystemDesc,
    "Corrupt",
    "Attempts to Scramble or inflict Unstable on all enemies.",
    150.0,
    0,
    AbilityTargetType::Enemy,
    AbilityTargetArea::All,
        AbilityUsability::Unique(&[SupporterSpacebotDrone::character_id(), DefenderDrone::character_id()]),
    [
       AbilityActionDesc::Wave(0, 0.3, None),
       AbilityActionDesc::Fire(FireDesc::laser(5.0, Element::Plasma, 0.8, 0.2), 0),
       AbilityActionDesc::Wait(0.22),
       AbilityActionDesc::Wave(0, 0.35, None),
        AbilityActionDesc::Wait(0.12),
        AbilityActionDesc::Fire(FireDesc::laser(5.0, Element::Plasma, 0.8, 0.2), 0),
        AbilityActionDesc::Wait(0.3),
        AbilityActionDesc::Wave(0, 0.4, None),
        AbilityActionDesc::Wait(0.12),
        AbilityActionDesc::Fire(FireDesc::laser(5.0, Element::Plasma, 0.8, 0.2), 0),
        AbilityActionDesc::InflictStatus(StatusInflictDesc {
            ty: StatusType::Scramble,
            turns: 1,
            chance: 0.3,
        }),
        AbilityActionDesc::InflictStatus(StatusInflictDesc {
            ty: StatusType::Unstable,
            turns: 1,
            chance: 0.7,
        }),

         AbilityActionDesc::Wait(1.2)
]
);