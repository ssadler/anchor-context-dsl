
use syn::*;
use crate::types::*;
use crate::indented::*;
use crate::propsets::*;



impl parse::Parse for YamlDoc {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        let mut accounts = KeyedVec::<DynStruct::<ParseAccountProps>>::new();
        let mut contexts = KeyedVec::<Vec<ContextProp>>::default();


        while !input.is_empty() {
            input.parse::<Indent<0>>()?;
            let mut name = input.parse::<syn::Ident>()?;

            if name == "context" {
                name = input.parse::<syn::Ident>()?;
                input.parse::<Token![:]>()?;
                let props = parse_many_indented::<Indent<1>, ContextProp>(input)?;
                contexts.push((name, props));
            } else {
                if accounts.has(&name) {
                    panic!("account exists: {}", name);
                }
                input.parse::<Token![:]>()?;

                accounts.push((name, parse_indented::<Indent<1>, _>(input)?));
            }
        }

        Ok(YamlDoc(YamlContexts(contexts), accounts))
    }
}


impl<L: IndentLevel> ParseIndented<L> for ContextProp {
    fn parse_indented(input: parse::ParseStream) -> Result<Self> {

        let id = input.parse::<syn::Ident>()?;
        input.parse::<Token![:]>()?;

        let content;
        syn::bracketed!(content in input);

        let args = content.parse_terminated::<syn::Ident, Token![,]>(|p| p.parse())?;
        let args = args.into_iter().collect();
        Ok(ContextProp::Account { name: id, args })
    }
}


macro_rules! define_prop_parser {
    ($struct_name:ident, $f:ident) => {
        impl<L: IndentLevel> ParseIndented<L> for $struct_name {
            fn parse_indented(input: parse::ParseStream) -> Result<Self> {
                Ok($struct_name($f::<L::Next, _>(input)?))
            }
        }
    };
    ($struct_name:ident) => { define_prop_parser!($struct_name, parse_indented_token); }
}

define_prop_parser!(Mut);
define_prop_parser!(Constraints, parse_yaml_token_array);
define_prop_parser!(AccountType);
define_prop_parser!(Struct);
define_prop_parser!(TokenMint);
define_prop_parser!(TokenAuthority);
define_prop_parser!(Depends, parse_yaml_token_array);
define_prop_parser!(Seeds);
define_prop_parser!(Init, parse_indented);
define_prop_parser!(NoInit, parse_indented);


impl<L: IndentLevel, Set: DispatchParseIndented> ParseIndented<L> for DynStruct<Set> {
    fn parse_indented(input: parse::ParseStream) -> Result<Self> {
        let mut props = DynStruct::<Set>::new();

        parse_many_indented_fn::<L::Next>(input, || {
            props.insert_dyn(parse_indented::<L::Next, _>(input)?);
            Ok(())
        })?;

        Ok(props)
    }
}




trait DispatchParseIndented: PropSet {
    fn dispatch<L: IndentLevel>(label: &str, input: parse::ParseStream) -> Result<Self>;
}
impl DispatchParseIndented for ParseAccountProps {
    fn dispatch<L: IndentLevel>(label: &str, input: parse::ParseStream) -> Result<Self> {
        impl_prop_dispatch_ParseAccountProps!(label, |T| {
            <T as ParseIndented<L>>::parse_indented(input).map(|s| s.into())
        }, Err(parse_error!(input.span(), "invalid property")))
    }
}
impl DispatchParseIndented for RealAccountPropsSansInit {
    fn dispatch<L: IndentLevel>(label: &str, input: parse::ParseStream) -> Result<Self> {
        impl_prop_dispatch_RealAccountPropsSansInit!(label, |T| {
            <T as ParseIndented<L>>::parse_indented(input).map(|r| r.into())
        }, Err(parse_error!(input.span(), "invalid property")))
    }
}

impl<L: IndentLevel, Set: DispatchParseIndented> ParseIndented<L> for Set {
    fn parse_indented(input: parse::ParseStream) -> Result<Self> {
        let PropLabel(label) = input.fork().parse()?;
        if !Set::has_prop(label) {
            return Err(parse_error!(input.span(), "invalid property"));
        }
        let PropLabel(label) = input.fork().parse()?;
        input.parse::<Token![:]>()?;

        <Set as DispatchParseIndented>::dispatch::<L>(label, input)
    }
}




struct PropLabel(&'static str);
impl parse::Parse for PropLabel {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        if input.peek(Token![type]) {
            input.parse::<Token![type]>()?;
            Ok(PropLabel("type"))
        } else if input.peek(Token![struct]) {
            input.parse::<Token![struct]>()?;
            Ok(PropLabel("struct"))
        } else if input.peek(Token![mut]) {
            input.parse::<Token![struct]>()?;
            Ok(PropLabel("mut"))
        } else {
            let path = input.parse::<syn::Path>()?;
            let s = quote::quote!(#path).to_string();
            let s = Box::leak(s.to_owned().into_boxed_str());
            Ok(PropLabel(s))
        }
    }
}
