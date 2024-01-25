# C3-creative-coding-components
WASM based tools for creative coders to generate 3D & XR art experiences

# Status
- Platforms
    - Desktop (using winit) ✅ 
    - Web (via WASM) ✅
    - WebXR (via WASM, tested on Quest 3) ✅
- Asset import
    - Wavefront .obj ✅
    - Skyboxes (cubemap pngs) ✅
    - Gaussian Splat ⏹️
    - glTF ⏹️
- Entity-Component-System (ECS) world modeling (via Bevy-ECS) ✅
- Physics (via Rapier3D) ✅
- Renderers
    - Skybox renderer ✅
    - Phong shader ✅
    - Gaussian Splat renderer ⏹️
- Controller input
    - Desktop (using winit) ✅
    - Web (using winit) ✅
    - WebXR (via WebXR API) ⏹️
- Audio
    - Audio import ⏹️ 
    - Audio controller ⏹️

# Credits 
- [learn-wgpu](https://sotrh.github.io/learn-wgpu/)
- [wgpu-demo](https://github.com/0xc0dec/wgpu-demo)



# Useful learning resources
- *[WebGPU Fundamentals](https://webgpufundamentals.org/)*
- [From 0 to glTF with WebGPU](https://www.willusher.io/graphics/2023/04/10/0-to-gltf-triangle)
- [Render Pipelines in wgpu and Rust](https://whoisryosuke.com/blog/2022/render-pipelines-in-wgpu-and-rust)

# Useful learning resources - Gaussian Splatting
- [https://www.thomasantony.com/posts/gaussian-splatting-renderer/](Understanding 3D Gaussian Splats by writing a software renderer)
