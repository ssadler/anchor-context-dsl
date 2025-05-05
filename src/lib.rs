

mod indented;
mod propsets;
mod types;
mod parse;
mod build;
mod programs;
mod render;

use build::build_contexts;
use indented::*;
use types::*;
use render::compile;


#[proc_macro]
pub fn yaml_contexts(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {

    let indented = match create_indented_tokenstream(tokens) {
        Some(i) => i,
        None => return Default::default()
    };
    let doc = syn::parse_macro_input!(indented as YamlDoc);
    let o = build_contexts(&doc).map(compile);

    //eprintln!("{}", o.clone().unwrap());

    o.unwrap().into() // .unwrap_or_else(syn::Error::into_compile_error).into()
}


// TODO
// 
// extract seed const strings
// no mut by default
// args are boolean
// merge constraints
// clean up indented
