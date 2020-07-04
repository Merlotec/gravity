use amethyst::{
    core::{
        SystemBundle,
        SystemDesc,
    },
    ecs::prelude::{DispatcherBuilder, World},
    Error,
};

pub mod control;
pub mod map;
pub mod combat;
pub mod character;
pub mod ui;

pub struct CoreGameBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for CoreGameBundle {
    fn build(
        self,
        world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        builder.add(
            crate::game::ui::dialogue::DialogueUiSystemDesc::default()
                .build(world),
            "dialogue_ui",
            &[],
        );

        Ok(())
    }
}