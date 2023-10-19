use crate::assets::load_string;
use crate::device::Device;
use std::io::{BufReader, Cursor};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshVertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
    normal: [f32; 3],
}

impl MeshVertex {
    pub fn buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<MeshVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

struct MeshPart {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

impl MeshPart {
    fn from_buffers(device: &Device, vertices: &[MeshVertex], indices: &[u32]) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32
        }
    }

    // TODO Use different vertex description and remove unused attributes
    fn quad(device: &Device) -> MeshPart {
        let vertices = vec![
            // Bottom left
            MeshVertex {
                position: [-1.0, -1.0, 0.0],
                tex_coords: [0.0, 0.0],
                normal: [0.0, 0.0, 0.0], // unused
            },
            // Top left
            MeshVertex {
                position: [-1.0, 1.0, 0.0],
                tex_coords: [0.0, 1.0],
                normal: [0.0, 0.0, 0.0], // unused
            },
            // Top right
            MeshVertex {
                position: [1.0, 1.0, 0.0],
                tex_coords: [1.0, 1.0],
                normal: [0.0, 0.0, 0.0], // unused
            },
            // Bottom right
            MeshVertex {
                position: [1.0, -1.0, 0.0],
                tex_coords: [1.0, 0.0],
                normal: [0.0, 0.0, 0.0], // unused
            },
        ];

        let indices: Vec<u32> = vec![0, 1, 2, 0, 2, 3];

        Self::from_buffers(device, &vertices, &indices)
    }
}

pub struct Mesh {
    parts: Vec<MeshPart>,
}

impl Mesh {
    pub fn quad(device: &Device) -> Self {
        Self {
            parts: vec![MeshPart::quad(device)],
        }
    }

    pub async fn from_file(file_name: &str, device: &Device) -> Mesh {
        let text = load_string(file_name).await.unwrap();
        let cursor = Cursor::new(text);
        let mut reader = BufReader::new(cursor);

        let (meshes, _) = tobj::load_obj_buf_async(
            &mut reader,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
            |p| async move {
                let mat_text = load_string(&p).await.unwrap();
                tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
            },
        ).await.unwrap();

        let parts = meshes
            .into_iter()
            .map(|m| {
                let vertices = (0..m.mesh.positions.len() / 3)
                    .map(|i| MeshVertex {
                        position: [
                            m.mesh.positions[i * 3],
                            m.mesh.positions[i * 3 + 1],
                            m.mesh.positions[i * 3 + 2],
                        ],
                        tex_coords: [
                            m.mesh.texcoords[i * 2],
                            m.mesh.texcoords[i * 2 + 1]
                        ],
                        normal: [
                            m.mesh.normals[i * 3],
                            m.mesh.normals[i * 3 + 1],
                            m.mesh.normals[i * 3 + 2],
                        ],
                    })
                    .collect::<Vec<_>>();

                MeshPart::from_buffers(device, &vertices, &m.mesh.indices)
            })
            .collect::<Vec<_>>();

        Mesh { parts }
    }
}

pub trait DrawMesh<'a> {
    fn draw_mesh(&mut self, mesh: &'a Mesh);
}

impl<'a> DrawMesh<'a> for wgpu::RenderBundleEncoder<'a> {
    fn draw_mesh(&mut self, mesh: &'a Mesh) {
        for part in &mesh.parts {
            self.set_vertex_buffer(0, part.vertex_buffer.slice(..));
            self.set_index_buffer(part.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            self.draw_indexed(0..part.num_indices, 0, 0..1);
        }
    }
}
