
mod indented;
mod propsets;
mod types;
mod parse;
mod build;
mod compile;

use build::build_contexts;
use indented::*;
use types::*;
use compile::compile;


#[proc_macro]
pub fn yaml_contexts(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let indented = match create_indented_tokenstream(tokens) {
        Some(i) => i,
        None => return Default::default()
    };
    let doc = syn::parse_macro_input!(indented as YamlDoc);
    build_contexts(&doc).map(compile).unwrap_or_else(syn::Error::into_compile_error).into()
}
