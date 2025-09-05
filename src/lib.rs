#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashSet;
use syn::{
    Expr, Ident, ItemFn, Token,
    ext::IdentExt,
    parse::{Parse, ParseStream},
    parse_macro_input,
    spanned::Spanned,
};

#[derive(Default)]
struct InstrumentArgs {
    skip: HashSet<String>,
    skip_all: bool,
    fields: Vec<(String, Expr)>,
    ret: bool,
    err: Option<Expr>,
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
                    let names = content.parse_terminated(Ident::parse_any, Token![,])?;
                    args.skip = names.into_iter().map(|i| i.to_string()).collect();
                }
                "fields" => {
                    let content;
                    syn::parenthesized!(content in input);
                    while !content.is_empty() {
                        let field_name: Ident = content.parse()?;
                        let field_expr = if content.peek(Token![=]) {
                            content.parse::<Token![=]>()?;
                            content.parse::<Expr>()?
                        } else {
                            // Fallback to name = name shorthand
                            syn::parse_quote!(#field_name)
                        };
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
                    if input.peek(Token![=]) {
                        input.parse::<Token![=]>()?;
                        let err_expr: Expr = input.parse()?;
                        args.err = Some(err_expr);
                    } else {
                        args.err = Some(syn::parse_quote!(e));
                    }
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
    let mut self_ident = None;
    let param_names: Vec<_> = input_fn
        .sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            syn::FnArg::Typed(pat_type) => {
                if let syn::Pat::Ident(ident) = pat_type.pat.as_ref() {
                    Some(ident.ident.clone())
                } else {
                    None
                }
            }
            syn::FnArg::Receiver(recv) => {
                self_ident = Some(Ident::new("self", recv.span()));
                None
            }
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

    // Generate return value capture if requested
    let ret_capture = args
        .ret
        .then_some(quote! {
            if let Ok(ref ret_val) = result {
                ::opentelemetry::trace::get_active_span(|span| {
                    span.set_attribute(
                        ::opentelemetry::KeyValue::new("return", format!("{:?}", ret_val))
                    );
                });
            }
        })
        .unwrap_or_default();

    // Generate error capture if requested (enhanced version)
    let err_capture = if let Some(err_expr) = &args.err {
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
                        let err = #err_expr;
                        span.record_error(err);
                    });
                }
            }
        }
    } else {
        quote! {
            if let Ok(_) = result {
               ::opentelemetry::trace::get_active_span(|span| {
                   span.set_status(::opentelemetry::trace::Status::Ok);
               });
            }
        }
    };

    // Generate span creation code based on whether parent is specified
    let span_creation = if let Some(parent_expr) = &args.parent {
        quote! {
            use ::opentelemetry::Context;
            // The parent_value should implement Into<Context> or be a Context
            // This allows for flexibility in what users can pass:
            // - Context directly
            // - Span (which can be converted to Context)
            // - SpanContext (which can be used to create Context)
            let parent_ctx = #parent_expr.clone().into();
            let mut span = tracer.start_with_context(#span_name, &parent_ctx);
        }
    } else {
        quote! { let mut span = tracer.start(#span_name); }
    };

    let mut original_fn = input_fn.clone();
    original_fn.sig.ident = syn::Ident::new(
        &(input_fn.sig.ident.to_string() + "original"),
        input_fn.sig.span(),
    );
    let original_ident = original_fn.sig.ident.clone();
    let call = if let Some(ident) = self_ident {
        quote! {
            #ident.#original_ident(#(#param_names),*)
        }
    } else {
        quote! {
            #original_ident(#(#param_names),*)
        }
    };

    // Generate the result execution block based on whether function is async or sync
    let result_block = if is_async {
        quote! {
            use ::opentelemetry::{context::FutureExt, trace::TraceContextExt};
            let result = async move {
                let result = #call.await;
                #ret_capture
                #err_capture
                result
            }
            .with_context(::opentelemetry::Context::current_with_span(span))
            .await;
        }
    } else {
        quote! {
            let _guard = ::opentelemetry::trace::mark_span_as_active(span);
            let result = #call;
            #ret_capture
            #err_capture
        }
    };

    // Create the instrumented function body
    let instrumented_body = quote! {
        {
            use ::opentelemetry::{trace::{Tracer, Span}, global};

            let tracer = global::tracer(_OTEL_TRACER_NAME);
            #span_creation
            #(#span_attrs)*
            #(#field_attrs)*
            #result_block
            result
        }
    };

    // Replace the function body
    input_fn.block = syn::parse2(instrumented_body)?;

    Ok(quote! {
        #[doc(hidden)]
        #original_fn
        #input_fn
    })
}
