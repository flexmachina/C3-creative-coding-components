use crate::camera::Camera;
use crate::transform::Transform;
use crate::Rect;
use crate::light::Light;
use crate::node::Node;


pub trait Pass {
    fn draw (
        &mut self,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        nodes: &Vec<Node>,
        camera: (&Camera, &Transform),
        light: &Light,
        viewport: &Option<Rect>,
        clear_color: bool,
        clear_depth: bool,
    ) -> wgpu::CommandBuffer;
}