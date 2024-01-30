use anyhow::*;
use bevy_ecs::prelude::Resource;
use cfg_if::cfg_if;

use wgpu::util::DeviceExt;
use std::io::{BufReader, Cursor};
use std::collections::HashMap;

use crate::device::Device;
use crate::texture::Texture;
use crate::{model, texture};
use crate::math::{Vec2, Vec3, Vec3f, to_point};
use rapier3d::prelude::{Point,Real};

use crate::logging::printlog;
use crate::renderers::{PhongPass, SkyboxPass};

use std::path::Path;











#[cfg(target_arch = "wasm32")]
fn format_url(file_name: &str) -> reqwest::Url {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let origin = location.origin().unwrap();
    //if !origin.ends_with("learn-wgpu") {
    //    origin = format!("{}/learn-wgpu", origin);
    //}
    let base = reqwest::Url::parse(&format!("{}/res/", origin,)).unwrap();
    base.join(file_name).unwrap()
}

pub async fn load_string(file_name: &str) -> anyhow::Result<String> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url = format_url(file_name);
            let txt = reqwest::get(url)
                .await?
                .text()
                .await?;
        } else {
            let path = std::path::Path::new(env!("OUT_DIR"))
                .join("res")
                .join(file_name);
            let txt = std::fs::read_to_string(path)?;
        }
    }

    Ok(txt)
}

pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url = format_url(file_name);
            printlog(url.as_str());
            let data = reqwest::get(url)
                .await?
                .bytes()
                .await?
                .to_vec();
            printlog("got data")
        } else {
            let path = std::path::Path::new(env!("OUT_DIR"))
                .join("res")
                .join(file_name);
            let data = std::fs::read(path)?;
        }
    }

    Ok(data)
}



pub async fn load_texture(
    file_name: &str,
    is_normal_map: bool,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> anyhow::Result<texture::Texture> {
    let data = load_binary(file_name).await?;
    texture::Texture::from_bytes(device, queue, &data, file_name, is_normal_map)
}

pub async fn load_model(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> anyhow::Result<model::Model> {
    
    let obj_text = load_string(file_name).await?;
    let obj_cursor = Cursor::new(obj_text);
    let mut obj_reader = BufReader::new(obj_cursor);

    let file_folder = Path::new(&file_name).parent().unwrap();

    let (models, obj_materials) = tobj::load_obj_buf_async(
        &mut obj_reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |p| async move {
            println!("mtl file path is {}",&file_folder.join(&p).to_str().unwrap());
            let mat_text = load_string(&file_folder.join(&p).to_str().unwrap()).await.unwrap();
            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
        },
    )
    .await?;

    let mut materials = Vec::new();
    for m in obj_materials? {

        println!("mtl diffuse file path is {}",&file_folder.join(&m.diffuse_texture).to_str().unwrap());
        println!("mtl normal file path is {}",&file_folder.join(&m.normal_texture).to_str().unwrap());

        let diffuse_texture = load_texture(&file_folder.join(&m.diffuse_texture).to_str().unwrap(), false, device, queue).await?;
        let normal_texture = load_texture(&file_folder.join(&m.normal_texture).to_str().unwrap(), true, device, queue).await?;

        materials.push(model::Material::new(
            &m.name,
            diffuse_texture,
            normal_texture
        ));
    }

    let meshes = models
        .into_iter()
        .map(|m| {
            let mut vertices = (0..m.mesh.positions.len() / 3)
                .map(|i| model::ModelVertex {
                    position: [
                        m.mesh.positions[i * 3],
                        m.mesh.positions[i * 3 + 1],
                        m.mesh.positions[i * 3 + 2],
                    ],
                    tex_coords: [m.mesh.texcoords[i * 2], m.mesh.texcoords[i * 2 + 1]],
                    normal: [
                        m.mesh.normals[i * 3],
                        m.mesh.normals[i * 3 + 1],
                        m.mesh.normals[i * 3 + 2],
                    ],
                    // We'll calculate these later
                    tangent: [0.0; 3],
                    bitangent: [0.0; 3],
                })
                .collect::<Vec<_>>();

            let indices = &m.mesh.indices;
            let mut triangles_included = vec![0; vertices.len()];

            // Calculate tangents and bitangets. We're going to
            // use the triangles, so we need to loop through the
            // indices in chunks of 3
            for c in indices.chunks(3) {
                let v0 = vertices[c[0] as usize];
                let v1 = vertices[c[1] as usize];
                let v2 = vertices[c[2] as usize];

                let pos0: Vec3<_> = v0.position.into();
                let pos1: Vec3<_> = v1.position.into();
                let pos2: Vec3<_> = v2.position.into();

                let uv0: Vec2<_> = v0.tex_coords.into();
                let uv1: Vec2<_> = v1.tex_coords.into();
                let uv2: Vec2<_> = v2.tex_coords.into();

                // Calculate the edges of the triangle
                let delta_pos1 = pos1 - pos0;
                let delta_pos2 = pos2 - pos0;

                // This will give us a direction to calculate the
                // tangent and bitangent
                let delta_uv1 = uv1 - uv0;
                let delta_uv2 = uv2 - uv0;

                // Solving the following system of equations will
                // give us the tangent and bitangent.
                //     delta_pos1 = delta_uv1.x * T + delta_u.y * B
                //     delta_pos2 = delta_uv2.x * T + delta_uv2.y * B
                // Luckily, the place I found this equation provided
                // the solution!
                let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
                let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
                // We flip the bitangent to enable right-handed normal
                // maps with wgpu texture coordinate system
                let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * -r;

                // We'll use the same tangent/bitangent for each vertex in the triangle
                vertices[c[0] as usize].tangent =
                    (tangent + Vec3::from(vertices[c[0] as usize].tangent)).into();
                vertices[c[1] as usize].tangent =
                    (tangent + Vec3::from(vertices[c[1] as usize].tangent)).into();
                vertices[c[2] as usize].tangent =
                    (tangent + Vec3::from(vertices[c[2] as usize].tangent)).into();
                vertices[c[0] as usize].bitangent =
                    (bitangent + Vec3::from(vertices[c[0] as usize].bitangent)).into();
                vertices[c[1] as usize].bitangent =
                    (bitangent + Vec3::from(vertices[c[1] as usize].bitangent)).into();
                vertices[c[2] as usize].bitangent =
                    (bitangent + Vec3::from(vertices[c[2] as usize].bitangent)).into();

                // Used to average the tangents/bitangents
                triangles_included[c[0] as usize] += 1;
                triangles_included[c[1] as usize] += 1;
                triangles_included[c[2] as usize] += 1;
            }

            // Average the tangents/bitangents
            for (i, n) in triangles_included.into_iter().enumerate() {
                let denom = 1.0 / n as f32;
                let v = &mut vertices[i];
                v.tangent = (Vec3::from(v.tangent) * denom).into();
                v.bitangent = (Vec3::from(v.bitangent) * denom).into();
            }

            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Vertex Buffer", file_name)),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Index Buffer", file_name)),
                contents: bytemuck::cast_slice(&m.mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            model::Mesh {
                name: file_name.to_string(),
                vertex_buffer,
                index_buffer,
                num_elements: m.mesh.indices.len() as u32,
                material: m.mesh.material_id.unwrap_or(0),
            }
        })
        .collect::<Vec<_>>();

    if materials.len() == 0 {
        materials.push(model::Material::new(
            "Default",
            Texture::default(device, queue),
            Texture::default(device, queue)));
    }
    Ok(model::Model { meshes, materials })
}



#[derive(Clone, Debug)]
pub struct CollisionMesh {
    pub vertices: Vec<Vec3f>,
    pub triangle_indices: Vec<[u32; 3]> 
}

#[derive(Clone, Debug)]
pub struct CollisionModel {
    pub collision_meshes: Vec<CollisionMesh>,
}

impl CollisionModel {

    pub fn get_all_vertices(&self) -> Vec<Vec3f> {
        self.collision_meshes.clone().into_iter().map(|m| {
            m.vertices            
        }).collect::<Vec<_>>().into_iter().flatten().collect::<Vec<Vec3f>>()
    }

    pub fn get_all_vertices_points(&self) -> Vec<Point<Real>> {
        self.get_all_vertices().into_iter().map( |v| {
            to_point(v)
        }).collect::<Vec<Point<Real>>>()
    }

    pub fn get_all_triangle_indices(&self) -> Vec<[u32; 3]> {
        self.collision_meshes.clone().into_iter().map(|m| {
            m.triangle_indices
        }).collect::<Vec<_>>().into_iter().flatten().collect::<Vec<[u32; 3]>>()
    }
}




pub async fn load_collision_model(file_name: &str) -> anyhow::Result<CollisionModel> {
    let obj_text = load_string(file_name).await?;
    let obj_cursor = Cursor::new(obj_text);
    let mut obj_reader = BufReader::new(obj_cursor);

    let file_folder = Path::new(&file_name).parent().unwrap();

    let (models, obj_materials) = tobj::load_obj_buf_async(
        &mut obj_reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |p| async move {
            let mat_text = load_string(&file_folder.join(&p).to_str().unwrap()).await.unwrap();
            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
        },
    )
    .await?;

    let collision_meshes = models
        .into_iter()
        .map(|m| {
            let mut vertices = (0..m.mesh.positions.len() / 3)
                .map(|i| Vec3f::new(
                        m.mesh.positions[i * 3] as f32,
                        m.mesh.positions[i * 3 + 1] as f32,
                        m.mesh.positions[i * 3 + 2] as f32
                    )
                ).collect::<Vec<_>>();

            let triangle_indices = m.mesh.indices.chunks(3).collect::<Vec<_>>().iter().map(
                    |i| [i[0] as u32, i[1] as u32, i[2] as u32]
                ).collect::<Vec<[u32;3]>>();

            CollisionMesh {
                vertices,
                triangle_indices,
            }
        }).collect::<Vec<_>>();

    Ok(CollisionModel { collision_meshes })
}


























// TODO Load also shaders, meshes, etc.
#[derive(Resource)]
pub struct Assets {
    pub skybox_tex: texture::Texture,
    pub model_store: HashMap<String,model::Model>,
    pub collision_model_store: HashMap<String,CollisionModel>,
}

impl Assets {

    pub async fn load_and_return(device: &Device) -> Self {
        printlog("In assets.load_and_return");

        let model_paths = vec![
            "cube.obj",
            "sphere.obj",
            "moon_surface/moon_surface.obj",
            "Rock1/RedishRock.obj",
            "Rock2/Rock2.obj",
            "Rock1/RedishRock-collider.obj",
            "Rock2/Rock2-collider.obj",
        ];

        let collision_model_paths = vec![
            "moon_surface/moon_surface-collider.obj",
            "Rock1/RedishRock-collider.obj",
            "Rock2/Rock2-collider.obj",
        ];


        let mut model_store = HashMap::new();
        for model_path in model_paths {
            let model = load_model(model_path, &device, &device.queue()).await.unwrap();
            model_store.insert(model_path.to_string(), model);
        }

        let mut collision_model_store = HashMap::new();
        for collision_model_path in collision_model_paths {
            let model = load_collision_model(collision_model_path).await.unwrap();
            collision_model_store.insert(collision_model_path.to_string(), model);
        }

        let skybox_tex = texture::Texture::load_cubemap_from_pngs(
                "skyboxes/planet_atmosphere", &device, &device.queue()).await;
        Self {
            skybox_tex,
            model_store,
            collision_model_store
        }
    }


}



// TODO Load also shaders, meshes, etc.
#[derive(Resource)]
pub struct Renderers {
    pub skybox_renderer: Option<SkyboxPass>,
    pub phong_renderer: Option<PhongPass>,
}

impl Renderers {
    pub fn init() -> Self {
        Self {
            skybox_renderer: None, 
            phong_renderer: None
        }
    }
}
