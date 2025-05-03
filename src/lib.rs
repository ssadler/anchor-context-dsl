
mod indented;
mod types;
mod expand;
mod parse;
mod propsets;

use syn::*;
use quote::*;
use indented::*;
use types::*;


#[proc_macro]
pub fn yaml_contexts(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {

    //panic!("{:?}", tokens);
    let indented = create_indented_tokenstream(tokens);
    let doc = syn::parse_macro_input!(indented as YamlDoc);
    //panic!("{:?}", doc);
    Default::default()
}
