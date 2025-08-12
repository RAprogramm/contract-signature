#[cfg(feature = "leptos")]
use leptos::{
    ev, html::Canvas, node_ref::NodeRef, IntoView, RwSignal, SignalGet, SignalSet, View
};

#[cfg(feature = "leptos")]
use crate::error::SigError;
#[cfg(feature = "leptos")]
use crate::signature_core::SignaturePad;

#[cfg(feature = "leptos")]
#[derive(Clone)]
pub struct LProps {
    pub contract_container_id: String,
    pub place_x:               i32,
    pub place_y:               i32,
    pub place_width:           i32
}

#[cfg(feature = "leptos")]
pub fn signature_pad_leptos(props: LProps) -> View {
    let canvas_ref: NodeRef<Canvas> = NodeRef::new();
    let pad: RwSignal<Option<std::rc::Rc<std::cell::RefCell<SignaturePad>>>> = RwSignal::new(None);

    let on_mount = {
        let canvas_ref = canvas_ref.clone();
        move |_| {
            if let Some(canvas) = canvas_ref.get() {
                if let Ok(p) = SignaturePad::new(canvas) {
                    pad.set(Some(std::rc::Rc::new(std::cell::RefCell::new(p))));
                }
            }
        }
    };

    let pointer_down = {
        let pad = pad.clone();
        move |ev: web_sys::PointerEvent| {
            if let Some(p) = pad.get() {
                if let Ok(rect) = ev
                    .target()
                    .and_then(|t| t.dyn_into::<web_sys::HtmlCanvasElement>().ok())
                    .and_then(|c| Some(c.get_bounding_client_rect()))
                {
                    let x = ev.client_x() as f64 - rect.x();
                    let y = ev.client_y() as f64 - rect.y();
                    p.borrow_mut().pointer_down(x, y);
                }
            }
        }
    };

    let pointer_move = {
        let pad = pad.clone();
        move |ev: web_sys::PointerEvent| {
            if let Some(p) = pad.get() {
                if let Ok(rect) = ev
                    .target()
                    .and_then(|t| t.dyn_into::<web_sys::HtmlCanvasElement>().ok())
                    .and_then(|c| Some(c.get_bounding_client_rect()))
                {
                    let x = ev.client_x() as f64 - rect.x();
                    let y = ev.client_y() as f64 - rect.y();
                    p.borrow_mut().pointer_move(x, y);
                }
            }
        }
    };

    let pointer_up = {
        let pad = pad.clone();
        move |_ev: web_sys::PointerEvent| {
            if let Some(p) = pad.get() {
                p.borrow_mut().pointer_up();
            }
        }
    };

    let do_sign = {
        let pad = pad.clone();
        let props = props.clone();
        move |_ev: web_sys::MouseEvent| {
            if let Some(p) = pad.get() {
                if let Ok(data_url) = p.borrow().to_png_data_url() {
                    let _ = crate::ui_common::DomBindings::place_signature_img(
                        &props.contract_container_id,
                        &data_url,
                        props.place_x,
                        props.place_y,
                        props.place_width
                    );
                }
            }
        }
    };

    let do_clear = {
        let pad = pad.clone();
        move |_ev: web_sys::MouseEvent| {
            if let Some(p) = pad.get() {
                p.borrow_mut().clear();
            }
        }
    };

    leptos::view! {
        on:load=on_mount
        <div class="signature-area">
            <canvas
                _ref=canvas_ref
                width="500" height="200"
                on:pointerdown=pointer_down
                on:pointermove=pointer_move
                on:pointerup=pointer_up
                on:pointerleave=pointer_up
            />
            <div class="controls">
                <button on:click=do_sign>{"Подписать"}</button>
                <button on:click=do_clear>{"Очистить"}</button>
            </div>
        </div>
    }
}
