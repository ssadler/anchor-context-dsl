use std::collections::HashMap;

use crate::{propsets::DynStruct, types::*};
use quote::quote as q;
use quote::*;
use syn::*;

type TS2 = proc_macro2::TokenStream;

pub fn compile(BuiltContexts(contexts): BuiltContexts) -> TS2 {
    let mut constants = HashMap::<String, Item>::new();

    let r = contexts
        .into_iter()
        .map(|c| { compile_context(&mut constants, c) })
        .collect::<Vec<_>>();
    let wrapper_types = compile_wrapper_types();

    let mut constants = constants.into_iter().collect::<Vec<_>>();
    constants.sort_by_key(|(k,_)| k.clone());
    let constants = constants.into_iter().map(|(_,v)| v);

    quote! {
        #(#constants)*
        #(#r)*
        #(#wrapper_types)*
    }
}

pub fn compile_wrapper_types() -> Vec<TS2> {
    vec![
        q! { pub struct ReadOnly<T>(T); },
        q! {
            impl<T> std::ops::Deref for ReadOnly<T> {
                type Target = T;
                fn deref(&self) -> &T { &self.0 }
            }
        },
        q! { pub trait HasMutWrapper { type W; } },
    ]
}

fn compile_context(constants: &mut HashMap<String, Item>, (name, ctx): (Ident, BuiltContext)) -> TS2 {
    let instruction = ctx.instruction.iter();

    let mut accounts = ctx.accounts.clone().into_iter().collect::<Vec<_>>();
    accounts.sort_by_key(|(i, a)| (a.get::<AccountType>().map(|a| { format!("{:?}", a.1) }), i.clone()));

    extract_account_constants(constants, &mut accounts);

    let compile = accounts.into_iter().map(compile_account);
    let (accounts, wrapped): (Vec<TS2>, Vec<TS2>) = compile.unzip();
    let name_wrap = format_ident!("{}Wrap", name);

    quote! {
        #[derive(Accounts)]
        #( #[instruction( #(#instruction),* )] )*
        pub struct #name<'info> { #(#accounts),* }
        pub struct #name_wrap<'info> { #(#wrapped),* }
        impl<'info> HasMutWrapper for #name<'info> { type W = #name_wrap<'info>; }
        const _: () = assert!(
            std::mem::size_of::<#name>() == std::mem::size_of::<#name_wrap>()
        );
    }
}

fn extract_account_constants(
    constants: &mut HashMap<String, Item>,
    accounts: &mut Vec<(Ident, BuiltAccount)>
) {
    // extract seed constants
    accounts.iter_mut().for_each(|(_, a)| {
        if let Some(seeds) = a.get_mut::<Seeds>() {
            seeds.1.elems.iter_mut().enumerate().for_each(|(idx, expr)| {
                match expr {
                    Expr::Lit(ExprLit { lit: Lit::ByteStr(lit), .. }) => {
                        match String::from_utf8(lit.value()) {
                            Ok(s) => {
                                let s = format!("KEY_{}_{}", s.to_uppercase().replace("-", "_"), idx);
                                let c = Ident::new(s.as_str(), lit.span());
                                constants.insert(s, syn::parse_quote!(pub const #c: &[u8] = #lit;));
                                Some(syn::parse_quote!(#c))
                            },
                            _ => None
                        }
                    },
                    _ => None
                }
                .map(|e| *expr = e);
            });
        }
    });
}

pub fn compile_context_wrapper(
    (name, accounts): (Ident, HashMap<Ident, BuiltAccount>),
) -> Vec<TS2> {
    let compile = accounts.into_iter().map(compile_account);
    let (_, wrapped): (Vec<TS2>, Vec<TS2>) = compile.unzip();
    let name_wrap = format_ident!("{}Wrap", name);

    vec![
        quote! { pub struct #name_wrap<'info> { #(#wrapped),* } },
        quote! { impl<'info> HasMutWrapper for #name<'info> { type W = #name_wrap<'info>; } },
        quote! {
            const _: () = assert!(
                std::mem::size_of::<#name>() == std::mem::size_of::<#name_wrap>()
            );
        },
    ]
}

pub fn compile_account((name, mut props): (Ident, BuiltAccount)) -> (TS2, TS2) {
    let ro = props.is_ro();
    let mut typ = compile_type(&mut props);
    let metas = compile_metas(&mut props);
    let check = props.remove::<Check>();

    let leftover = Vec::from_iter(props.keys());
    if leftover.len() > 0 {
        panic!("leftover account props: {:?}", leftover);
    }

    let check = check.map(|Check(LabelledProp(_, s))| {
        let s = format!("CHECK {}", s.value());
        q!(#[doc = #s])
    }).into_iter();
    let acct = quote! { #metas #(#check)* pub #name: #typ };
    if ro {
        typ = q!(ReadOnly<#typ>)
    }
    (acct, q!(pub #name: #typ))
}

pub fn compile_type(props: &mut BuiltAccount) -> TS2 {
    let t = props.remove::<AccountType>();

    let is_zero_copy = props
        .remove::<ZeroCopy>()
        .map(|ZeroCopy(z)| z.value())
        .unwrap_or(false);

    let mut o = match t {
        None => {
            eprintln!("{:?}", props);
            panic!("accountType or struct required")
        }
        Some(AccountType(LabelledProp(_, ty))) => {
            let is_struct = ty.segments.len() == 1 && ty.segments[0].arguments.is_none();
            match (is_struct, is_zero_copy, props.is_token_2022()) {
                (true, false, false) => quote! { Account<'info, #ty> },
                (true, true, _) => quote! { AccountLoader<'info, #ty> },
                (true, _, true) => quote! { InterfaceAccount<'info, #ty> },
                _ => quote! { #ty },
            }
        }
    };

    if props
        .remove::<Boxed>()
        .map(|ob| ob.value())
        .unwrap_or(false)
    {
        o = quote! { Box<#o> };
    }

    o
}

pub fn compile_metas(props: &mut DynStruct<RealAccountProps>) -> Option<TS2> {
    let mut metas = Vec::<TS2>::new();

    macro_rules! qa { ($($t:tt)*) => { metas.push(quote!($($t)*)); }; }

    macro_rules! add_prop {
        ($prop:ident) => {
            if let Some($prop(LabelledProp(label, p))) = props.remove::<$prop>() {
                qa!(#label = #p);
            }
        };
    }

    if let Some(RealInit(LabelledProp(l, ()))) = props.remove::<RealInit>() {
        qa!(#l);
        qa!(payer = payer);
    }
    if let Some(InitIfNeeded(LabelledProp(l, _))) = props.remove::<InitIfNeeded>() {
        qa!(#l);
        qa!(payer = payer);
    }
    if let Some(Seeds(LabelledProp(l, seeds))) = props.remove::<Seeds>() {
        qa!(#l = #seeds);
        qa!(bump);
    }
    add_prop!(Space);
    if let Some(Mut(LabelledProp(l, ()))) = props.remove::<Mut>() {
        qa!(#l);
    }

    // token::
    add_prop!(TokenMint);
    add_prop!(TokenAuthority);
    add_prop!(TokenProgram);

    // associated_token::
    add_prop!(AssociatedTokenMint);
    add_prop!(AssociatedTokenAuthority);
    add_prop!(AssociatedTokenProgram);

    // mint::
    add_prop!(MintAuthority);
    add_prop!(MintDecimals);
    add_prop!(MintTokenProgram);

    // extensions::
    add_prop!(TransferHookAuthority);
    add_prop!(TransferHookProgramId);

    if let Some(Constraints(LabelledProp(l, c))) = props.remove::<Constraints>() {
        let l = quote::quote_spanned! { l.span() => constraint };
        c.into_iter().for_each(|c| { qa!(#l = #c); });
    }

    if metas.is_empty() {
        None
    } else {
        let s = metas
            .into_iter()
            .map(|m| format!("        {}", m))
            .map(|s| s.replace("\n", " "))
            .collect::<Vec<_>>()
            .join(",\n");

        let s = format!("#[account(\n{}\n    )]", s);
        Some(quote!(#s))
    }
}
