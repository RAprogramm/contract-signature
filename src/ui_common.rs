// src/ui_common.rs

use wasm_bindgen::JsCast;
use web_sys::{Document, Element, HtmlCanvasElement, HtmlElement, HtmlImageElement, Window};

use crate::{error::SigError, signature_core::SignaturePad};

/// Append inline style pairs without clobbering existing rules.
/// Very simple: appends `key:value;` if key is not present.
fn merge_inline_style(el: &Element, pairs: &[(&str, &str)]) -> Result<(), SigError> {
    let current = el.get_attribute("style").unwrap_or_default();
    let mut style = current.trim().to_string();
    if !style.is_empty() && !style.ends_with(';') {
        style.push(';');
    }
    for (k, v) in pairs {
        let needle = format!("{k}:");
        if !style.contains(&needle) {
            style.push_str(k);
            style.push(':');
            style.push_str(v);
            style.push(';');
        }
    }
    el.set_attribute("style", &style)
        .map_err(|_| SigError::OpFailed("setAttribute(style)".into()))
}

/// Compute (left, top) inside a scrollable container using an anchor element.
/// Returns None if anchor/container not found or not HtmlElement.
fn compute_anchor_offset(
    document: &Document,
    container_id: &str,
    anchor_id: &str
) -> Option<(i32, i32)> {
    let cont_el = document.get_element_by_id(container_id)?;
    let anch_el = document.get_element_by_id(anchor_id)?;
    let cont = cont_el.dyn_into::<HtmlElement>().ok()?;
    let anch = anch_el.dyn_into::<HtmlElement>().ok()?;

    let cr = cont.get_bounding_client_rect();
    let ar = anch.get_bounding_client_rect();

    let left = (ar.left() - cr.left()) + cont.scroll_left() as f64;
    let top = (ar.top() - cr.top()) + cont.scroll_top() as f64;

    Some((left.round() as i32, top.round() as i32))
}

/// RAII handle that owns the SignaturePad and its JS listeners.
/// On drop, listeners are removed.
pub struct SignatureHandle {
    canvas:   HtmlCanvasElement,
    pad:      std::rc::Rc<std::cell::RefCell<SignaturePad>>,
    on_down:  Option<wasm_bindgen::closure::Closure<dyn FnMut(web_sys::PointerEvent)>>,
    on_move:  Option<wasm_bindgen::closure::Closure<dyn FnMut(web_sys::PointerEvent)>>,
    on_up:    Option<wasm_bindgen::closure::Closure<dyn FnMut(web_sys::PointerEvent)>>,
    on_leave: Option<wasm_bindgen::closure::Closure<dyn FnMut(web_sys::PointerEvent)>>
}

impl SignatureHandle {
    /// Initialize on a given canvas element.
    pub fn new(canvas: HtmlCanvasElement) -> Result<Self, SigError> {
        let pad_core = SignaturePad::new(canvas.clone())?;
        let pad = std::rc::Rc::new(std::cell::RefCell::new(pad_core));

        // pointerdown
        let canvas_for_listen = canvas.clone();
        let canvas_in_cb = canvas.clone();
        let pad_down = std::rc::Rc::clone(&pad);
        let on_down =
            wasm_bindgen::closure::Closure::wrap(Box::new(move |e: web_sys::PointerEvent| {
                let rect = canvas_in_cb.get_bounding_client_rect();
                let x = e.client_x() as f64 - rect.x();
                let y = e.client_y() as f64 - rect.y();
                pad_down.borrow_mut().pointer_down(x, y);
            }) as Box<dyn FnMut(_)>);
        canvas_for_listen
            .add_event_listener_with_callback("pointerdown", on_down.as_ref().unchecked_ref())
            .map_err(|_| SigError::OpFailed("addEventListener(pointerdown)".into()))?;

        // pointermove
        let canvas_for_listen = canvas.clone();
        let canvas_in_cb = canvas.clone();
        let pad_move = std::rc::Rc::clone(&pad);
        let on_move =
            wasm_bindgen::closure::Closure::wrap(Box::new(move |e: web_sys::PointerEvent| {
                let rect = canvas_in_cb.get_bounding_client_rect();
                let x = e.client_x() as f64 - rect.x();
                let y = e.client_y() as f64 - rect.y();
                pad_move.borrow_mut().pointer_move(x, y);
            }) as Box<dyn FnMut(_)>);
        canvas_for_listen
            .add_event_listener_with_callback("pointermove", on_move.as_ref().unchecked_ref())
            .map_err(|_| SigError::OpFailed("addEventListener(pointermove)".into()))?;

        // pointerup
        let canvas_for_listen = canvas.clone();
        let pad_up = std::rc::Rc::clone(&pad);
        let on_up =
            wasm_bindgen::closure::Closure::wrap(Box::new(move |_e: web_sys::PointerEvent| {
                pad_up.borrow_mut().pointer_up();
            }) as Box<dyn FnMut(_)>);
        canvas_for_listen
            .add_event_listener_with_callback("pointerup", on_up.as_ref().unchecked_ref())
            .map_err(|_| SigError::OpFailed("addEventListener(pointerup)".into()))?;

        // pointerleave
        let canvas_for_listen = canvas.clone();
        let pad_leave = std::rc::Rc::clone(&pad);
        let on_leave =
            wasm_bindgen::closure::Closure::wrap(Box::new(move |_e: web_sys::PointerEvent| {
                pad_leave.borrow_mut().pointer_up();
            }) as Box<dyn FnMut(_)>);
        canvas_for_listen
            .add_event_listener_with_callback("pointerleave", on_leave.as_ref().unchecked_ref())
            .map_err(|_| SigError::OpFailed("addEventListener(pointerleave)".into()))?;

        Ok(Self {
            canvas,
            pad,
            on_down: Some(on_down),
            on_move: Some(on_move),
            on_up: Some(on_up),
            on_leave: Some(on_leave)
        })
    }

    pub fn to_png_data_url(&self) -> Result<String, SigError> {
        self.pad.borrow().to_png_data_url()
    }

    pub fn clear(&self) {
        self.pad.borrow_mut().clear();
    }
}

impl Drop for SignatureHandle {
    fn drop(&mut self) {
        if let Some(cb) = self.on_down.take() {
            let _ = self
                .canvas
                .remove_event_listener_with_callback("pointerdown", cb.as_ref().unchecked_ref());
        }
        if let Some(cb) = self.on_move.take() {
            let _ = self
                .canvas
                .remove_event_listener_with_callback("pointermove", cb.as_ref().unchecked_ref());
        }
        if let Some(cb) = self.on_up.take() {
            let _ = self
                .canvas
                .remove_event_listener_with_callback("pointerup", cb.as_ref().unchecked_ref());
        }
        if let Some(cb) = self.on_leave.take() {
            let _ = self
                .canvas
                .remove_event_listener_with_callback("pointerleave", cb.as_ref().unchecked_ref());
        }
    }
}

/// DOM helpers. Stateless. All state lives in `SignatureHandle`.
pub struct DomBindings;

impl DomBindings {
    /// Initialize by canvas id and return a RAII handle.
    pub fn init_by_canvas_id(canvas_id: &str) -> Result<SignatureHandle, SigError> {
        let window: Window = web_sys::window().ok_or(SigError::DomUnavailable)?;
        let document: Document = window.document().ok_or(SigError::DomUnavailable)?;
        let el = document
            .get_element_by_id(canvas_id)
            .ok_or_else(|| SigError::ElementNotFound(canvas_id.to_string()))?;
        let canvas: HtmlCanvasElement = el
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| SigError::ElementNotFound(canvas_id.to_string()))?;

        SignatureHandle::new(canvas)
    }

    /// Public API used from `yew.rs`:
    /// Insert/update `<img id="signature-img">` at (x, y) inside
    /// `container_id`.
    pub fn place_signature_img(
        container_id: &str,
        data_url: &str,
        x: i32,
        y: i32,
        width: i32
    ) -> Result<(), SigError> {
        Self::place_signature_img_with_id(container_id, "signature-img", data_url, x, y, width)
    }

    /// Convenience: place by anchor; falls back to (x,y) if anchor not found.
    pub fn place_signature_img_by_anchor(
        container_id: &str,
        anchor_id: &str,
        data_url: &str,
        fallback_x: i32,
        fallback_y: i32,
        width: i32
    ) -> Result<(), SigError> {
        let window: Window = web_sys::window().ok_or(SigError::DomUnavailable)?;
        let document: Document = window.document().ok_or(SigError::DomUnavailable)?;
        let (x, y) = compute_anchor_offset(&document, container_id, anchor_id)
            .unwrap_or((fallback_x, fallback_y));
        Self::place_signature_img_with_id(container_id, "signature-img", data_url, x, y, width)
    }

    /// Internal helper that ensures container is positioning context and reuses
    /// <img id>.
    fn place_signature_img_with_id(
        container_id: &str,
        img_id: &str,
        data_url: &str,
        x: i32,
        y: i32,
        width: i32
    ) -> Result<(), SigError> {
        let document = web_sys::window()
            .ok_or(SigError::DomUnavailable)?
            .document()
            .ok_or(SigError::DomUnavailable)?;

        let target: Element = document
            .get_element_by_id(container_id)
            .ok_or_else(|| SigError::ElementNotFound(container_id.to_string()))?;

        // Ensure container establishes positioning context
        merge_inline_style(&target, &[("position", "relative")])?;

        // Find or create <img id=img_id>
        let img_el: HtmlImageElement = if let Some(existing) = document.get_element_by_id(img_id) {
            existing.dyn_into::<HtmlImageElement>().map_err(|_| {
                SigError::OpFailed("existing signature element is not <img>".into())
            })?
        } else {
            let img = document
                .create_element("img")
                .map_err(|_| SigError::OpFailed("createElement(img)".into()))?;
            img.set_id(img_id);
            let img: HtmlImageElement = img
                .dyn_into::<HtmlImageElement>()
                .map_err(|_| SigError::OpFailed("created element is not <img>".into()))?;
            target
                .append_child(&img)
                .map_err(|_| SigError::OpFailed("appendChild(img)".into()))?;
            img
        };

        img_el.set_src(data_url);
        // Absolute positioning and stacking
        let img_el_ref: &Element = img_el.as_ref();
        merge_inline_style(
            img_el_ref,
            &[
                ("position", "absolute"),
                ("left", &format!("{x}px")),
                ("top", &format!("{y}px")),
                ("width", &format!("{width}px")),
                ("height", "auto"),
                ("z-index", "1"),
                ("pointer-events", "none")
            ]
        )?;

        Ok(())
    }
}
