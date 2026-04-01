//! Quote engine: quantity takeoff from geometry + materials, line items, tier comparison.
//!
//! Pure computation — no I/O. Takes zone geometries, material assignments, and
//! a material catalog; produces a [`Quote`] with line items, subtotal, and total.

pub mod engine;
pub mod error;
pub mod types;

pub use engine::compute_quote;
pub use error::QuoteError;
pub use types::{LineItem, Quote};
