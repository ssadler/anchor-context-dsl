
use syn::spanned::Spanned;
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
                let props = parse_at_level::<Indent<1>, ContextProp>(input)?;
                contexts.insert(name, props)?;
            } else {
                if accounts.has(&name) {
                    panic!("account exists: {}", name);
                }
                let t = input.parse::<Token![:]>()?;
                //return Err(parse_error!(input.span(), "wat"));

                accounts.insert(name, parse_indented::<Indent<1>, _>(input)?)?;
            }
        }

        Ok(YamlDoc(YamlContexts(contexts), accounts))
    }
}


impl<L: IndentLevel> ParseIndented<L> for ContextProp {
    fn parse_indented(input: parse::ParseStream) -> Result<Self> {

        let id = input.parse::<syn::Ident>()?;

        if id == "instruction" {
            input.parse::<Token![:]>()?;
            let args = parse_instruction_args::<L, syn::FnArg>(input)?;
            Ok(ContextProp::Instruction { args })
        } else {

            let args = if input.peek(Token![:]) {
                input.parse::<Token![:]>()?;
                let content;
                syn::bracketed!(content in input);

                let args = content.parse_terminated::<syn::Ident, Token![,]>(|p| p.parse())?;
                args.into_iter().collect()
            } else {
                vec![]
            };
            
            Ok(ContextProp::Account { name: id, args })
        }
    }
}


macro_rules! define_prop_parser {
    ($struct_name:ident, $f:ident) => {
        impl<L: IndentLevel> ParseIndented<L> for $struct_name {
            fn parse_indented(input: parse::ParseStream) -> Result<Self> {
                Ok($struct_name($f::<L, _>(input)?))
            }
        }
    };
    ($struct_name:ident) => { define_prop_parser!($struct_name, parse_indented_token); }
}

define_prop_parser!(Space);
define_prop_parser!(Mut);
define_prop_parser!(Boxed);
define_prop_parser!(ZeroCopy);
define_prop_parser!(Check);
define_prop_parser!(Constraints, parse_yaml_token_array);
define_prop_parser!(AccountType);
define_prop_parser!(TokenMint);
define_prop_parser!(TokenAuthority);
define_prop_parser!(TokenProgram);
define_prop_parser!(AssociatedTokenMint);
define_prop_parser!(AssociatedTokenAuthority);
define_prop_parser!(AssociatedTokenProgram);
define_prop_parser!(MintAuthority);
define_prop_parser!(MintDecimals);
define_prop_parser!(MintTokenProgram);
define_prop_parser!(TransferHookAuthority);
define_prop_parser!(TransferHookProgramId);
define_prop_parser!(Depends, parse_yaml_token_array);
define_prop_parser!(Seeds);
define_prop_parser!(Init, parse_indented_next_level);
define_prop_parser!(NoInit, parse_indented_next_level);
define_prop_parser!(InitIfNeeded);

fn parse_instruction_args<I: IndentLevel, T: parse::Parse>(input: parse::ParseStream) -> Result<Vec<T>> {
    let content;
    syn::parenthesized!(content in input);
    let args = content.parse_terminated::<T, Token![,]>(|p| p.parse())?;
    Ok(args.into_iter().collect())
}

impl<L: IndentLevel, Set: DispatchParseIndented> ParseIndented<L> for DynStruct<Set> {
    fn parse_indented(input: parse::ParseStream) -> Result<Self> {
        let mut props = DynStruct::<Set>::new();

        parse_at_level_fn::<L>(input, || {
            let p = parse_indented::<L, _>(input)?;
            props.insert_dyn(p);
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
        let PropLabel(span, label) = input.fork().parse()?;
        if !Set::has_prop(label) {
            eprintln!("{:?}", Set::PROPS);
            return Err(parse_error!(input.span(), format!("invalid property: {}", label)));
        }
        let PropLabel(span, label) = input.parse()?;
        input.parse::<Token![:]>()?;
        <Set as DispatchParseIndented>::dispatch::<L>(label, input)
    }
}




struct PropLabel(proc_macro2::Span, &'static str);
impl parse::Parse for PropLabel {
    fn parse(input: parse::ParseStream) -> Result<Self> {
        if input.peek(Token![type]) {
            let t = input.parse::<Token![type]>()?;
            Ok(PropLabel(t.span, "type"))
        } else if input.peek(Token![struct]) {
            let t = input.parse::<Token![struct]>()?;
            Ok(PropLabel(t.span, "struct"))
        } else if input.peek(Token![mut]) {
            let t = input.parse::<Token![mut]>()?;
            Ok(PropLabel(t.span, "mut"))
        } else {
            let path = input.parse::<syn::Path>()?;
            let s = quote::quote!(#path).to_string();
            let s = Box::leak(s.to_owned().into_boxed_str());
            Ok(PropLabel(path.segments[0].span(), s))
        }
    }
}
