use crate::components::transform::Transform;
use crate::components::{PhysicsBody, PhysicsBodyParams, Player};
use crate::components::ModelSpec;
use crate::input::Input;
use crate::math::{Vec3f,UnitQuatf};
use crate::physics_world::PhysicsWorld;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct FreeBox;

impl FreeBox {
    pub fn spawn(
        mut commands: Commands,
        mut physics: ResMut<PhysicsWorld>,
    ) {
        let pos = Vec3f::y_axis().xyz() * 10.0;
        commands.spawn(Self::new_components(pos, &mut physics));
    }

    pub fn spawn_by_player(
        player: Query<&Transform, With<Player>>,
        mut commands: Commands,
        mut physics: ResMut<PhysicsWorld>,
        input: Res<Input>,
    ) {
        if input.space_just_pressed {
            let player_transform = player.single();
            let pos = player_transform.position() + player_transform.forward().xyz() * 5.0;
            commands.spawn(Self::new_components(pos, &mut physics));
        }
    }

    fn new_components(
        pos: Vec3f,
        physics: &mut PhysicsWorld,
    ) -> (FreeBox, PhysicsBody, Transform, ModelSpec) {
        let rot = UnitQuatf::identity();
        let scale = Vec3f::from_element(1.0);
        let physics_body = PhysicsBody::new(
            PhysicsBodyParams {
                pos,
                scale,
                rotation_axis: Vec3f::identity(),
                rotation_angle: 0.0,
                movable: true,
                collision_model: None,
                collision_ball: None,
                gravity_scale: Some(0.6),
                lin_vel: None,
                ang_vel: None,
            },
            physics,
        );
        let modelspec = ModelSpec::new(String::from("cube.obj"));
        let transform = Transform::new(pos, rot, scale);
        (FreeBox, physics_body, transform, modelspec)
    }
}
