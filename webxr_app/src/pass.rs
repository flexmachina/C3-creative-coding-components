use crate::camera::CameraUniform;
use crate::light::LightUniform;
use crate::node::Node;

pub trait Pass {
    fn draw(
        &mut self,
        view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        nodes: &Vec<Node>,
        camera: &CameraUniform,
        light: &LightUniform
    );
}