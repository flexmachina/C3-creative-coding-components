[Screencast from 2024-02-12 00-07-45.webm](https://github.com/user-attachments/assets/d3a8e960-45e9-4eeb-8e6c-085fe92da927)# C3-creative-coding-components
WASM based tools for creative coders to generate 3D & XR art experiences

# Demos

[Screencast from 2024-02-12 00-07-45.webm](https://github.com/user-attachments/assets/31d4a53a-fae1-496b-b498-349ca85a34a1)



https://github.com/user-attachments/assets/e16390a1-a451-4a26-833f-a97f0ea87686






# Usage
Instructions [here](https://github.com/flexmachina/C3-creative-coding-components/tree/main/dev_dreamscape)

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
    - WebXR (via WebXR API) 
        - Movement tracking via headset ✅
        - Hand tracking ✅
        - Movement via controller ⏹️
- Audio
    - Audio import ⏹️ 
    - Audio controller ⏹️

# Support & Discussion
- [Discord](https://discord.com/invite/u5jy7sMxB7)


# Credits 

## Code & Tutorials
- [learn-wgpu](https://sotrh.github.io/learn-wgpu/)
- [wgpu-demo](https://github.com/0xc0dec/wgpu-demo)
- [From 0 to glTF with WebGPU](https://www.willusher.io/graphics/2023/04/10/0-to-gltf-triangle)
- [GLTF Animations in wgpu and Rust](https://whoisryosuke.com/blog/2022/importing-gltf-with-wgpu-and-rust)

## Other Learning resources
- *[WebGPU Fundamentals](https://webgpufundamentals.org/)*
- [Render Pipelines in wgpu and Rust](https://whoisryosuke.com/blog/2022/render-pipelines-in-wgpu-and-rust)

## Learning resources - Gaussian Splatting
- [https://arxiv.org/abs/2308.04079](Gaussian Splatting paper)
- [https://www.thomasantony.com/posts/gaussian-splatting-renderer/](Understanding 3D Gaussian Splats by writing a software renderer)


## Assets ([CC Licence](https://creativecommons.org/licenses/by/4.0/))
- [Crater City - Terrain](https://sketchfab.com/3d-models/crater-city-terrain-0bdacc08da824abda64701698dd5cdd1)
- [Alien planet surface landscape with craters](https://sketchfab.com/3d-models/alien-planet-surface-landscape-with-craters-653797d4ae4749f4aa02c721d7d6596e)
- [A Rock](https://sketchfab.com/3d-models/a-rock-c49139dbab5e4c498c225b56cca30466)
- [Red Rock Photo-scan Gameready](https://sketchfab.com/3d-models/red-rock-photo-scan-gameready-66b9ecc1a1a14a2e8e7234e9363b7360)


