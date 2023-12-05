use std::f32::consts::PI;
use crate::components::camera::Camera;
use crate::components::transform::TransformSpace;
use crate::components::Transform;
use crate::device::{Device, SurfaceSize};
use crate::events::WindowResizeEvent;
use crate::input::Input;
use crate::math::Vec3;
use crate::physics_world::PhysicsWorld;
use crate::render_tags::RenderTags;
use crate::render_target::RenderTarget;
use bevy_ecs::prelude::*;
use rapier3d::prelude::*;
use crate::frame_time::FrameTime;

#[derive(Component)]
pub struct Player {
    target_pt: Option<Vec3>,
    target_body: Option<RigidBodyHandle>,
    collider_handle: ColliderHandle,
    h_rot_acc: f32,
    v_rot_acc: f32,
    translation_acc: Vec3
}

impl Player {
    pub fn spawn(
        device: Res<Device>,
        mut physics: ResMut<PhysicsWorld>,
        mut commands: Commands,
    ) {
        let pos = Vec3::new(7.0, 7.0, 7.0);

        let rt = RenderTarget::new(&device, None);
        let camera = Camera::new(
            device.surface_size().width as f32 / device.surface_size().height as f32,
            RenderTags::SCENE,
            Some(rt),
        );
        let mut transform = Transform::from_pos(pos);
        transform.look_at(Vec3::from_element(0.0));

        let collider = ColliderBuilder::ball(0.5)
            .restitution(0.7)
            .translation(pos)
            .build();
        let collider_handle = physics.colliders.insert(collider);

        commands.spawn((
            Player {
                collider_handle,
                target_pt: None,
                target_body: None,
                h_rot_acc: 0.0,
                v_rot_acc: 0.0,
                translation_acc: Vec3::zeros()
            },
            camera,
            transform
        ));
    }

    pub fn target_pt(&self) -> Option<Vec3> {
        self.target_pt
    }

    pub fn target_body(&self) -> Option<RigidBodyHandle> {
        self.target_body
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
        if input.rmb_down {
            player.rotate(&mut transform, &input, dt);
            player.translate(&mut transform, dt, &input, &mut physics);
        }

        update_target((&mut player, &transform), &physics);
    }

    fn translate(
        &mut self,
        transform: &mut Transform,
        dt: f32,
        input: &Input,
        physics: &mut PhysicsWorld,
    ) {
        let mut translation: Vec3 = Vec3::from_element(0.0);
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

        let angle_to_top = transform.forward().angle(&Vec3::y_axis());
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

        transform.rotate_around_axis(Vec3::y_axis().xyz(), h_rot, TransformSpace::World);
        transform.rotate_around_axis(Vec3::x_axis().xyz(), v_rot, TransformSpace::Local);
    }
}

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

fn update_cam_aspect(camera: &mut Camera, new_surface_size: SurfaceSize, device: &Device) {
    camera.set_aspect(new_surface_size.width as f32 / new_surface_size.height as f32);
    if let Some(target) = camera.target_mut() {
        target.resize(
            (new_surface_size.width, new_surface_size.height),
            device,
        )
    }
}
