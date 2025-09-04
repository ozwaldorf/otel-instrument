#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashSet;
use syn::{Expr, Ident, ItemFn, Token, parse::Parse, parse::ParseStream, parse_macro_input};

#[derive(Default)]
struct InstrumentArgs {
    skip: HashSet<String>,
    skip_all: bool,
    fields: Vec<(String, Expr)>,
    ret: bool,
    err: bool,
    name: Option<String>,
    parent: Option<Expr>,
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
                "name" => {
                    input.parse::<Token![=]>()?;
                    let name_str: syn::LitStr = input.parse()?;
                    args.name = Some(name_str.value());
                }
                "parent" => {
                    input.parse::<Token![=]>()?;
                    let parent_expr: Expr = input.parse()?;
                    args.parent = Some(parent_expr);
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

/// Define the global tracer name for instrumentation.
/// If not called, defaults to "otel-instrument".
///
/// # Example
/// ```rust
/// use otel_instrument::tracer_name;
///
/// tracer_name!("my-service");
/// ```
#[proc_macro]
pub fn tracer_name(input: TokenStream) -> TokenStream {
    let tracer_name = if input.is_empty() {
        "otel-instrument".to_string()
    } else {
        let literal: syn::LitStr = parse_macro_input!(input as syn::LitStr);
        literal.value()
    };

    let expanded = quote! {
        pub(crate) const _OTEL_TRACER_NAME: &str = #tracer_name;
    };

    expanded.into()
}

/// See crate level documentation for usage.
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
    let span_name = args.name.unwrap_or(fn_name_str.clone());

    // Check if function is async
    let is_async = input_fn.sig.asyncness.is_some();

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
                ::opentelemetry::trace::get_active_span(|span| {
                    span.set_attribute(::opentelemetry::KeyValue::new("return", format!("{:?}", ret_val)));
                });
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
                    ::opentelemetry::trace::get_active_span(|span| {
                        span.set_status(::opentelemetry::trace::Status::Ok);
                    });
                }
                Err(e) => {
                    ::opentelemetry::trace::get_active_span(|span| {
                        span.set_attribute(::opentelemetry::KeyValue::new("error", format!("{:?}", e)));
                        span.set_status(::opentelemetry::trace::Status::error(format!("{:?}", e)));
                    });
                }
            }
        }
    } else {
        quote! {
            match &result {
                Ok(_) => {
                    ::opentelemetry::trace::get_active_span(|span| {
                        span.set_status(::opentelemetry::trace::Status::Ok);
                    });
                }
                Err(e) => {
                    ::opentelemetry::trace::get_active_span(|span| {
                        span.set_status(::opentelemetry::trace::Status::error(format!("{:?}", e)));
                    });
                }
            }
        }
    };

    // Generate span creation code based on whether parent is specified
    let span_creation = if let Some(parent_expr) = &args.parent {
        quote! {
            let parent_ctx = {
                use ::opentelemetry::Context;
                let parent_value = #parent_expr;

                // The parent_value should implement Into<Context> or be a Context
                // This allows for flexibility in what users can pass:
                // - Context directly
                // - Span (which can be converted to Context)
                // - SpanContext (which can be used to create Context)
                parent_value.into()
            };
            let mut span = tracer.start_with_context(#span_name, &parent_ctx);
        }
    } else {
        quote! {
            let mut span = tracer.start(#span_name);
        }
    };

    // Generate the result execution block based on whether function is async or sync
    let result_block = if is_async {
        quote! {
            let result = async move #original_block.with_current_context().await;
        }
    } else {
        quote! {
            let result = #original_block;
        }
    };

    // Generate the imports based on whether function is async or sync
    let imports = if is_async {
        quote! {
            use ::opentelemetry::{trace::{Tracer, Span}, context::FutureExt, global};
        }
    } else {
        quote! {
            use ::opentelemetry::{trace::{Tracer, Span}, global};
        }
    };

    // Create the instrumented function body
    let instrumented_body = quote! {
        {
            #imports

            let tracer = global::tracer(_OTEL_TRACER_NAME);
            #span_creation
            #(#span_attrs)*
            #(#field_attrs)*
            let _guard = ::opentelemetry::trace::mark_span_as_active(span);

            // Execute the original function with instrumentation
            #result_block
            // Capture return value if requested
            #ret_capture
            // Set result status and error capture
            #err_capture

            result
        }
    };

    // Replace the function body
    input_fn.block = syn::parse2(instrumented_body)?;

    Ok(quote! { #input_fn })
}
