#![cfg(web_sys_unstable_apis)]

#[allow(unused_imports)]
use log::{debug,error,info};
use std::cell::RefCell;
use std::rc::Rc;

use js_sys::{Object, Reflect};
use wasm_bindgen::prelude::*;
use web_sys::*;

use crate::logging::printlog;
use crate::math::{Mat4, Mat4f, Quat, Rect, Vec3f, UnitQuat};
use crate::xr::utils;


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

fn to_joint_transform_mats(arr: &[f32; 16 * JOINTS.len()]) -> Vec<Mat4f> {
    let mut mats: Vec<Mat4f> = vec![];
    for i in 0..JOINTS.len() {
        let mat = Mat4::from_column_slice(&arr[i*16..(i+1)*16]);
        mats.push(mat);
    }
    mats
}

fn js_array(values: &[&str]) -> JsValue {
    return JsValue::from(values.into_iter()
        .map(|x| JsValue::from_str(x))
        .collect::<js_sys::Array>());
}

fn create_webgl_context(xr_mode: bool) -> Result<WebGl2RenderingContext, JsValue> {
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

pub struct WebXRApp {
    session: Rc<RefCell<XrSession>>,
    ref_space: Rc<RefCell<XrReferenceSpace>>,
    gl: Rc<WebGl2RenderingContext>,
}

impl WebXRApp {
    pub async fn new() -> Self {
        printlog("Starting WebGL2 for WebXR");

        let gl = Rc::new(create_webgl_context(true).unwrap());

        printlog("Starting WebXR...");
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
            panic!("XR session not supported");
        }

        let mut session_init = XrSessionInit::new();
        session_init.optional_features(&js_array(&["hand-tracking"]));
        let xr_session_promise = xr.request_session_with_options(session_mode, &session_init);
        //let xr_session_promise = xr.request_session(session_mode);
        let xr_session = wasm_bindgen_futures::JsFuture::from(xr_session_promise).await;
        let xr_session: XrSession = xr_session.unwrap().into();

        // TODO need to use right ref_space depending on  session mode
        //let ref_space_type = if xr_session.is_immersive() {XrReferenceSpaceType::Local} else  {XrReferenceSpaceType::Viewer};        
        let ref_space_type = XrReferenceSpaceType::Local;
        
        // Since we're dealing with multiple sessions now we need to track

        let xr_gl_layer = XrWebGlLayer::new_with_web_gl2_rendering_context(&xr_session, &gl).unwrap();
        let mut render_state_init = XrRenderStateInit::new();
        render_state_init.base_layer(Some(&xr_gl_layer));
        xr_session.update_render_state_with_state(&render_state_init);

        let ref_space_promise = xr_session.request_reference_space(ref_space_type);
        let ref_space = wasm_bindgen_futures::JsFuture::from(ref_space_promise).await;
        let ref_space: XrReferenceSpace = ref_space.unwrap().into();

        let session = Rc::new(RefCell::new(xr_session));
        let ref_space = Rc::new(RefCell::new(ref_space));

        Self { session, ref_space, gl }
    }

    pub fn start(&self, app: Rc<RefCell<crate::app::App>>) {
        let f = Rc::new(RefCell::new(None));
        let g = f.clone();

        let app = app.clone();
        let gl = self.gl.clone();
        let ref_space = self.ref_space.clone();
        let last_frame_time = Rc::new(RefCell::new(0.));

        *g.borrow_mut() = Some(Closure::new(move | time: f64, frame: XrFrame| {
            let sess: XrSession = frame.session();
            let mut app = app.borrow_mut();
            let ref_space = &ref_space.borrow_mut();

            // Get hand poses and send to app
            for i in 0..sess.input_sources().length() {
                let input_source = sess.input_sources().get(i).unwrap();
                match input_source.hand() {
                    Some(hand) => {
                        let mut poses: [f32; 16 * JOINTS.len()] = [0.; 16 * JOINTS.len()];
                        let mut radii: [f32; JOINTS.len()] = [0.; JOINTS.len()];
                        let joint_arr = js_sys::Array::new_with_length(JOINTS.len() as u32);
                        for (j, joint) in JOINTS.iter().enumerate() {
                            let join_pose = hand.get(joint.clone());
                            joint_arr.set(j as u32, join_pose.into());
                        }        
                        if !frame.fill_poses(&joint_arr, ref_space, &mut poses) {
                            log::error!("Failed to fill hand join poses");
                        }
                        if !frame.fill_joint_radii(&joint_arr, &mut radii) {       
                            log::error!("Failed to fill hand joint radii");
                        }

                        let joint_transform_mats = to_joint_transform_mats(&poses);
                        match input_source.handedness() {
                            XrHandedness::Left => { app.update_hand(false, joint_transform_mats, radii.to_vec()) }
                            XrHandedness::Right => { app.update_hand(true, joint_transform_mats, radii.to_vec()) }
                            _ => {}
                        }
                    }
                    None => {} 
                };
            }

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

            let color_texture = utils::create_view_from_device_framebuffer(
                app.device(),
                framebuffer.clone(),
                &xr_gl_layer,
                app.color_format(),
                "device framebuffer (colour)");
            
            // Not used - see render_to_texture call below
            /*
            let depth_texture = utils::create_view_from_device_framebuffer(
                app.device(),
                framebuffer,
                &xr_gl_layer,
                crate::texture::Texture::DEPTH_FORMAT,
                "device framebuffer (depth)");
            */
        
            let delta_time = std::time::Duration::from_millis((time - *last_frame_time.borrow()) as u64);
            last_frame_time.replace(time);
            // Callback
            app.update_scene(delta_time);

            let viewer_pose = frame.get_viewer_pose(&ref_space).unwrap();
            for (view_idx, view) in viewer_pose.views().iter().enumerate() {
                let view: XrView = view.into();
                let viewport = xr_gl_layer.get_viewport(&view).unwrap();
                //gl.viewport(viewport.x(), viewport.y(), viewport.width(), viewport.height());
                let vp = Rect { 
                    x: viewport.x() as f32, 
                    y: viewport.y() as f32, 
                    w: viewport.width() as f32, 
                    h: viewport.height() as f32
                };

                // Get decomposed position and orientation as they are easier to operate on than the view matrix
                let pos = view.transform().position();
                let position = Vec3f::new(pos.x() as f32, pos.y() as f32, pos.z() as f32);
                let r = view.transform().orientation();
                let rotation = Quat::new(r.w() as f32, r.x() as f32, r.y() as f32, r.z() as f32);
                let rotation = UnitQuat::new_normalize(rotation);

                // Callback - Set camera to XrView pose 
                app.update_camera(position, rotation, to_mat(&view.projection_matrix()));

                // Each view is rendered to a different region of the same framebuffer,
                // so only clear the framebuffer once before the first render pass.
                let clear = view_idx == 0;
                // Callback
                // Not using the WebXR depth buffer as there's an error when using it together
                // with the offscreen colour buffer ("WebGL: INVALID_OPERATION: drawBuffers: BACK or NONE)
                app.render_to_texture(&color_texture, /*&depth_texture,*/ Some(vp), clear);
            }

            // Schedule ourself for another requestAnimationFrame callback.
            // TODO: WebXR Samples call this at top of request_animation_frame - should this be moved?
            request_animation_frame(&sess, f.borrow().as_ref().unwrap());
        }));

        let session = &self.session.borrow();
        request_animation_frame(session, g.borrow().as_ref().unwrap());
    }
}

// TODO: move to another file
const JOINTS: &'static [XrHandJoint] = &[
    XrHandJoint::Wrist,
    XrHandJoint::ThumbMetacarpal,
    XrHandJoint::ThumbPhalanxProximal,
    XrHandJoint::ThumbPhalanxDistal,
    XrHandJoint::ThumbTip,
    XrHandJoint::IndexFingerMetacarpal,
    XrHandJoint::IndexFingerPhalanxProximal,
    XrHandJoint::IndexFingerPhalanxIntermediate,
    XrHandJoint::IndexFingerPhalanxDistal,
    XrHandJoint::IndexFingerTip,
    XrHandJoint::MiddleFingerMetacarpal,
    XrHandJoint::MiddleFingerPhalanxProximal,
    XrHandJoint::MiddleFingerPhalanxIntermediate,
    XrHandJoint::MiddleFingerPhalanxDistal,
    XrHandJoint::MiddleFingerTip,
    XrHandJoint::RingFingerMetacarpal,
    XrHandJoint::RingFingerPhalanxProximal,
    XrHandJoint::RingFingerPhalanxIntermediate,
    XrHandJoint::RingFingerPhalanxDistal,
    XrHandJoint::RingFingerTip,
    XrHandJoint::PinkyFingerMetacarpal,
    XrHandJoint::PinkyFingerPhalanxProximal,
    XrHandJoint::PinkyFingerPhalanxIntermediate,
    XrHandJoint::PinkyFingerPhalanxDistal,
    XrHandJoint::PinkyFingerTip,
];