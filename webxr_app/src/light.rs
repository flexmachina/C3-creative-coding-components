use cgmath::Vector3;

// Uniform for light data (position + color)
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    position: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    _padding: u32,
    color: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    _padding2: u32,
}

pub struct Light {
    pub position: Vector3<f32>,
    pub color: Vector3<f32>,
}

impl Light {
    
    pub fn to_uniform(&self) -> LightUniform {
        LightUniform {
            position: self.position.into(),
            _padding: 0,
            color: self.color.into(),
            _padding2: 0,
        }
    }
}