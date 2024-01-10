#![cfg(web_sys_unstable_apis)]

#[allow(unused_imports)]
use log::{debug,error,info};
use std::cell::RefCell;
use std::rc::Rc;

use js_sys::{Object, Reflect};
use wasm_bindgen::prelude::*;
use web_sys::*;

use crate::camera::XrCamera;
use crate::maths::{Mat4, Mat4f, Point3, Quat, UnitQuat};


fn request_animation_frame(session: &XrSession, f: &Closure<dyn FnMut(f64, XrFrame)>) -> u32 {
    // This turns the Closure into a js_sys::Function
    // See https://rustwasm.github.io/wasm-bindgen/api/wasm_bindgen/closure/struct.Closure.html#casting-a-closure-to-a-js_sysfunction
    session.request_animation_frame(f.as_ref().unchecked_ref())
}

// We need to take care here because:
// * WebGL matrices are stored as an array in column-major order
// * nalgebra::Matrix4::new args are in row-major order
// https://developer.mozilla.org/en-US/docs/Web/API/XRRigidTransform/matrix
fn to_mat(v: &Vec<f32>) -> Mat4f {
    Mat4::new(
        v[0],  v[4],  v[8],  v[12],
        v[1],  v[5],  v[9],  v[13],
        v[2],  v[6],  v[10], v[14],
        v[3],  v[7],  v[11], v[15],
    )
}

pub fn create_webgl_context(xr_mode: bool) -> Result<WebGl2RenderingContext, JsValue> {
    let canvas = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()
        .unwrap();

    let gl: WebGl2RenderingContext = if xr_mode {
        let gl_attribs = Object::new();
        Reflect::set(
            &gl_attribs,
            &JsValue::from_str("xrCompatible"),
            &JsValue::TRUE,
        )
        .unwrap();

        canvas
            .get_context_with_context_options("webgl2", &gl_attribs)?
            .unwrap()
            .dyn_into()?
    } else {
        canvas.get_context("webgl2")?.unwrap().dyn_into()?
    };

    Ok(gl)
}

pub struct XrApp {
    session: Rc<RefCell<Option<XrSession>>>,
    ref_space: Rc<RefCell<Option<XrReferenceSpace>>>,
    gl: Rc<WebGl2RenderingContext>,
    state: Rc<RefCell<crate::State>>,
}

impl XrApp {
    pub fn new(state: Rc<RefCell<crate::State>>) -> Self {

        let session = Rc::new(RefCell::new(None));
        let ref_space = Rc::new(RefCell::new(None));
        let xr_mode = true;
        let gl = Rc::new(create_webgl_context(xr_mode).unwrap());
        Self { session, ref_space, gl, state: state.clone() }
    }

    pub async fn init(&self) {
        info!("Starting WebXR...");
        let navigator: web_sys::Navigator = web_sys::window().unwrap().navigator();
        let xr = navigator.xr();
        // XrSessionMode::ImmersiveVr seems work now
        // TODO: make this configurable
        let session_mode = XrSessionMode::ImmersiveVr;
        let session_supported_promise = xr.is_session_supported(session_mode);

        let supports_session =
            wasm_bindgen_futures::JsFuture::from(session_supported_promise).await;
        let supports_session = supports_session.unwrap();
        if supports_session == false {
            info!("XR session not supported");
            return ();
        }

        let xr_session_promise = xr.request_session(session_mode);
        let xr_session = wasm_bindgen_futures::JsFuture::from(xr_session_promise).await;
        let xr_session: XrSession = xr_session.unwrap().into();

        // TODO need to use right ref_space depending on  session mode
        //let ref_space_type = if xr_session.is_immersive() {XrReferenceSpaceType::Local} else  {XrReferenceSpaceType::Viewer};        
        let ref_space_type = XrReferenceSpaceType::Local;
        
        // Since we're dealing with multiple sessions now we need to track

        let xr_gl_layer = XrWebGlLayer::new_with_web_gl2_rendering_context(&xr_session, &self.gl).unwrap();
        let mut render_state_init = XrRenderStateInit::new();
        render_state_init.base_layer(Some(&xr_gl_layer));
        xr_session.update_render_state_with_state(&render_state_init);

        let ref_space_promise = xr_session.request_reference_space(ref_space_type);
        let ref_space = wasm_bindgen_futures::JsFuture::from(ref_space_promise).await;
        let ref_space: XrReferenceSpace = ref_space.unwrap().into();
        self.session.borrow_mut().replace(xr_session);
        self.ref_space.borrow_mut().replace(ref_space);

        self.start();
    }

    fn start(&self) {
        let f = Rc::new(RefCell::new(None));
        let g = f.clone();

        let state = self.state.clone();
        let gl = self.gl.clone();
        let ref_space = self.ref_space.clone();
        let last_frame_time = Rc::new(RefCell::new(0.));

        *g.borrow_mut() = Some(Closure::new(move | time: f64, frame: XrFrame| {
            let sess: XrSession = frame.session();
            let mut state = state.borrow_mut();
            let ref_space = ref_space.borrow_mut();
            let ref_space = ref_space.as_ref().unwrap();

            let xr_gl_layer = sess.render_state().base_layer().unwrap();

            let framebuffer = {
                match xr_gl_layer.framebuffer() {
                    Some(lfb) => {
                        debug!("Found XRWebGLLayer framebuffer!");
                        lfb
                    }
                    None    => {
                        debug!("XRWebGLLayer is null, using default one");
                        gl.get_parameter(WebGl2RenderingContext::FRAMEBUFFER_BINDING).unwrap().into()  
                    }
                }
            };

            let color_texture = crate::utils::create_view_from_device_framebuffer(
                &state.render_state.device,
                framebuffer.clone(),
                &xr_gl_layer,
                state.render_state.color_format,
                "device framebuffer (colour)");

            let depth_texture = crate::utils::create_view_from_device_framebuffer(
                &state.render_state.device,
                framebuffer,
                &xr_gl_layer,
                crate::texture::Texture::DEPTH_FORMAT,
                "device framebuffer (depth)");
        
            let viewer_pose = frame.get_viewer_pose(&ref_space).unwrap();
            for (view_idx, view) in viewer_pose.views().iter().enumerate() {
                let view: XrView = view.into();
                let viewport = xr_gl_layer.get_viewport(&view).unwrap();
                //gl.viewport(viewport.x(), viewport.y(), viewport.width(), viewport.height());
                let vp = crate::Rect { 
                    x: viewport.x() as f32, 
                    y: viewport.y() as f32, 
                    w: viewport.width() as f32, 
                    h: viewport.height() as f32
                };

                // Get decomposed position and orientation as they are easier to operate on than the view matrix
                let pos = view.transform().position();
                let position = Point3::new(pos.x() as f32, pos.y() as f32, pos.z() as f32);
                let r = view.transform().orientation();
                let rotation = Quat::new(r.w() as f32, r.x() as f32, r.y() as f32, r.z() as f32);
                let rotation = UnitQuat::new_normalize(rotation);
                state.scene.camera.xr_camera = XrCamera{position, rotation, projection: to_mat(&view.projection_matrix())};

                let delta_time = std::time::Duration::from_millis((time - *last_frame_time.borrow()) as u64);
                last_frame_time.replace(time);
                state.update_scene(delta_time);

                // Each view is rendered to a different region of the same framebuffer,
                // so only clear the framebuffer once before the first render pass.
                let clear = view_idx == 0;
                state.render_to_texture(&color_texture, Some(&depth_texture), Some(vp), clear);
            }

            // Schedule ourself for another requestAnimationFrame callback.
            // TODO: WebXR Samples call this at top of request_animation_frame - should this be moved?
            request_animation_frame(&sess, f.borrow().as_ref().unwrap());
        }));

        let session: &Option<XrSession> = &self.session.borrow();
        let sess: &XrSession = if let Some(sess) = session {
            sess
        } else {
            return ();
        };
        request_animation_frame(sess, g.borrow().as_ref().unwrap());
    }
}
