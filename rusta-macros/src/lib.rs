use proc_macro::TokenStream;

// Re-export all macros from rusta-di-macros
pub use rusta_di_macros::{controller, delete, get, injectable, middleware, patch, post, put};
