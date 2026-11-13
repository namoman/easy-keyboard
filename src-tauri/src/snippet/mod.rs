//! Snippet engine: inject / parse / crypto / persist / ring / bundle.

pub mod bundle;
pub mod clipboard_ring;
pub mod clipboard_set;
pub mod crypto;
pub mod injector;
pub mod keys;
pub mod parser;
pub mod persist;
pub mod profiles;
pub mod store;
pub mod substitute;
pub mod unicode_type;

pub use injector::{inject, InjectConfig};
pub use store::{slot_index, SnippetSlot, SnippetStore};
