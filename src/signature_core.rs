use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::error::SigError;

/// Encapsulates drawing logic and export of signature.
pub struct SignaturePad {
    canvas:   HtmlCanvasElement,
    ctx:      CanvasRenderingContext2d,
    drawing:  bool,
    last_x:   f64,
    last_y:   f64,
    is_empty: bool
}

impl SignaturePad {
    /// Create pad from an existing <canvas> element.
    pub fn new(canvas: HtmlCanvasElement) -> Result<Self, SigError> {
        let ctx = canvas
            .get_context("2d")
            .map_err(|_| SigError::NoContext2d)?
            .ok_or(SigError::NoContext2d)?
            .dyn_into::<CanvasRenderingContext2d>()
            .map_err(|_| SigError::NoContext2d)?;

        // Sensible defaults for signature lines
        ctx.set_line_width(2.0);
        ctx.set_line_join("round");
        ctx.set_line_cap("round");

        Ok(Self {
            canvas,
            ctx,
            drawing: false,
            last_x: 0.0,
            last_y: 0.0,
            is_empty: true
        })
    }

    /// Handle pointer down: start drawing.
    pub fn pointer_down(&mut self, x: f64, y: f64) {
        self.drawing = true;
        self.last_x = x;
        self.last_y = y;
    }

    /// Handle pointer move: draw if active.
    pub fn pointer_move(&mut self, x: f64, y: f64) {
        if !self.drawing {
            return;
        }
        let _ = self.ctx.begin_path();
        let _ = self.ctx.move_to(self.last_x, self.last_y);
        let _ = self.ctx.line_to(x, y);
        let _ = self.ctx.stroke();
        self.last_x = x;
        self.last_y = y;
        self.is_empty = false;
    }

    /// Handle pointer up/cancel: stop drawing.
    pub fn pointer_up(&mut self) {
        self.drawing = false;
    }

    /// Clear the canvas.
    pub fn clear(&mut self) {
        let w = self.canvas.width();
        let h = self.canvas.height();
        let _ = self.ctx.clear_rect(0.0, 0.0, w as f64, h as f64);
        self.is_empty = true;
    }

    /// Is pad empty (nothing drawn)?
    pub fn is_empty(&self) -> bool {
        self.is_empty
    }

    /// Export as PNG data URL (for <img src="...">).
    pub fn to_png_data_url(&self) -> Result<String, SigError> {
        self.canvas
            .to_data_url()
            .map_err(|_| SigError::OpFailed("to_data_url".into()))
    }

    /// Export raw PNG bytes (without data URL).
    pub fn to_png_bytes(&self) -> Result<Vec<u8>, SigError> {
        // Using to_blob would be nicer, but it is async-callback based.
        // Data URL is simpler to get synchronously and then decode.
        let data_url = self.to_png_data_url()?;
        let prefix = "data:image/png;base64,";
        let b64 = data_url
            .strip_prefix(prefix)
            .ok_or_else(|| SigError::OpFailed("unexpected data URL".into()))?;
        base64_decode(b64)
    }
}

/// Minimal base64 decoder using JS at runtime to avoid extra deps.
fn base64_decode(b64: &str) -> Result<Vec<u8>, SigError> {
    let js = format!("Uint8Array.from(atob('{b64}'), c => c.charCodeAt(0))");
    let val = js_sys::eval(&js).map_err(|_| SigError::OpFailed("eval".into()))?;
    let arr = Uint8Array::new(&val);
    let mut out = vec![0u8; arr.length() as usize];
    arr.copy_to(&mut out[..]);
    Ok(out)
}
