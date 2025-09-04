use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashSet;
use syn::{parse::Parse, parse::ParseStream, parse_macro_input, Expr, Ident, ItemFn, Token};

#[derive(Default)]
struct InstrumentArgs {
    skip: HashSet<String>,
    skip_all: bool,
    fields: Vec<(String, Expr)>,
    ret: bool,
    err: bool,
}

impl Parse for InstrumentArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = InstrumentArgs::default();

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            match ident.to_string().as_str() {
                "skip_all" => {
                    args.skip_all = true;
                }
                "skip" => {
                    let content;
                    syn::parenthesized!(content in input);
                    while !content.is_empty() {
                        let param: Ident = content.parse()?;
                        args.skip.insert(param.to_string());
                        if !content.is_empty() {
                            content.parse::<Token![,]>()?;
                        }
                    }
                }
                "fields" => {
                    let content;
                    syn::parenthesized!(content in input);
                    while !content.is_empty() {
                        let field_name: Ident = content.parse()?;
                        content.parse::<Token![=]>()?;
                        let field_expr: Expr = content.parse()?;
                        args.fields.push((field_name.to_string(), field_expr));
                        if !content.is_empty() {
                            content.parse::<Token![,]>()?;
                        }
                    }
                }
                "ret" => {
                    args.ret = true;
                }
                "err" => {
                    args.err = true;
                }
                _ => {
                    return Err(syn::Error::new_spanned(ident, "Unknown attribute"));
                }
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(args)
    }
}

/// A procedural macro that instruments async functions with OpenTelemetry tracing.
/// Similar to tracing's #[instrument] macro but specifically designed for OpenTelemetry.
///
/// # Attributes
///
/// - `skip(param1, param2)`: Skip specific parameters from being recorded as attributes
/// - `skip_all`: Skip all parameters from being recorded as attributes  
/// - `fields(key = value)`: Add custom fields to the span
/// - `ret`: Record the return value as a span attribute
/// - `err`: Record error values as span attributes
///
/// # Example
///
/// ```rust
/// use otel_instrument::instrument;
///
/// #[instrument(skip(password), ret, err)]
/// async fn my_function(username: &str, password: &str) -> Result<String, String> {
///     // Your async code here
///     Ok(format!("Hello, {}", username))
/// }
/// ```
#[proc_macro_attribute]
pub fn instrument(args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    let args = if args.is_empty() {
        InstrumentArgs::default()
    } else {
        parse_macro_input!(args as InstrumentArgs)
    };

    match instrument_impl(args, input_fn) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn instrument_impl(
    args: InstrumentArgs,
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

    // Generate span attributes from parameters (respecting skip and skip_all)
    let span_attrs: Vec<_> = if args.skip_all {
        Vec::new()
    } else {
        param_names.iter()
            .filter(|name| !args.skip.contains(&name.to_string()))
            .map(|name| {
                let name_str = name.to_string();
                quote! {
                    span.set_attribute(::opentelemetry::KeyValue::new(#name_str, format!("{:?}", #name)));
                }
            })
            .collect()
    };

    // Generate custom field attributes
    let field_attrs = args.fields.iter().map(|(name, expr)| {
        quote! {
            span.set_attribute(::opentelemetry::KeyValue::new(#name, format!("{:?}", #expr)));
        }
    });

    // Get the original function body
    let original_block = &input_fn.block;

    // Generate return value capture if requested
    let ret_capture = if args.ret {
        quote! {
            if let Ok(ref ret_val) = result {
                span.set_attribute(::opentelemetry::KeyValue::new("return", format!("{:?}", ret_val)));
            }
        }
    } else {
        quote! {}
    };

    // Generate error capture if requested (enhanced version)
    let err_capture = if args.err {
        quote! {
            match &result {
                Ok(_) => {
                    span.set_status(::opentelemetry::trace::Status::Ok);
                }
                Err(e) => {
                    span.set_attribute(::opentelemetry::KeyValue::new("error", format!("{:?}", e)));
                    span.set_status(::opentelemetry::trace::Status::error(format!("{:?}", e)));
                }
            }
        }
    } else {
        quote! {
            match &result {
                Ok(_) => {
                    span.set_status(::opentelemetry::trace::Status::Ok);
                }
                Err(e) => {
                    span.set_status(::opentelemetry::trace::Status::error(format!("{:?}", e)));
                }
            }
        }
    };

    // Create the instrumented function body
    let instrumented_body = quote! {
        {
            use ::opentelemetry::{trace::{Tracer, Span}, context::FutureExt, global};

            let tracer = global::tracer("otel-instrument");
            let mut span = tracer.start(#fn_name_str);

            // Set parameter attributes
            #(#span_attrs)*

            // Set custom field attributes
            #(#field_attrs)*

            // Execute the original function with instrumentation
            let result = async move #original_block.with_current_context().await;

            // Capture return value if requested
            #ret_capture

            // Set result status and error capture
            #err_capture

            span.end();
            result
        }
    };

    // Replace the function body
    input_fn.block = syn::parse2(instrumented_body)?;

    Ok(quote! { #input_fn })
}
