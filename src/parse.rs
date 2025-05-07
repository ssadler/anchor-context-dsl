use crate::indented::*;
use crate::propsets::*;
use crate::types::*;
use syn::*;


type PS<'a> = parse::ParseStream<'a>;


impl parse::Parse for YamlDoc {
    fn parse(input: PS) -> Result<Self> {
        let mut accounts = KeyedVec::<DynStruct<ParseAccountProps>>::new();
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
                input.parse::<Token![:]>()?;

                accounts.insert(name, parse_indented::<Indent<1>, _>(input)?)?;
            }
        }

        Ok(YamlDoc(YamlContexts(contexts), accounts))
    }
}

impl<L: IndentLevel> ParseIndented<L> for ContextProp {
    fn parse_indented(input: PS) -> Result<Self> {
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
                let args = content.parse_terminated::<AccountArg, Token![,]>(|p| p.parse())?;
                args.into_iter().collect()
            } else {
                vec![]
            };

            Ok(ContextProp::Account { name: id, args })
        }
    }
}

impl<I: IndentLevel, Set: DispatchParseIndented> ParseIndented<I> for AccountConditionalProps<Set> {
    fn parse_indented(input: PS) -> Result<Self> {
        //input.parse::<Token![if]>()?;
        let arg = input.parse::<AccountArg>()?;
        input.parse::<Token![:]>()?;
        let _if = parse_indented_next_level::<I, DynStruct<Set>>(input)?;
        let mut _else = DynStruct::new();
        if input.fork().parse::<I>().is_ok() && input.peek2(Token![else]) {
            input.parse::<I>()?;
            input.parse::<Token![else]>()?;
            input.parse::<Token![:]>()?;
            _else = parse_indented_next_level::<I, DynStruct<Set>>(input)?;
        }
        Ok(AccountConditionalProps { arg, _if, _else })
    }
}
impl<I: IndentLevel> ParseIndented<I> for ConditionalProps {
    fn parse_indented(input: PS) -> Result<Self> {
        let t = input.parse()?;
        Ok(ConditionalProps(LabelledProp(
            t,
            parse_indented::<I, _>(input)?,
        )))
    }
}
impl<T> LabelledProp<T> {
    pub fn parse_with(input: PS, f: impl Fn(PS) -> Result<T>) -> Result<Self> {
        let label = input.parse()?;
        input.parse::<Token![:]>()?;
        Ok(LabelledProp(label, f(input)?))
    }
}
impl<T> LabelledProp<Option<T>> {
    pub fn parse_with_opt(input: PS, f: impl Fn(PS) -> Result<T>) -> Result<Self> {
        let label = input.parse()?;
        if input.peek(Token![:]) {
            input.parse::<Token![:]>()?;
            Ok(LabelledProp(label, Some(f(input)?)))
        } else {
            Ok(LabelledProp(label, None))
        }
    }
}

macro_rules! define_prop_parser {
    ($struct_name:ident, $f:ident, $with:ident) => {
        impl<L: IndentLevel> ParseIndented<L> for $struct_name {
            fn parse_indented(input: PS) -> Result<Self> {
                Ok($struct_name(LabelledProp::$with(input, $f::<L, _>)?))
            }
        }
    };
    ($struct_name:ident) => { define_prop_parser!($struct_name, parse_indented_token); };
    ($struct_name:ident, $f:ident) => { define_prop_parser!($struct_name, $f, parse_with); };
}

define_prop_parser!(Space);
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
define_prop_parser!(InitIfNeeded, parse_indented_next_level, parse_with_opt);


fn parse_instruction_args<I: IndentLevel, T: parse::Parse>(
    input: PS,
) -> Result<Vec<T>> {
    let content;
    syn::parenthesized!(content in input);
    let args = content.parse_terminated::<T, Token![,]>(|p| p.parse())?;
    Ok(args.into_iter().collect())
}

impl<L: IndentLevel, Set: DispatchParseIndented> ParseIndented<L> for DynStruct<Set> {
    fn parse_indented(input: PS) -> Result<Self> {
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
    fn dispatch<L: IndentLevel>(label: PropLabel, input: PS) -> Result<Self>;
}
impl DispatchParseIndented for ParseAccountProps {
    fn dispatch<L: IndentLevel>(label: PropLabel, input: PS) -> Result<Self> {
        prop_dispatch_ParseAccountProps!(
            label,
            |T| <T as ParseIndented<L>>::parse_indented(input).map(|s| s.into()),
            Err(parse_error!(label.span(), "invalid property"))
        )
    }
}
impl DispatchParseIndented for RealAccountPropsSansInit {
    fn dispatch<L: IndentLevel>(label: PropLabel, input: PS) -> Result<Self> {
        prop_dispatch_RealAccountPropsSansInit!(
            label,
            |T| <T as ParseIndented<L>>::parse_indented(input).map(|r| r.into()),
            Err(parse_error!(label.span(), "invalid property"))
        )
    }
}

impl<L: IndentLevel, Set: DispatchParseIndented> ParseIndented<L> for Set {
    fn parse_indented(input: PS) -> Result<Self> {
        let label: PropLabel = input.fork().parse()?;
        if !Set::has_prop(label.label) {
            return Err(parse_error!(
                input.span(),
                format!("invalid property: {}", label.label)
            ));
        }
        <Set as DispatchParseIndented>::dispatch::<L>(label, input)
    }
}

impl<I: IndentLevel, T: ParseIndented<I>> ParseIndented<I> for LabelledProp<T> {
    fn parse_indented(input: PS) -> Result<Self> {
        eprintln!("LP: {:?}", input);
        let label = input.parse()?;
        input.parse::<Token![:]>()?;
        Ok(LabelledProp(label, parse_indented(input)?))
    }
}

struct AnyIdent(proc_macro2::Ident);
impl parse::Parse for AnyIdent {
    fn parse(input: PS) -> Result<Self> {
        let (id, _) = input
            .cursor()
            .ident()
            .ok_or(parse_error!(input.span(), "Expected identifier"))?;
        input.step(|c| Ok(((), c.ident().unwrap().1)))?;
        Ok(AnyIdent(id))
    }
}

impl parse::Parse for PropLabel {
    fn parse(input: PS) -> Result<Self> {
        let path = if input.peek2(Token![::]) {
            input.parse::<syn::Path>()?
        } else {
            let AnyIdent(id) = input.parse()?;
            id.into()
        };

        Ok(PropLabel::from_path(path))
    }
}

impl parse::Parse for AccountArg {
    fn parse(input: PS) -> Result<Self> {
        let AnyIdent(id) = input.parse()?;
        Ok(AccountArg(id))
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_wat() {
        let id = syn::Ident::new("type", proc_macro2::Span::call_site());
        println!("{}", id);
    }
}
