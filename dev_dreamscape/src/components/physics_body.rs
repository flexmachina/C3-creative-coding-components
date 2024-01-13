use crate::components::{Player, Transform};
use crate::math::{to_point, Vec3f};
use crate::physics_world::PhysicsWorld;
use bevy_ecs::prelude::*;
use rapier3d::prelude::*;
//use crate::components::grab::Grab;

#[derive(Component)]
pub struct PhysicsBody {
    handle: RigidBodyHandle,
    movable: bool
}

pub struct PhysicsBodyParams {
    pub pos: Vec3f,
    pub scale: Vec3f,
    pub rotation_angle: f32,
    pub rotation_axis: Vec3f,
    pub movable: bool,
}

impl PhysicsBody {
    pub fn new(params: PhysicsBodyParams, physics: &mut PhysicsWorld) -> Self {
        let PhysicsBodyParams {
            pos,
            scale,
            rotation_axis,
            rotation_angle,
            movable,
        } = params;


        let body = RigidBodyBuilder::new(orig_type(movable))
            .translation(vector![pos.x, pos.y, pos.z])
            .rotation(rotation_axis * rotation_angle)
            .build();

        // TODO Other shapes
        let collider = ColliderBuilder::cuboid(scale.x, scale.y, scale.z)
            .restitution(0.2)
            .friction(0.7)
            .build();
        let (handle, _) = physics.add_body(body, collider);

        Self {
            handle,
            movable
        }
    }

    pub fn sync(mut q: Query<(&mut Transform, &PhysicsBody)>, physics: Res<PhysicsWorld>) {
        for (mut transform, body) in q.iter_mut() {
            let body = physics.bodies.get(body.handle).unwrap();
            let phys_pos = body.translation();
            let phys_rot = body.rotation().inverse(); // Not sure why inverse is needed
            transform.set(*phys_pos, *phys_rot.quaternion());
        }
    }

    /*
    pub fn grab_start_stop(
        mut physics: ResMut<PhysicsWorld>,
        mut ungrabbed: RemovedComponents<Grab>,
        bodies: Query<&PhysicsBody>,
        new_grabbed: Query<&PhysicsBody, Added<Grab>>,
    ) {
        // Tweak newly grabbed
        if let Ok(g) = new_grabbed.get_single() {
            let body = physics.bodies.get_mut(g.handle).unwrap();
            body.set_body_type(RigidBodyType::KinematicPositionBased, true);
        }

        // Tweak no longer grabbed
        if let Some(e) = ungrabbed.iter().next() {
            let phys_body = bodies.get(e).unwrap();
            let body = physics.bodies.get_mut(phys_body.handle).unwrap();
            body.set_body_type(orig_type(phys_body.movable), true);
        }
    }

    pub fn update_grabbed(
        player: Query<&Transform, With<Player>>,
        grabbed: Query<(&mut PhysicsBody, &Grab)>,
        mut physics: ResMut<PhysicsWorld>
    ) {
        let player_transform = player.single();
        if let Ok(grabbed) = grabbed.get_single() {
            let body = physics.bodies.get_mut(grabbed.0.handle).unwrap();
            let new_pos = player_transform.matrix().transform_point(&to_point(grabbed.1.body_local_pos));
            body.set_translation(new_pos.coords, true);
        }
    }
    */

    pub fn body_handle(&self) -> RigidBodyHandle {
        self.handle
    }
}

fn orig_type(movable: bool) -> RigidBodyType {
    if movable { RigidBodyType::Dynamic } else { RigidBodyType::Fixed }
}
