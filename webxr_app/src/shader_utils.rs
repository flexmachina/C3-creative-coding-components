#[allow(unused_imports)]
use log::{debug, error, info};
use naga_oil::compose::{
    ComposableModuleDescriptor, Composer, NagaModuleDescriptor, ShaderDefValue
};

macro_rules! load_shader {
    ($composer:expr, $path:literal, $webxr:expr) => {{
        shader_utils::make_module(
            $composer,
            $path,
            include_str!($path), 
            $webxr)
    }};
}

pub(crate) use load_shader;

pub fn make_module(composer: &mut Composer, shader_path: &str, shader_source: &str, webxr: bool) -> naga::Module {
    
    let mut shader_defs = std::collections::HashMap::new();
    if webxr {
        shader_defs.insert("WEBXR".to_string(),  ShaderDefValue::Bool(true));
    }
    composer
    .make_naga_module(NagaModuleDescriptor {
        source: shader_source,
        file_path: shader_path,
        shader_defs,
        ..Default::default()
    })
    .unwrap()
}

pub fn init_composer() -> Composer {
    let mut composer = Composer::default();

    let mut load_composable = |source: &str, file_path: &str| {
        match composer.add_composable_module(ComposableModuleDescriptor {
            source,
            file_path,
            ..Default::default()
        }) {
            Ok(_module) => {
                //info!("{} -> {:#?}", module.name, module)
            }
            Err(e) => {
                error!("? -> {e:#?}")
            }
        }
    };

    // Init modules for shared utils
    load_composable(
        include_str!("utils.wgsl"),
        "utils.wgsl",
    );
    composer
}
