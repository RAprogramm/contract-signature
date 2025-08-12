use contract_signature::{signature_pad_leptos, LProps};
use leptos::*;

/// Small Leptos CSR example that enables signing after "read" checkbox.
#[component]
fn App() -> impl IntoView {
    // We only toggle UI affordance; the actual "Sign" button is inside the lib
    // component.
    view! {
        <div class="signature-area">
            { signature_pad_leptos(LProps {
                contract_container_id: "contract-root".into(),
                place_x: 120,
                place_y: 350,
                place_width: 220,
            }) }
            <style>{"
              #sign-btn:disabled { opacity: 0.5; cursor: not-allowed; }
            "}</style>
        </div>

        // Disable/enable the internal sign button based on checkbox state.
        <script>
            {"
              (function(){
                var cb = document.getElementById('read-checkbox');
                function sync(){
                  var btn = document.getElementById('sign-btn');
                  if (!btn) return;
                  btn.disabled = !cb.checked;
                }
                if (cb) {
                  cb.addEventListener('change', sync);
                  setTimeout(sync, 100);
                }
              })();
            "}
        </script>
    }
}

pub fn main() {
    mount_to_body(|| view! { <App/> });
}
