extern crate proc_macro;

mod derive_config;

use proc_macro::TokenStream;
use std::io::Write;
use syn::__private::ToTokens;

#[proc_macro_derive(Wrapper)]
pub fn derive_wrapper(item: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(item).unwrap();
    let wrapper_type = ast.ident.to_string();
    let fields = if let syn::Data::Struct(x) = ast.data {
        x.fields
    } else {
        panic!("Expected data struct.");
    };
    if fields.len() != 1 {
        panic!("Only single-field structs can be wrappers.");
    }
    let field = fields.into_iter().next().unwrap();
    if field.ident.is_some() {
        panic!("Only anonymous fields are allowed in wrappers.");
    }
    let wrapped_type = field.ty.to_token_stream().to_string();
    let mut result = Vec::new();

    // Wrapper to wrapped conversion.
    writeln!(result, "impl From<{wrapper_type}> for {wrapped_type} {{").unwrap();
    writeln!(result, "   fn from(value: {wrapper_type}) -> Self {{").unwrap();
    writeln!(result, "      value.0").unwrap();
    writeln!(result, "   }}").unwrap();
    writeln!(result, "}}").unwrap();

    // Wrapped to wrapper conversion.
    writeln!(result, "impl From<{wrapped_type}> for {wrapper_type} {{").unwrap();
    writeln!(result, "   fn from(value: {wrapped_type}) -> Self {{").unwrap();
    writeln!(result, "      {wrapper_type}(value)").unwrap();
    writeln!(result, "   }}").unwrap();
    writeln!(result, "}}").unwrap();

    // AsNative implementation.
    writeln!(
        result,
        "impl crate::AsNative<{wrapped_type}> for {wrapper_type} {{"
    )
    .unwrap();
    writeln!(result, "   fn as_native(&self) -> &{wrapped_type} {{").unwrap();
    writeln!(result, "      &self.0").unwrap();
    writeln!(result, "   }}").unwrap();
    writeln!(
        result,
        "   fn as_native_mut(&mut self) -> &mut {wrapped_type} {{"
    )
    .unwrap();
    writeln!(result, "      &mut self.0").unwrap();
    writeln!(result, "   }}").unwrap();
    writeln!(result, "}}").unwrap();

    let result = String::from_utf8(result).unwrap();
    result.parse().unwrap()
}

/// A derive macro that automatically implements the `Config` trait for structs
/// that have a `cancellation` field of type `Box<dyn CancellationHandler>`.
///
/// # Example
///
/// ```
/// #[derive(Config)]
/// struct MyConfig {
///     pub cancellation: Box<dyn CancellationHandler>,
///     // other fields...
/// }
/// ```
#[proc_macro_derive(Config)]
pub fn derive_config(input: TokenStream) -> TokenStream {
    derive_config::derive_config_impl(input)
}
