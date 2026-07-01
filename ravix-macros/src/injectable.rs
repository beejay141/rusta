use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, Fields, ItemStruct};

pub fn expand(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_struct = parse_macro_input!(input as ItemStruct);
    let struct_name = &input_struct.ident;

    let field_assignments: Vec<TokenStream2> = match &input_struct.fields {
        Fields::Named(fields) => fields
            .named
            .iter()
            .map(|f| {
                let field_name = f.ident.as_ref().unwrap();
                let field_ty = &f.ty;

                let has_inject = f
                    .attrs
                    .iter()
                    .any(|a| a.path().get_ident().map(|i| i == "inject").unwrap_or(false));

                if has_inject {
                    // The field type (e.g. Arc<dyn UserRepository>) IS the container key.
                    // container.resolve::<Arc<dyn UserRepository>>() returns Arc<dyn UserRepository>.
                    quote! { #field_name: container.resolve::<#field_ty>(), }
                } else {
                    quote! { #field_name: ::std::default::Default::default(), }
                }
            })
            .collect(),
        _ => vec![],
    };

    // Strip #[inject] from fields in the emitted struct so the compiler
    // does not see an unknown attribute on the struct field.
    let mut clean = input_struct.clone();
    if let Fields::Named(ref mut fields) = clean.fields {
        for field in fields.named.iter_mut() {
            field
                .attrs
                .retain(|a| a.path().get_ident().map(|i| i != "inject").unwrap_or(true));
        }
    }

    let expanded = quote! {
        #clean

        impl ::ravix::Injectable for #struct_name {
            fn construct(container: &::ravix::Container) -> ::std::sync::Arc<Self> {
                ::std::sync::Arc::new(Self {
                    #(#field_assignments)*
                })
            }
        }
    };

    expanded.into()
}
