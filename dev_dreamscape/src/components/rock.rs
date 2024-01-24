use crate::components::transform::Transform;
use crate::components::{PhysicsBody, PhysicsBodyParams};
use crate::components::ModelSpec;
use crate::math::{Vec3f,UnitQuatf,UnitVec3f};
use crate::assets::Assets;
use crate::physics_world::PhysicsWorld;
use bevy_ecs::prelude::*;

use rand::rngs::OsRng;
use rand::rngs::adapter::ReseedingRng;
use rand::prelude::*;
use rand_chacha::ChaCha20Core; 


#[derive(Component)]
pub struct Rock;

impl Rock {


    pub fn spawn_rock_field(
        assets: Res<Assets>,
        mut commands: Commands,
        mut physics: ResMut<PhysicsWorld>,
    ) {

        let prng = ChaCha20Core::from_entropy();
        let mut reseeding_rng = ReseedingRng::new(prng, 0, OsRng);
        for _x in 0..1000 {
        
            let rock_choice = reseeding_rng.gen_range(0.0..1.0) > 0.25; 
            let model_label = if rock_choice { 
                        String::from("Rock1/RedishRock-collider.obj") 
                    } else {
                        String::from("Rock2/Rock2-collider.obj")
                    };
            let collision_model_label = if rock_choice {
                        String::from("Rock1/RedishRock-collider.obj") 
                    } else {
                        String::from("Rock2/Rock2-collider.obj")
                    };


            let pos_x = reseeding_rng.gen_range(-50.0..50.0);
            let pos_y = reseeding_rng.gen_range(5.0..20.0);
            let pos_z = reseeding_rng.gen_range(-50.0..50.0);
            //println!("pos x, y {} {} {}",pos_x, pos_y, pos_z );
            let pos = Vec3f::new(pos_x, pos_y, pos_z);

            

            let scale_factor = reseeding_rng.gen_range(if rock_choice {0.1..0.8} else {0.1..0.8});
            let scale = Vec3f::new( scale_factor , scale_factor , scale_factor ); 
           

            let rotation_axis = Vec3f::new(pos_x, pos_y, pos_z);
            let rotation_angle = reseeding_rng.gen_range(0.0..2.0*std::f32::consts::PI);


            let has_movement = reseeding_rng.gen_range(0.0..1.0) < 0.25;
            let lv = if has_movement {
                //linear velocity seems to look a bit weird, so I set it to zero
                let _lv_x = reseeding_rng.gen_range(-0.02..0.02);
                let _lv_y = reseeding_rng.gen_range(-0.02..0.02);
                let _lv_z = reseeding_rng.gen_range(-0.02..0.02);
                Some(Vec3f::new(0.0, 0.0, 0.0))
            } else {
                None
            };

            let av = if has_movement {
                let av_x = reseeding_rng.gen_range(-1.0..1.0);
                let av_y = reseeding_rng.gen_range(-1.0..1.0);
                let av_z = reseeding_rng.gen_range(-1.0..1.0);
                Some(Vec3f::new(av_x, av_y, av_z))
            } else {
                None
            };

            commands.spawn(Self::new_component(
                    model_label,
                    collision_model_label,
                    pos,
                    scale,
                    rotation_axis,
                    rotation_angle,
                    lv,
                    av,
                    &assets,
                    &mut physics)
            );

        } 

    }


    fn new_component(
        model_label: String,
        _collision_model_label: String,
        pos: Vec3f,
        scale: Vec3f,
        rotation_axis: Vec3f,
        rotation_angle: f32,
        lin_vel: Option<Vec3f>,
        ang_vel: Option<Vec3f>,
        _assets: &Assets,
        physics: &mut PhysicsWorld,
    ) -> (Rock, PhysicsBody, Transform, ModelSpec) {

        
        let rot = UnitQuatf::from_axis_angle(&UnitVec3f::new_normalize(rotation_axis), 
                                             rotation_angle);

        let transform = Transform::new(pos, rot, scale);
        
        //println!("looking for collision_model {}", &collision_model_label);

        //Collider circle works just fine
        //let collision_model =  assets.collision_model_store.get(
        //                            &collision_model_label).unwrap();

        let physics_body = PhysicsBody::new(
            PhysicsBodyParams {
                pos,
                scale,
                rotation_axis,
                rotation_angle,
                movable: true,
                collision_model: None,//Some(&collision_model),
                collision_ball: Some(scale.norm()) , //None,
                gravity_scale: Some(0.0),
                lin_vel,
                ang_vel
            },
            physics,
        );

        let modelspec = ModelSpec::new(model_label);
        (Rock, physics_body, transform, modelspec)


    }
}
