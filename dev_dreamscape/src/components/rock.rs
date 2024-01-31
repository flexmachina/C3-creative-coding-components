use crate::components::transform::Transform;
use crate::components::{PhysicsBody, PhysicsBodyParams, Player};
use crate::components::ModelSpec;
use crate::input::Input;
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
        for x in 0..1000 {
        
            let rock_choice = true;//reseeding_rng.gen::<bool>();
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


            let pos_x = reseeding_rng.gen_range(-100.0..100.0);
            let pos_y = reseeding_rng.gen_range(1.0..10.0);
            let pos_z = reseeding_rng.gen_range(-100.0..100.0);
            println!("pos x, y {} {} {}",pos_x, pos_y, pos_z );
            let pos = Vec3f::new(pos_x, pos_y, pos_z);

            

            let scale_factor = reseeding_rng.gen_range(if rock_choice {0.1..0.5} else {0.001..0.006});
            let scale = Vec3f::new( scale_factor , scale_factor , scale_factor ); 
           

            let rotation_axis = Vec3f::y_axis().into_inner();
            let rotation_angle = 0.0;

            commands.spawn(Self::new_component(
                    model_label,
                    collision_model_label,
                    pos,
                    scale,
                    rotation_axis,
                    rotation_angle,
                    &assets,
                    &mut physics)
            );

        } 

    }


    fn new_component(
        model_label: String,
        collision_model_label: String,
        pos: Vec3f,
        scale: Vec3f,
        rotation_axis: Vec3f,
        rotation_angle: f32,
        assets: &Assets,
        physics: &mut PhysicsWorld,
    ) -> (Rock, PhysicsBody, Transform, ModelSpec) {

        
        let rot = UnitQuatf::from_axis_angle(&UnitVec3f::new_normalize(rotation_axis), 
                                             rotation_angle);

        let transform = Transform::new(pos, rot, scale);
        
        //println!("looking for collision_model {}", &collision_model_label);

        let collision_model =  assets.collision_model_store.get(
                                    &collision_model_label).unwrap();

        let physics_body = PhysicsBody::new(
            PhysicsBodyParams {
                pos,
                scale,
                rotation_axis,
                rotation_angle,
                movable: true,
                collision_model: None,//Some(&collision_model),
                collision_ball: Some(scale.norm()) , //None,
                gravity_scale: Some(0.005)
            },
            physics,
        );

        let modelspec = ModelSpec::new(model_label);
        (Rock, physics_body, transform, modelspec)


    }
}