// src/yew.rs
// feature = "yew"

#[cfg(feature = "yew")]
use std::{cell::RefCell, rc::Rc};

#[cfg(feature = "yew")]
use wasm_bindgen::JsCast;
#[cfg(feature = "yew")]
use web_sys::HtmlCanvasElement;
#[cfg(feature = "yew")]
use yew::{
    function_component, html, use_effect_with, use_node_ref, use_state, Callback, Html, Properties
};

#[cfg(feature = "yew")]
use crate::signature_core::SignaturePad;

/// Visual customization for the signature canvas area.
#[cfg(feature = "yew")]
#[derive(Clone, PartialEq)]
pub struct SignatureStyle {
    /// Canvas border CSS, e.g. "2px dashed #2b8a3e"
    pub border:     String,
    /// Canvas background CSS color, e.g. "#fff"
    pub background: String,
    /// Canvas width/height in CSS pixels
    pub width:      u32,
    pub height:     u32,
    /// Pen stroke width (kept for future API; core defaults to 2.0)
    pub line_width: f64
}

#[cfg(feature = "yew")]
impl Default for SignatureStyle {
    fn default() -> Self {
        Self {
            border:     "2px dashed #888".into(),
            background: "#fff".into(),
            width:      520,
            height:     220,
            line_width: 2.0
        }
    }
}

/// Yew wrapper component for the signature modal and placement.
#[cfg(feature = "yew")]
#[derive(Properties, PartialEq, Clone)]
pub struct SignatureProps {
    /// Contract container where signature image will be placed (absolute
    /// positioning inside).
    pub contract_container_id: String,

    /// Fallback coordinates (used if anchor_id is None or anchor not found).
    pub place_x:     i32,
    pub place_y:     i32,
    pub place_width: i32,

    /// Whether user is allowed to sign (after reading/checkbox).
    #[prop_or(true)]
    pub enabled: bool,

    /// Visual style of the signature canvas.
    #[prop_or_default]
    pub style: Option<SignatureStyle>,

    /// Optional anchor element id inside the contract container.
    /// If present and resolved, we place the signature aligned to this anchor.
    #[prop_or_default]
    pub anchor_id: Option<String>,

    /// Optional callback called with data URL of the signature PNG.
    #[prop_or_default]
    pub on_signed: Option<Callback<String>>
}

#[cfg(feature = "yew")]
#[function_component(SignaturePadYew)]
pub fn signature_pad_yew(props: &SignatureProps) -> Html {
    // Modal state
    let is_open = use_state(|| false);
    let open = {
        let is_open = is_open.clone();
        move |_| is_open.set(true)
    };
    let close = {
        let is_open = is_open.clone();
        move |_| is_open.set(false)
    };

    // SignaturePad state (appears only while modal open)
    let pad_state: yew::UseStateHandle<Option<Rc<RefCell<SignaturePad>>>> = use_state(|| None);

    // Canvas ref to init SignaturePad once the modal is rendered
    let canvas_ref = use_node_ref();

    let style = props.style.clone().unwrap_or_default();

    // Init SignaturePad when modal opens and canvas is in DOM
    {
        let canvas_ref = canvas_ref.clone();
        let pad_state = pad_state.clone();
        let style = style.clone();

        use_effect_with(*is_open, move |open_now| {
            if *open_now {
                if let Some(canvas_el) = canvas_ref.cast::<HtmlCanvasElement>() {
                    // size from props; visual border/background задаём стилями
                    canvas_el.set_width(style.width);
                    canvas_el.set_height(style.height);

                    match SignaturePad::new(canvas_el.clone()) {
                        Ok(pad) => {
                            pad_state.set(Some(Rc::new(RefCell::new(pad))));
                        }
                        Err(_e) => {
                            // noop: без канваса подписывать не будем
                            pad_state.set(None);
                        }
                    }
                }
            } else {
                // close -> drop pad
                pad_state.set(None);
            }
            || ()
        });
    }

    // Pointer handlers bound via Yew props (fire only when modal open and pad
    // exists)
    let on_down = {
        let pad_state = pad_state.clone();
        Callback::from(move |e: web_sys::PointerEvent| {
            if let Some(pad) = pad_state.as_ref() {
                if let Some(target) = e
                    .target()
                    .and_then(|t| t.dyn_into::<web_sys::HtmlCanvasElement>().ok())
                {
                    let rect = target.get_bounding_client_rect();
                    let x = e.client_x() as f64 - rect.x();
                    let y = e.client_y() as f64 - rect.y();
                    pad.borrow_mut().pointer_down(x, y);
                }
            }
        })
    };

    let on_move = {
        let pad_state = pad_state.clone();
        Callback::from(move |e: web_sys::PointerEvent| {
            if let Some(pad) = pad_state.as_ref() {
                if let Some(target) = e
                    .target()
                    .and_then(|t| t.dyn_into::<web_sys::HtmlCanvasElement>().ok())
                {
                    let rect = target.get_bounding_client_rect();
                    let x = e.client_x() as f64 - rect.x();
                    let y = e.client_y() as f64 - rect.y();
                    pad.borrow_mut().pointer_move(x, y);
                }
            }
        })
    };

    let on_up = {
        let pad_state = pad_state.clone();
        Callback::from(move |_e: web_sys::PointerEvent| {
            if let Some(pad) = pad_state.as_ref() {
                pad.borrow_mut().pointer_up();
            }
        })
    };

    // Confirm click: export, place, scroll, hide placeholder, close modal
    let on_confirm = {
        let props = props.clone();
        let pad_state = pad_state.clone();
        let is_open = is_open.clone();
        Callback::from(move |_e: web_sys::MouseEvent| {
            if !props.enabled {
                return;
            }
            if let Some(pad) = pad_state.as_ref() {
                if let Ok(data_url) = pad.borrow().to_png_data_url() {
                    let (x, y) = compute_placement(&props);
                    let _ = crate::ui_common::DomBindings::place_signature_img(
                        &props.contract_container_id,
                        &data_url,
                        x,
                        y,
                        props.place_width
                    );

                    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                        if let Some(cont_el) = doc
                            .get_element_by_id(&props.contract_container_id)
                            .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
                        {
                            cont_el.set_scroll_top(y.saturating_sub(40));
                        }
                        if let Some(anchor_id) = &props.anchor_id {
                            if let Some(ph) = doc.get_element_by_id(anchor_id) {
                                let _ = ph.set_attribute("style", "display:none");
                            }
                        }
                    }

                    if let Some(cb) = &props.on_signed {
                        cb.emit(data_url);
                    }
                    is_open.set(false);
                }
            }
        })
    };

    // Clear click
    let on_clear = {
        let pad_state = pad_state.clone();
        Callback::from(move |_e: web_sys::MouseEvent| {
            if let Some(pad) = pad_state.as_ref() {
                pad.borrow_mut().clear();
            }
        })
    };

    let open_btn_disabled = !props.enabled;

    html! {
        <>
          <button id="sign-open-btn"
                  disabled={open_btn_disabled}
                  onclick={open}>
            {"Открыть окно подписи"}
          </button>

          {
            if *is_open {
              html! {
                <>
                  <div class="cs-modal-backdrop" onclick={close.clone()}></div>
                  <div class="cs-modal">
                    <div class="cs-modal__panel"
                         onclick={Callback::from(|e: web_sys::MouseEvent| { e.stop_propagation(); })}>
                      <h3>{"Подпись"}</h3>
                      <canvas id="signature-canvas"
                              ref={canvas_ref}
                              width={style.width.to_string()}
                              height={style.height.to_string()}
                              style={format!(
                                "background:{};border:{};touch-action:none;",
                                style.background, style.border
                              )}
                              onpointerdown={on_down}
                              onpointermove={on_move}
                              onpointerup={on_up.clone()}
                              onpointerleave={on_up}
                      />
                      <div class="controls">
                        <button id="sign-confirm-btn" onclick={on_confirm.clone()} disabled={!props.enabled}>
                          {"Подписать"}
                        </button>
                        <button id="sign-clear-btn" onclick={on_clear}>{"Очистить"}</button>
                        <button class="secondary" onclick={close}>{"Отмена"}</button>
                      </div>
                    </div>
                  </div>
                </>
              }
            } else {
              Html::default()
            }
          }
        </>
    }
}

#[cfg(feature = "yew")]
fn compute_placement(props: &SignatureProps) -> (i32, i32) {
    if let Some(anchor_id) = &props.anchor_id {
        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
            let cont_el = doc.get_element_by_id(&props.contract_container_id);
            let anchor_el = doc.get_element_by_id(anchor_id);
            if let (Some(cont_el), Some(anchor_el)) = (cont_el, anchor_el) {
                if let (Ok(cont), Ok(anchor)) = (
                    cont_el.dyn_into::<web_sys::HtmlElement>(),
                    anchor_el.dyn_into::<web_sys::HtmlElement>()
                ) {
                    let cr = cont.get_bounding_client_rect();
                    let ar = anchor.get_bounding_client_rect();
                    let left = (ar.left() - cr.left()) + cont.scroll_left() as f64;
                    let top = (ar.top() - cr.top()) + cont.scroll_top() as f64;
                    return (left.round() as i32, top.round() as i32);
                }
            }
        }
    }
    (props.place_x, props.place_y)
}
