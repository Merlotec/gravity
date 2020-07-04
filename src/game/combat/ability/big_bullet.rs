use amethyst::{
    core::SystemDesc,
    ecs::prelude::*,
};

use crate::game::character::*;

use super::*;

define_ability!(
    BigBulletAbility,
    BigBulletAbilitySystem,
    BigBulletAbilitySystemDesc,
    "Big Bullet",
    "Strikes a target. The giant bullet causes the target to become Unstable. Unstable targets do more damage, but have reduced Accuracy",
    100.0,
    0,
    AbilityTargetType::Enemy,
    AbilityTargetArea::Single,
    AbilityUsability::Unique(&[SpacebotDrone::character_id(), SpanglesDrone::character_id()]),
    [
       //AbilityActionDesc::Fire(FireDesc::torpedo(50.0, Element::Kinetic, 0.95), 0),
       AbilityActionDesc::Fire(
           FireDesc {
                ty: FireType::Torpedo(Element::Plasma),
                accuracy: 95.0,
                power: 50.0,
                effect: Some (
                    StatusInflictDesc {
                        turns: 2,
                        ty: StatusType::Unstable,
                        chance: 1.0,
                    }
                ),
           },
           0,
       ),
       AbilityActionDesc::Wait(2.5)
    ]
);