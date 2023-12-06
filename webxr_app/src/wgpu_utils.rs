use wgpu::util::DeviceExt;

pub fn new_uniform_bind_group(
    device: &wgpu::Device,
    data: &[u8],
    visibility: wgpu::ShaderStages,
    label: &'static str,
) -> (wgpu::Buffer, wgpu::BindGroupLayout, wgpu::BindGroup) {
    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: data,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
        label: Some(label),
    });

    let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
        label: Some(label),
    });

    (buffer, group_layout, group)
}
