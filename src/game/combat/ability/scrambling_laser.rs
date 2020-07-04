use amethyst::{
    core::SystemDesc,
    ecs::prelude::*,
};

use crate::game::character::Character;
use crate::game::character::*;

use super::*;

define_ability!(
    ScramblingLaserAbility,
    ScramblingLaserAbilitySystem,
    ScramblingLaserAbilitySystemDesc,
    "Scrambling Laser",
    "Strikes a single target with a laser. May cause Scramble.",
    120.0,
    0,
    AbilityTargetType::Enemy,
    AbilityTargetArea::Single,
    AbilityUsability::Unique(&[SpanglesDrone::character_id(),
    SpacebotDrone::character_id(),
    SparkyDrone::character_id(),
    ChargeSpacebotDrone::character_id()
    ]),
    [
       AbilityActionDesc::Fire(
           FireDesc {
                ty: FireType::Laser(Element::Plasma, 1.0, false),
                accuracy: 0.8,
                power: 70.0,
                effect: Some (
                    StatusInflictDesc {
                        turns: 1,
                        ty: StatusType::Scramble,
                        chance: 0.5,
                    }
                ),
           },
           0,
       ),
       AbilityActionDesc::Wait(0.7)
    ]
);