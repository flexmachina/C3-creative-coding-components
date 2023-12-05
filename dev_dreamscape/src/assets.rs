use anyhow::*;
use std::path::PathBuf;
use bevy_ecs::prelude::{Commands, Res, Resource};
use cfg_if::cfg_if;

use crate::device::Device;
use crate::texture::Texture;


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
            let data = reqwest::get(url)
                .await?
                .bytes()
                .await?
                .to_vec();
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
}

impl Assets {
    pub fn load(device: Res<Device>, mut commands: Commands) {
        let (skybox_tex, stone_tex) = pollster::block_on(async {
            let skybox_tex = Texture::new_cube_from_file("skybox_bgra.dds", &device)
                .await
                .unwrap();
            let stone_tex = Texture::new_2d_from_file("stonewall.jpg", &device)
                .await
                .unwrap();
            (skybox_tex, stone_tex)
        });

        commands.insert_resource(Self {
            skybox_tex,
            stone_tex
        })
    }
}
