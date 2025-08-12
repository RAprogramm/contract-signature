mod error;
mod signature_core;
mod ui_common;

#[cfg(feature = "yew")]
mod yew;

#[cfg(feature = "leptos")]
mod leptos;

pub use error::SigError;
pub use signature_core::SignaturePad;

#[cfg(feature = "leptos")]
pub use crate::leptos::{signature_pad_leptos, LProps};
#[cfg(feature = "yew")]
pub use crate::yew::{SignaturePadYew, SignatureProps, SignatureStyle};
