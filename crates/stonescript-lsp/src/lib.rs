//! StoneScript LSP Library

pub mod data;
pub mod server;
pub mod utils;
pub mod providers;

pub use server::Backend;
pub use utils::{ScopeAnalyzer, infer_type};
pub use providers::CompletionProvider;
