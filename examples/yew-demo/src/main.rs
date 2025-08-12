use contract_signature::{SignaturePadYew, SignatureStyle};
use wasm_bindgen::{prelude::*, JsCast};
use yew::prelude::*;

// Call global JS function window.exportContractToPdf(containerId, filename)
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = exportContractToPdf)]
    fn export_contract_to_pdf(container_id: &str, filename: &str);
}

#[function_component(App)]
fn app() -> Html {
    let can_sign = use_state(|| false);
    let has_signed = use_state(|| false);

    // Подхватываем чекбокс из DOM (как и делали)
    {
        let can_sign = can_sign.clone();
        use_effect_with((), move |_| {
            if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                if let Some(cb) = doc
                    .get_element_by_id("read-checkbox")
                    .and_then(|e| e.dyn_into::<web_sys::HtmlInputElement>().ok())
                {
                    can_sign.set(cb.checked());
                    let state = can_sign.clone();
                    let cb_for_closure = cb.clone();
                    let closure =
                        wasm_bindgen::closure::Closure::wrap(Box::new(move |_e: web_sys::Event| {
                            state.set(cb_for_closure.checked());
                        })
                            as Box<dyn FnMut(_)>);
                    let _ = cb.add_event_listener_with_callback(
                        "change",
                        closure.as_ref().unchecked_ref()
                    );
                    closure.forget();
                }
            }
            || ()
        });
    }

    // Колбэк от компонента подписи: отметим, что подпись сделана
    let on_signed = {
        let has_signed = has_signed.clone();
        Callback::from(move |_data_url: String| {
            has_signed.set(true);
        })
    };

    // Клик «Скачать PDF»
    let on_export_click = {
        let has_signed = *has_signed;
        Callback::from(move |_e: web_sys::MouseEvent| {
            // Можно не запрещать, но UX логичнее — только после подписи
            if has_signed {
                export_contract_to_pdf("paper", "signed-contract.pdf");
            }
        })
    };

    html! {
        <>
          <SignaturePadYew
            contract_container_id={"contract-root".to_string()}
            // fallback, если вдруг якорь не найдётся
            place_x={520}
            place_y={80}
            place_width={220}
            anchor_id={Some("signature-anchor".to_string())}
            enabled = {*can_sign}
            style = { Some(SignatureStyle{
                border: "2px dashed #2b8a3e".into(),
                background: "#ffffff".into(),
                width: 520,
                height: 220,
                line_width: 2.0
            })}
            on_signed={on_signed}
          />

          // Кнопка экспорта, активна только после подписи
          <div class="export">
            <button class="secondary"
                    onclick={on_export_click}
                    disabled={!*has_signed}>
              {"Скачать PDF"}
            </button>
            {
              if !*has_signed {
                html! { <span class="export__hint">{"Подпишитесь, чтобы скачать PDF"}</span> }
              } else {
                Html::default()
              }
            }
          </div>
        </>
    }
}

fn main() {
    if let Some(root) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("app"))
    {
        yew::Renderer::<App>::with_root(root).render();
    } else {
        yew::Renderer::<App>::new().render();
    }
}
