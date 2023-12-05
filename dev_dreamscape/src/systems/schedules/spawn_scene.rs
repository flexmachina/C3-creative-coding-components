use bevy_ecs::prelude::*;
use bevy_ecs::schedule::{ScheduleLabel};
use crate::assets::Assets;
use crate::components::{
    FloorBox, 
    FreeBox, 
    Player, 
    PostProcessor,
    Skybox, 
    //PlayerTarget
};

#[derive(ScheduleLabel, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Label;

pub fn new_spawn_scene_schedule() -> (Schedule, Label) {
    let mut schedule = Schedule::default();
    schedule
        .add_system(Assets::load.run_if(run_once()))
        .add_system(Skybox::spawn.run_if(run_once()))
        .add_system(FreeBox::spawn.run_if(run_once()))
        .add_system(FloorBox::spawn.run_if(run_once()))
        .add_system(Player::spawn.run_if(run_once()))
        //.add_system(PlayerTarget::spawn.run_if(run_once()))
        .add_system(PostProcessor::spawn
            .run_if((|player: Query<&Player>| !player.is_empty())
                .and_then(run_once()))
        );
    (schedule, Label)
}
