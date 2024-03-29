use std::f32::consts::PI;
use crate::components::camera::Camera;
use crate::components::Transform;
use crate::device::{Device, SurfaceSize};
use crate::events::{WindowResizeEvent, CameraSetEvent};
use crate::input::Input;
use crate::math::Vec3f;
use crate::physics_world::PhysicsWorld;
use bevy_ecs::prelude::*;
use rapier3d::prelude::*;
use crate::frame_time::FrameTime;

#[derive(Component)]
pub struct Player {
    /*
    target_pt: Option<Vec3f>,
    target_body: Option<RigidBodyHandle>,
    */
    collider_handle: ColliderHandle,
    h_rot_acc: f32,
    v_rot_acc: f32,
    translation_acc: Vec3f
}

impl Player {
    pub fn spawn(
        device: Res<Device>,
        mut physics: ResMut<PhysicsWorld>,
        mut commands: Commands,
    ) {
        let pos = Vec3f::new(0.0, 1.7, 0.0);

        let znear = 0.1;
        let zfar = 100.0;
        let fov = 45.0;

        let camera = Camera::new(
            device.surface_size().width as u32,
            device.surface_size().height as u32, 
            fov, znear, zfar
        );
        let mut transform = Transform::from_position(pos);
        transform.look_at(Vec3f::new(0., 0., -100.));

        let collider = ColliderBuilder::ball(0.5)
            .restitution(0.1)
            .friction(0.1)
            .mass(500.0)
            .translation(pos)
            .build();
        let collider_handle = physics.colliders.insert(collider);

        commands.spawn((
            Player {
                collider_handle,
                /*
                target_pt: None,
                target_body: None,
                */
                h_rot_acc: 0.0,
                v_rot_acc: 0.0,
                translation_acc: Vec3f::zeros()
            },
            camera,
            transform
        ));
    }

    /*
    pub fn target_pt(&self) -> Option<Vec3f> {
        self.target_pt
    }

    pub fn target_body(&self) -> Option<RigidBodyHandle> {
        self.target_body
    }
    */

    //Player isn't moving here, just looking around
    //(likely in WebXR, so only the view needs to change, no need to change physics)
    pub fn update_player_view_xr(mut player: Query<(&mut Self, &mut Camera, &mut Transform)>,
                                mut cameraset_events: EventReader<CameraSetEvent>) {

        if let Some(e) = cameraset_events.iter().last() {
            let (_, mut camera, mut transform) = player.single_mut();
            transform.set_pose(e.pos, e.rot);
            camera.set_projection_matrix(e.projection_matrix);
        }
    }

    pub fn update(
        frame_time: Res<FrameTime>,
        device: Res<Device>,
        input: Res<Input>,
        mut player: Query<(&mut Self, &mut Camera, &mut Transform)>,
        mut physics: ResMut<PhysicsWorld>,
        mut resize_events: EventReader<WindowResizeEvent>,
    ) {
        let (mut player, mut camera, mut transform) = player.single_mut();

        // Update camera aspect
        let last_resize = resize_events.iter().last();
        if let Some(last_resize) = last_resize {
            update_cam_aspect(&mut camera, last_resize.new_size, &device);
        }

        // Move and rotate
        let dt = frame_time.delta;
        if input.lmb_down {
            player.rotate(&mut transform, &input, dt);
            player.translate(&mut transform, dt, &input, &mut physics);
        }

        //update_target((&mut player, &transform), &physics);
    }

    fn translate(
        &mut self,
        transform: &mut Transform,
        dt: f32,
        input: &Input,
        physics: &mut PhysicsWorld,
    ) {
        let mut translation: Vec3f = Vec3f::from_element(0.0);
        if input.forward_down {
            translation += transform.forward();
        }
        if input.back_down {
            translation -= transform.forward();
        }
        if input.right_down {
            translation += transform.right();
        }
        if input.left_down {
            translation -= transform.right();
        }
        if input.up_down {
            translation += transform.up();
        }
        if input.down_down {
            translation -= transform.up();
        }

        const SPEED: f32 = 10.0;

        // Apply only if there's anything to apply. Otherwise getting NaN after normalize() :|
        if translation.magnitude() > 0.01 {
            self.translation_acc += translation.normalize() * dt * SPEED;
        }

        let (possible_translation, collider_current_pos) = physics
            .move_character(dt, self.translation_acc, self.collider_handle);
        self.translation_acc = possible_translation;

        let translation = SPEED * dt * self.translation_acc;
        self.translation_acc -= translation;

        transform.translate(translation);
        physics
            .colliders
            .get_mut(self.collider_handle)
            .unwrap()
            .set_translation(collider_current_pos + translation);
    }

    fn rotate(&mut self, transform: &mut Transform, input: &Input, dt: f32) {
        const MIN_TOP_ANGLE: f32 = 0.1;
        const MIN_BOTTOM_ANGLE: f32 = PI - 0.1;
        const SPEED: f32 = 25.0;

        let angle_to_top = transform.forward().angle(&Vec3f::y_axis());
        self.v_rot_acc += input.mouse_delta.1 * dt;
        // Protect from overturning - prevent camera from reaching the vertical line with small
        // margin angles.
        if angle_to_top + self.v_rot_acc <= MIN_TOP_ANGLE {
            self.v_rot_acc = -(angle_to_top - MIN_TOP_ANGLE);
        } else if angle_to_top + self.v_rot_acc >= MIN_BOTTOM_ANGLE {
            self.v_rot_acc = MIN_BOTTOM_ANGLE - angle_to_top;
        }

        // Smooth the movement a bit
        let v_rot = SPEED * dt * self.v_rot_acc;
        self.v_rot_acc -= v_rot;

        self.h_rot_acc += input.mouse_delta.0 * dt;
        let h_rot = SPEED * dt * self.h_rot_acc;
        self.h_rot_acc -= h_rot;

        // The game world uses a right-hand coordinate system (where a positive angle
        // is anti-clockwise), so we negate the angles here
        transform.rotate_axis(&Vec3f::y_axis(), -h_rot);
        transform.rotate_local_axis(&Vec3f::x_axis(), -v_rot);
    }
}

/*
fn update_target(player: (&mut Player, &Transform), physics: &PhysicsWorld) {
    if let Some((hit_pt, _, hit_collider)) = physics.cast_ray(
        player.1.position(),
        player.1.forward(),
        Some(player.0.collider_handle),
    ) {
        player.0.target_pt = Some(hit_pt);
        player.0.target_body = Some(
            physics.colliders.get(hit_collider)
                .unwrap()
                .parent()
                .unwrap()
        );
    } else {
        player.0.target_pt = None;
        player.0.target_body = None;
    }
}
*/

fn update_cam_aspect(camera: &mut Camera, new_surface_size: SurfaceSize, _device: &Device) {
    //camera.Projection.set_aspect(new_surface_size.width as f32 / new_surface_size.height as f32);
    camera.resize(new_surface_size.width as u32 , new_surface_size.height as u32);
    //TODO handle camera resize!
    /*
    if let Some(target) = camera.target_mut() {
        target.resize(
            (new_surface_size.width, new_surface_size.height),
            device,
        )
    }
    */
}
