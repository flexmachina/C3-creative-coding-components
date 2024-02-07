use crate::components::transform::Transform;
use crate::components::{PhysicsBody, PhysicsBodyParams};
use crate::components::ModelSpec;
use crate::math::{Vec3f,UnitQuatf};
use crate::assets::Assets;
use crate::physics_world::PhysicsWorld;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct FloorBox;

impl FloorBox {
    pub fn spawn(
        mut commands: Commands,
        mut physics: ResMut<PhysicsWorld>,
        assets: Res<Assets>,
    ) {
        //let modelspec = ModelSpec::new(String::from("moon_surface/moon_surface.obj"));
        let modelspec = ModelSpec::new(String::from("mars_surface/Crater.obj"));

        let pos = Vec3f::new(0.0, -9., 0.0);
        let rot = UnitQuatf::identity();
        //let scale = Vec3f::new(100.0, 0.5, 100.0);
        let scale = Vec3f::new(0.5, 0.5, 0.5);
        let transform = Transform::new(pos, rot, scale);

        let collision_model =  assets.collision_model_store.get(
                        //&String::from("moon_surface/moon_surface-collider.obj")).unwrap();
                        &String::from("mars_surface/Crater_low-collision.obj")).unwrap();
        
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
        

        commands.spawn((FloorBox, modelspec, physics_body,
                        transform));
    }   
}
