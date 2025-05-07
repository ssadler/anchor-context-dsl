mod build;
mod build_from_mod;
mod indented;
mod parse;
mod programs;
mod propsets;
mod render;
mod types;

use build::build_contexts;
use indented::*;
use render::compile;
use types::*;

#[proc_macro]
pub fn yaml_contexts(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let indented = match create_indented_tokenstream(tokens) {
        Some(i) => i,
        None => return Default::default(),
    };
    let doc = syn::parse_macro_input!(indented as YamlDoc);
    let o = build_contexts(&doc).map(compile);

    eprintln!("{}", o.clone().unwrap());

    o.unwrap().into()
}

#[proc_macro_attribute]
pub fn anchor_context_dsl(
    args: proc_macro::TokenStream,
    tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let indented = match create_indented_tokenstream(args) {
        Some(i) => i,
        None => return Default::default(),
    };
    let p = |input: syn::parse::ParseStream| {
        let YamlDoc(_, doc) = input.parse()?;
        build_from_mod::build_contexts_with_module(tokens, doc)
    };
    return syn::parse_macro_input!(indented with p);
}

#[proc_macro]
pub fn yaml_contexts_to_string(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {

    let indented = match create_indented_tokenstream(tokens) {
        Some(i) => i,
        None => return Default::default(),
    };
    let doc = syn::parse_macro_input!(indented as YamlDoc);
    let o = build_contexts(&doc).map(compile).unwrap();

    let code = format!("{}", o);

    let code = code
        .replace("\"#[account(", "\n#[account(")
        .replace(")]\"", ")]")
        .replace("\\n", "\n")
        .replace("\\\"", "\"")
        .replace("#[derive(Accounts)]", "\n\n\n#[derive(Accounts)]");

    quote::quote! { #code }.into()
}

// TODO
//
// extract seed const strings
// no mut by default
// args are boolean
// merge constraints
// clean up indented
