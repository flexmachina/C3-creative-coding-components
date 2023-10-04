#![cfg(web_sys_unstable_apis)]

use log::{debug,info};
use std::cell::RefCell;
use std::rc::Rc;

use js_sys::{Object, Reflect};
use wasm_bindgen::prelude::*;
use web_sys::*;


fn request_animation_frame(session: &XrSession, f: &Closure<dyn FnMut(f64, XrFrame)>) -> u32 {
    // This turns the Closure into a js_sys::Function
    // See https://rustwasm.github.io/wasm-bindgen/api/wasm_bindgen/closure/struct.Closure.html#casting-a-closure-to-a-js_sysfunction
    session.request_animation_frame(f.as_ref().unchecked_ref())
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
    gl: Rc<WebGl2RenderingContext>,
    state: Rc<RefCell<crate::State>>,
}

impl XrApp {
    pub fn new(state: crate::State) -> Self {

        let session = Rc::new(RefCell::new(None));
        let xr_mode = true;
        let gl = Rc::new(create_webgl_context(xr_mode).unwrap());

        let state = Rc::new(RefCell::new(state));
        Self { session, gl, state }
    }

    pub async fn init(&self) {
        info!("Starting WebXR...");
        let navigator: web_sys::Navigator = web_sys::window().unwrap().navigator();
        let xr = navigator.xr();
        // XrSessionMode::ImmersiveVr results in nothing being
        // rendered.
        // TODO: investigate 
        let session_mode = XrSessionMode::Inline;
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

        let xr_gl_layer = XrWebGlLayer::new_with_web_gl2_rendering_context(&xr_session, &self.gl).unwrap();
        let mut render_state_init = XrRenderStateInit::new();
        render_state_init.base_layer(Some(&xr_gl_layer));
        xr_session.update_render_state_with_state(&render_state_init);

        self.session.borrow_mut().replace(xr_session);

        self.start();
    }

    fn start(&self) {
        let f = Rc::new(RefCell::new(None));
        let g = f.clone();

        let state = self.state.clone();
        let gl = self.gl.clone();

        *g.borrow_mut() = Some(Closure::new(move |_time: f64, frame: XrFrame| {
            let sess: XrSession = frame.session();
            let mut state = state.borrow_mut();
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

            let texture = crate::utils::create_view_from_device_framebuffer(
                &state.render_state.device,
                framebuffer,
                &xr_gl_layer,
                state.render_state.color_format,
                "device framebuffer (colour)");

            state.render_to_texture(&texture);

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
