//! Traits and supporting types that mirror the HTML Canvas 2D context surface.
//! These are interface definitions only; you can implement them for any backend
//! (software rasterizer, OpenGL, WebGPU, etc.).

pub mod api;
pub mod error;


#[cfg(feature = "cairo")]
pub mod cairo_backend;

