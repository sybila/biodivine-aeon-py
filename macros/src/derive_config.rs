use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

pub(crate) fn derive_config_impl(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Check if we're dealing with a struct
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return syn::Error::new_spanned(
                    &input,
                    "Config can only be used on structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(&input, "Config can only be used on structs")
                .to_compile_error()
                .into();
        }
    };

    // Check if the struct has a cancellation field of type Box<dyn CancellationHandler>
    let has_cancellation_field = fields.iter().any(|field| {
        if let Some(ident) = &field.ident {
            if ident == "cancellation" {
                // This is a simplistic check that could be improved with proper type checking
                // Currently, we just check if the type contains "Box" and "CancellationHandler"
                let type_str = quote!(#field.ty).to_string();
                return type_str.contains("Box") && type_str.contains("CancellationHandler");
            }
        }
        false
    });

    if !has_cancellation_field {
        return syn::Error::new_spanned(
            &input,
            "Config requires a field named 'cancellation' of type Box<dyn CancellationHandler>",
        )
        .to_compile_error()
        .into();
    }

    // Generate the implementation of the Config trait
    let expanded = quote! {
        impl Config for #name {
            fn cancellation(&self) -> &dyn CancellationHandler {
                self.cancellation.as_ref()
            }

            fn set_cancellation(&mut self, cancellation: Box<dyn CancellationHandler>) {
                self.cancellation = cancellation;
            }
        }
    };

    TokenStream::from(expanded)
}
