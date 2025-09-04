use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// A procedural macro that instruments async functions with OpenTelemetry tracing.
/// Similar to tracing's #[instrument] macro but specifically designed for OpenTelemetry.
///
/// # Example
///
/// ```rust
/// use otel_instrument::instrument;
///
/// #[instrument]
/// async fn my_function(param: &str) -> Result<String, String> {
///     // Your async code here
///     Ok(format!("Hello, {}", param))
/// }
/// ```
#[proc_macro_attribute]
pub fn instrument(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);

    match instrument_impl(input_fn) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn instrument_impl(
    mut input_fn: ItemFn,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let fn_name = &input_fn.sig.ident;
    let fn_name_str = fn_name.to_string();

    // Check if function is async
    if input_fn.sig.asyncness.is_none() {
        return Err(syn::Error::new_spanned(
            &input_fn.sig,
            "instrument macro can only be applied to async functions",
        ));
    }

    // Extract function parameters for span attributes
    let param_names: Vec<_> = input_fn
        .sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            syn::FnArg::Typed(pat_type) => {
                if let syn::Pat::Ident(ident) = pat_type.pat.as_ref() {
                    Some(&ident.ident)
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect();

    // Generate span attributes from parameters
    let span_attrs = param_names.iter().map(|name| {
        let name_str = name.to_string();
        quote! {
            span.set_attribute(::opentelemetry::KeyValue::new(#name_str, format!("{:?}", #name)));
        }
    });

    // Get the original function body
    let original_block = &input_fn.block;

    // Create the instrumented function body
    let instrumented_body = quote! {
        {
            use ::opentelemetry::{trace::{Tracer, Span}, global};

            let tracer = global::tracer("otel-instrument");
            let mut span = tracer.start(#fn_name_str);

            // Set parameter attributes
            #(#span_attrs)*

            // Execute the original function with instrumentation
            let result = async move #original_block.await;

            // Set result status
            match &result {
                Ok(_) => {
                    span.set_status(::opentelemetry::trace::Status::Ok);
                }
                Err(e) => {
                    span.set_status(::opentelemetry::trace::Status::error(format!("{:?}", e)));
                }
            }

            span.end();
            result
        }
    };

    // Replace the function body
    input_fn.block = syn::parse2(instrumented_body)?;

    Ok(quote! { #input_fn })
}
