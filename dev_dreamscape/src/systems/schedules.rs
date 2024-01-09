use bevy_ecs::prelude::*;
use bevy_ecs::schedule::ScheduleLabel;
use crate::systems::{
        escape_on_exit,
        //grab_cursor,
        resize_device,
        update_input_state,
        update_frame_time,
        update_physics,
        render,
        prepare_render_pipelines,
};
use crate::assets::Assets;
use crate::components::{
    FloorBox, 
    FreeBox, 
    Player, 
    Skybox, 
    //PlayerTarget
};
use crate::components::{
    PhysicsBody, 
};


#[derive(ScheduleLabel, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct SpawnLabel;

pub fn new_spawn_scene_schedule() -> (Schedule, SpawnLabel) {
    let mut schedule = Schedule::default();
    schedule
        //.add_system(Assets::load.run_if(run_once()))
        .add_system(prepare_render_pipelines.run_if(run_once()))
        .add_system(Skybox::spawn.run_if(run_once()))
        .add_system(FreeBox::spawn.run_if(run_once()))
        .add_system(FloorBox::spawn.run_if(run_once()))
        .add_system(Player::spawn.run_if(run_once()));
        //.add_system(PlayerTarget::spawn.run_if(run_once()))
    (schedule, SpawnLabel)
}


#[derive(ScheduleLabel, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct PreupdateLabel;

pub fn new_preupdate_schedule() -> (Schedule, PreupdateLabel) {
    let mut schedule = Schedule::default();
    schedule
        .add_systems((
            escape_on_exit,
            //grab_cursor,
            resize_device,
            update_input_state,
            update_frame_time,
        ));
    (schedule, PreupdateLabel)
}


#[derive(ScheduleLabel, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct UpdateLabel;

pub fn new_update_schedule() -> (Schedule, UpdateLabel) {
    let mut schedule = Schedule::default();
    schedule
        .add_system(update_physics)
        .add_system(PhysicsBody::sync.after(update_physics))
        .add_system(Player::update.after(update_physics))
        //.add_system(PlayerTarget::update.after(Player::update))
        //.add_system(Grab::grab_or_release.after(Player::update))
        //.add_system(PhysicsBody::grab_start_stop.after(Player::update))
        //.add_system(PhysicsBody::update_grabbed.after(PhysicsBody::grab_start_stop))
        .add_system(FreeBox::spawn_by_player.after(Player::update));
    (schedule, UpdateLabel)
}


#[derive(ScheduleLabel, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct RenderLabel;

pub fn new_render_schedule() -> (Schedule, RenderLabel) {
    let mut schedule = Schedule::default();
    schedule
        .add_system(render);
    (schedule, RenderLabel)
}







