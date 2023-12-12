// Uniform for light data (position + color)
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    pub position: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    pub _padding: u32,
    pub color: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    pub _padding2: u32,
}

pub struct Light {
    pub uniform: LightUniform,
}

impl Light {
    pub fn new(position: [f32; 3], color: [f32; 3]) -> Self {
        let uniform = LightUniform {
            position,
            _padding: 0,
            color,
            _padding2: 0,
        };
        Self {
            uniform
        }
    }
}