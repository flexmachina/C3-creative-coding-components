use bevy_ecs::prelude::{Schedule};
use bevy_ecs::schedule::ScheduleLabel;
use crate::components::{
    //PhysicsBody, 
    Player, PostProcessor, 
    //PlayerTarget, Grab, FreeBox
};
use crate::systems::*;

#[derive(ScheduleLabel, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Label;

pub fn new_update_schedule() -> (Schedule, Label) {
    let mut schedule = Schedule::default();
    schedule
        .add_system(update_physics)
        //.add_system(PhysicsBody::sync.after(update_physics))
        .add_system(Player::update.after(update_physics))
        //.add_system(PlayerTarget::update.after(Player::update))
        //.add_system(Grab::grab_or_release.after(Player::update))
        //.add_system(PhysicsBody::grab_start_stop.after(Player::update))
        //.add_system(PhysicsBody::update_grabbed.after(PhysicsBody::grab_start_stop))
        //.add_system(FreeBox::spawn_by_player.after(Player::update))
        .add_system(PostProcessor::update.after(Player::update));
        //.add_system(update_and_build_debug_ui.after(update_physics));
    (schedule, Label)
}
