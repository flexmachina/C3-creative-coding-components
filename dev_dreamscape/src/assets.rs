use anyhow::*;
use std::path::PathBuf;
use bevy_ecs::prelude::{Commands, Res, Resource};
use cfg_if::cfg_if;

use crate::device::Device;
use crate::texture::Texture;
use crate::mesh::Mesh;
use std::collections::HashMap;

use crate::logging::{printlog};

#[cfg(target_arch = "wasm32")]
fn format_url(file_name: &str) -> reqwest::Url {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let mut origin = location.origin().unwrap();
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








// TODO Load also shaders, meshes, etc.
#[derive(Resource)]
pub struct Assets {
    pub skybox_tex: Texture,
    pub stone_tex: Texture,
    pub cube_mesh_string: String,
    pub texture_store: HashMap<String,Texture>,
    pub mesh_store: HashMap<String,String>,
    pub mesh_GPU_store: HashMap<String,Mesh>
}

impl Assets {

    pub async fn load_and_return(device: &Device) -> Self {
        printlog("In assets.load_and_return");
        /*
        let (skybox_tex, stone_tex) = pollster::block_on(async {
            printlog("In assets.load - pollster async function");
            let skybox_tex = Texture::new_cube_from_file("skybox_bgra.dds", device)
                .await
                .unwrap();
            printlog("In assets.load - loaded skybox");
            let stone_tex = Texture::new_2d_from_file("stonewall.jpg", device)
                .await
                .unwrap();
            (skybox_tex, stone_tex)
        });
        */
        let skybox_tex = Texture::new_cube_from_file("skybox_bgra.dds", device)
            .await
            .unwrap();
        printlog("In assets.load - loaded skybox");
        let stone_tex = Texture::new_2d_from_file("stonewall.jpg", device)
            .await
            .unwrap();

        let cube_mesh_string = load_string("cube.obj").await.unwrap();

        let skybox_tex2 = Texture::new_cube_from_file("skybox_bgra.dds", device)
            .await
            .unwrap();
        printlog("In assets.load - loaded skybox");
        let stone_tex2 = Texture::new_2d_from_file("stonewall.jpg", device)
            .await
            .unwrap();

        let cube_mesh_string2 = load_string("cube.obj").await.unwrap();




        let texture_store = HashMap::from_iter([
                                        ("skybox_bgra.dds".to_string(), skybox_tex2), 
                                        ("stonewall.jpg".to_string(), stone_tex2)
                                    ]);
        let mesh_store = HashMap::from_iter([
                                        ("cube.obj".to_string(), cube_mesh_string2)
                                    ]);




        let cube_mesh_string_for_mesh = load_string("cube.obj").await.unwrap();
        let cube_mesh = Mesh::from_string(cube_mesh_string_for_mesh, &device);
        let mesh_GPU_store = HashMap::from_iter([
                                        ("cube.obj".to_string(), cube_mesh),
                                        ("skybox".to_string(), Mesh::quad(&device))
                                    ]);


        //new_texture_bind_group(device, params.texture, wgpu::TextureViewDimension::D2);




        Self {
            skybox_tex,
            stone_tex,
            cube_mesh_string,
            texture_store,
            mesh_store,
            mesh_GPU_store
        }
    }


    pub fn load(device: Res<Device>, mut commands: Commands) {
        printlog("In assets.load");
        let (skybox_tex, stone_tex, cube_mesh_string) = pollster::block_on(async {
            printlog("In assets.load - pollster async function");
            let skybox_tex = Texture::new_cube_from_file("skybox_bgra.dds", &device)
                .await
                .unwrap();
            printlog("In assets.load - loaded skybox");
            let stone_tex = Texture::new_2d_from_file("stonewall.jpg", &device)
                .await
                .unwrap();
            let cube_mesh_string = load_string("cube.obj").await.unwrap();
            (skybox_tex, stone_tex, cube_mesh_string)
        });

        let texture_store:HashMap<String,Texture> = HashMap::from_iter([]);
        let mesh_store:HashMap<String,String> = HashMap::from_iter([]);
        let mesh_GPU_store:HashMap<String,Mesh> = HashMap::from_iter([]);

        commands.insert_resource(Self {
            skybox_tex,
            stone_tex,
            cube_mesh_string,
            texture_store,
            mesh_store,
            mesh_GPU_store
        })
    }
}
