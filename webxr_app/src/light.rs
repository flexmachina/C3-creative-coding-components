use crate::wgpu_utils;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    pub position: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    _padding: u32,
    pub color: [f32; 3],
    _padding2: u32,
}

pub struct Light {
    pub uniform: LightUniform,
    pub buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl Light {
    pub fn new(device: &wgpu::Device, position: [f32; 3], color: [f32; 3]) -> Self {
        let uniform = LightUniform {
            position,
            _padding: 0,
            color,
            _padding2: 0,
        };
        
        let (buffer, bind_group_layout, bind_group) =
            wgpu_utils::new_uniform_bind_group(
                &device,
                bytemuck::cast_slice(&[uniform]),
                wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                "Light");
        Self {
            uniform,
            buffer,
            bind_group_layout,
            bind_group
        }
    }
}