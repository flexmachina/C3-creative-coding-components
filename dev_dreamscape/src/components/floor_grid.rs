use crate::components::transform::Transform;
use crate::components::{PhysicsBody, PhysicsBodyParams};
use crate::components::ModelSpec;
use crate::math::{Vec3f,UnitQuatf};
use crate::assets::Assets;
use crate::physics_world::PhysicsWorld;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct FloorGrid;

#[derive(Component)]
pub struct FloorTile;

impl FloorGrid {
    pub fn spawn(
        mut commands: Commands,
        mut physics: ResMut<PhysicsWorld>,
        assets: Res<Assets>,
    ) {
        let num_tiles_x = 19;
        let num_tiles_z = 19;
        for i in 0..num_tiles_x {
            for j in 0..num_tiles_z {
                let pos = Vec3f::new((i - num_tiles_x/2) as f32 * 2., 0., (j-num_tiles_z/2) as f32 * 2.);
                let rot = UnitQuatf::identity();
                let scale = Vec3f::new(1.0, 1.0, 1.0);
                let transform = Transform::new(pos, rot, scale);
                let modelspec = ModelSpec::new(String::from("Sci-Fi-Floor.obj"));
                commands.spawn((FloorTile, modelspec, transform));
            }
        }

        let pos = Vec3f::new(0. as f32, 0., 0.);
        let rot = UnitQuatf::identity();
        let scale = Vec3f::new(num_tiles_x as f32, 1.0, num_tiles_z as f32);
        let transform = Transform::new(pos, rot, scale);

        let collision_model =  assets.collision_model_store.get(
                        &String::from("Sci-Fi-Floor.obj")).unwrap();
        
        let physics_body = PhysicsBody::new(
            PhysicsBodyParams {
                pos,
                scale,
                rotation_axis: Vec3f::from_element(0.0),
                rotation_angle: 0.0,
                movable: false,
                collision_model: Some(&collision_model),
                collision_ball: None,
                gravity_scale: None,
                lin_vel: None,
                ang_vel: None,
            },
            &mut physics,
        );
        

        commands.spawn((FloorGrid, physics_body, transform));
    }   
}
