use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type, TypePath};

pub(crate) fn derive_configurable_impl(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Check if we're dealing with a struct
    let fields = match &input.data {
        Data::Struct(data) => &data.fields,
        _ => {
            return syn::Error::new_spanned(&input, "Configurable can only be used on structs")
                .to_compile_error()
                .into();
        }
    };

    // Get the config field and its type - only allow a single unnamed field or a field named "config"
    let config_field = match fields {
        // Case 1: Tuple struct with exactly one element
        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
            // Get the first and only field
            fields.unnamed.first().unwrap()
        }
        // Case 2: Struct with a field named "config"
        Fields::Named(fields) => {
            // Find a field named "config"
            if let Some(config_field) = fields.named.iter().find(|f| {
                if let Some(ident) = &f.ident {
                    ident == "config"
                } else {
                    false
                }
            }) {
                config_field
            } else {
                return syn::Error::new_spanned(
                    &input,
                    "Configurable requires a field named 'config' for structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        }
        // Any other case is not allowed
        _ => {
            return syn::Error::new_spanned(
                &input,
                "Configurable can only be used on structs with exactly one unnamed field or with a named 'config' field",
            )
            .to_compile_error()
            .into();
        }
    };

    let config_type = &config_field.ty;

    // Verify that the type implements Config
    // This is a heuristic check - ideally we'd need to resolve trait implementations
    // but that's beyond the scope of proc macros
    let is_valid_type = match config_type {
        Type::Path(TypePath { path, .. }) => {
            // Check if any segment of the path contains "Config" in its name
            // This is a heuristic, but it's better than no checking
            path.segments
                .iter()
                .any(|segment| segment.ident.to_string().contains("Config"))
        }
        _ => false,
    };

    if !is_valid_type {
        let error_message = match fields {
            Fields::Unnamed(_) => "The field should have a type that implements Config trait (name should contain 'Config')",
            Fields::Named(_) => "The 'config' field should have a type that implements Config trait (name should contain 'Config')",
            _ => unreachable!(),
        };
        return syn::Error::new_spanned(config_field, error_message)
            .to_compile_error()
            .into();
    }

    // Generate the implementation of the Configurable trait
    let config_accessor = match fields {
        Fields::Unnamed(_) => quote! { &self.0 },
        Fields::Named(_) => quote! { &self.config },
        _ => unreachable!(),
    };

    // Generate appropriate constructor based on field type
    let constructor = match fields {
        Fields::Unnamed(_) => quote! { Self(config) },
        Fields::Named(_) => quote! {
            Self {
                config,
                ..Default::default()
            }
        },
        _ => unreachable!(),
    };

    let expanded = quote! {
        impl Configurable for #name {
            type ConfigType = #config_type;

            fn config(&self) -> &Self::ConfigType {
                #config_accessor
            }

            fn with_config(config: Self::ConfigType) -> Self {
                #constructor
            }
        }
    };

    TokenStream::from(expanded)
}
