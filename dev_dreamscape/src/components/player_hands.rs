use bevy_ecs::prelude::*;
use bevy_hierarchy::BuildChildren;
use bevy_hierarchy::Children;
use rapier3d::na::Rotation3;

use crate::components::Transform;
use crate::components::ModelSpec;
use crate::events::HandUpdateEvent;
use crate::math::Mat3f;
use crate::math::UnitQuatf;
use crate::math::Vec3;
use crate::math::Vec3f;


// TODO: Use same constant that's in webxr.rs
const NUM_JOINTS: usize = 25;

#[derive(Debug,Component)]
pub struct Joint {
    joint_index: usize
}

#[derive(Debug, Component, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Hand {
    Left,
    Right
}

pub struct PlayerHands {
}

impl PlayerHands {
    pub fn spawn(mut commands: Commands) {
        for hand in [Hand::Left, Hand::Right]
        { 
            commands.spawn((
                hand,
            ))
            .with_children(|parent| {
                for joint_index in 0..NUM_JOINTS {
                    parent.spawn((
                        Joint{joint_index},
                        Transform::default(),
                        ModelSpec::new(String::from("cube.obj"))
                    ));
                }
            });
        }
    }

    pub fn update(
        q_parent: Query<(&Hand, &Children)>,
        mut q_child: Query<(&Joint, &mut Transform)>,
        mut events: EventReader<HandUpdateEvent>,
    ) {
        for event in events.iter() {
            assert!(event.joint_transforms.len() == NUM_JOINTS);
            assert!(event.joint_radii.len() == NUM_JOINTS);
            for (&hand, children) in q_parent.iter() {
                if (hand == Hand::Right && event.hand) || 
                    (hand == Hand::Left && !event.hand)  {
                    for &child in children.iter() {
                        if let Ok((joint, mut transform)) = q_child.get_mut(child) {
                            let mat = event.joint_transforms[joint.joint_index];
                            let scale  = event.joint_radii[joint.joint_index];
                            // Decompose the pose matrix - not ideal performance-wise since it'll be recomputed in 
                            // Transform::rebuild_matrix().
                            let position: Vec3f = mat.fixed_view::<3, 1>(0, 3).into();
                            let rot_mat: Mat3f = mat.fixed_view::<3, 3>(0, 0).into();
                            // Note: Don't call UnitQuatf::from_matrix(&rot_mat) here as it seems to hang the
                            // app when hands are out of view. It's an iterative algorithm that probably
                            // doesn't handle degenerate rotations matrices.
                            let rotation = UnitQuatf::from_rotation_matrix(&Rotation3::from_matrix_unchecked(rot_mat));
                            transform.set_pose_and_scale(position, rotation, Vec3::from_element(scale));
                        }
                    }
                }
            }
        }
    }
}
