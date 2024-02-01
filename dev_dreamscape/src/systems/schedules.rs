use bevy_ecs::prelude::*;
use bevy_ecs::schedule::ScheduleLabel;
use crate::frame_time::FrameTime;
use crate::math::{Vec3, Vec3f, UnitQuat};
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
use crate::components::{
    FloorBox, 
    FreeBox,
    Light, 
    Player,
    PlayerHands,
    Skybox,
    Transform,
    Rock
    //PlayerTarget
};
use crate::components::PhysicsBody;


#[derive(ScheduleLabel, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct SpawnLabel;

pub fn new_spawn_scene_schedule(webxr: bool) -> (Schedule, SpawnLabel) {
    let mut schedule = Schedule::default();
    schedule
        //.add_system(Assets::load.run_if(run_once()))
        .add_systems(prepare_render_pipelines.run_if(run_once()))
        .add_systems(Skybox::spawn.run_if(run_once()))
        .add_systems(FreeBox::spawn.run_if(run_once()))
        .add_systems(Rock::spawn_rock_field.run_if(run_once()))
        .add_systems(FloorBox::spawn.run_if(run_once()))
        .add_systems(Player::spawn.run_if(run_once()))
        .add_systems(Light::spawn.run_if(run_once()));
        //.add_system(PlayerTarget::spawn.run_if(run_once()))

    if webxr {
        schedule.add_systems(PlayerHands::spawn.run_if(run_once()));
    }
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
        .add_systems(update_lights)
        .add_systems(update_physics)
        .add_systems(PhysicsBody::sync.after(update_physics))
        .add_systems(Player::update.after(update_physics))
        //.add_system(PlayerTarget::update.after(Player::update))
        //.add_system(Grab::grab_or_release.after(Player::update))
        //.add_system(PhysicsBody::grab_start_stop.after(Player::update))
        //.add_system(PhysicsBody::update_grabbed.after(PhysicsBody::grab_start_stop))
        .add_systems(FreeBox::spawn_by_player.after(Player::update));
    (schedule, UpdateLabel)
}

#[derive(ScheduleLabel, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct HandUpdateLabel;

pub fn new_hand_update_schedule() -> (Schedule, HandUpdateLabel) {
    let mut schedule = Schedule::default();
    schedule
        .add_systems((
            PlayerHands::update,
        ));
    (schedule, HandUpdateLabel)
}

#[derive(ScheduleLabel, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct CameraUpdateLabel;

pub fn new_camera_update_schedule() -> (Schedule, CameraUpdateLabel) {
    let mut schedule = Schedule::default();
    schedule
        .add_systems((
            Player::update_player_view_xr,
        ));
    (schedule, CameraUpdateLabel)
}


#[derive(ScheduleLabel, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct RenderLabel;

pub fn new_render_schedule() -> (Schedule, RenderLabel) {
    let mut schedule = Schedule::default();
    schedule
        .add_systems(render);
    (schedule, RenderLabel)
}

fn update_lights(
    frame_time: Res<FrameTime>,
    mut lights_query: Query<(&Light, &mut Transform)>
) {
    // Rotate lights about y axis at origin
    let deg_per_second: f32 = 45.0;
    for (_, mut transform) in lights_query.iter_mut() {
        transform.translate_around(
            Vec3f::zeros(),
            UnitQuat::from_axis_angle(&Vec3::y_axis(), deg_per_second.to_radians() * frame_time.delta)
        );
    }
}
