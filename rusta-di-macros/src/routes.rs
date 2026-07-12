use proc_macro::TokenStream;

/// Pass-through: these attributes are inert markers consumed by `#[controller]`.
/// When applied standalone they simply re-emit the item unchanged so that
/// the compiler does not see an unknown attribute.
pub fn noop_marker(_args: TokenStream, input: TokenStream) -> TokenStream {
    input
}
