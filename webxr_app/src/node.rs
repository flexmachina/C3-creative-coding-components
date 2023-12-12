use crate::{instance::Instance, model::Model};

// This represents a 3D model in a scene.
// It contains the 3D model and instance data
pub struct Node {
    // The vertex buffers and texture data
    pub model: Model,
    // An array of positional data for each instance (can just pass 1 instance)
    pub instances: Vec<Instance>,
}