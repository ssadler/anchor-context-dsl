
use crate::{propsets::DynStruct, types::*};
use syn::*;
use quote::*;

type TS2 = proc_macro2::TokenStream;


pub fn compile(BuiltContexts(contexts): BuiltContexts) -> TS2 {
    let r = contexts.into_iter().map(compile_context).collect::<Vec<_>>();
    
    let o = quote! {
        #(#r)*
        pub struct ReadOnly<T>(T);
        impl<T> std::ops::Deref for ReadOnly<T> {
            type Target = T;
            fn deref(&self) -> &T { &self.0 }
        }
        pub trait HasMutWrapper { type W; }
    };
    o
}

fn compile_context((name, ctx): (Ident, BuiltContext)) -> TS2 {

    let instruction = ctx.instruction.iter();
    let compile = ctx.accounts.into_iter().map(compile_account);
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

fn compile_account((name, mut props): (Ident, BuiltAccount)) -> (TS2, TS2) {

    let ro = props.is_ro();
    let typ = compile_type(&mut props);
    let metas = compile_metas(&mut props);

    let leftover = Vec::from_iter(props.keys());
    if leftover.len() > 0 {
        panic!("leftover account props: {:?}", leftover);
    }

    let acct = quote! { #metas pub #name: #typ };
    let wrap = if ro { quote! { ReadOnly<#typ> } } else { typ };
    let wrap = quote! { pub #name: #wrap };
    (acct, wrap)
}

fn compile_type(props: &mut BuiltAccount) -> TS2 {
    let t = props.remove::<AccountType>();

    let is_zero_copy = props.remove::<ZeroCopy>().map(|ZeroCopy(z)| z.value()).unwrap_or(false);

    let mut o = match t {
        None => { panic!("accountType or struct required") },
        Some(AccountType(LabelledProp(_, ty))) => {
            let is_struct = ty.segments.len() == 1 && ty.segments[0].arguments.is_none();
            match (is_struct, is_zero_copy, props.is_token_2022()) {
                (true, false, false) => quote! { Account<'info, #ty> },
                (true, true, _) => quote! { AccountLoader<'info, #ty> },
                (true, _, true) => quote! { InterfaceAccount<'info, #ty> },
                _ => quote! { #ty }
            }
        },
    };

    if props.remove::<Boxed>().map(|ob| ob.value()).unwrap_or(false) {
        o = quote! { Box<#o> };
    }

    //eprintln!("{}", o);

    if let Some(Check(s)) = props.remove::<Check>() {
        //o = quote! { #[doc = "CHECK"] #o };
    }

    o
}


fn compile_metas(props: &mut DynStruct<RealAccountProps>) -> TS2 {
    let mut metas = Vec::<TS2>::new();

    macro_rules! add_prop {
        ($prop:ident) => {
            if let Some($prop(LabelledProp(label, p))) = props.remove::<$prop>() {
                metas.push(quote! { #label = #p });
            }
        };
    }

    if let Some(RealInit(LabelledProp(l, ()))) = props.remove::<RealInit>() {
        metas.push(quote! { #l, payer = payer });
    }
    if let Some(InitIfNeeded(LabelledProp(l, b))) = props.remove::<InitIfNeeded>() {
        if b.value() {
            metas.push(quote! { #l, payer = payer });
        }
    }
    if let Some(Seeds(LabelledProp(l, seeds))) = props.remove::<Seeds>() {
        metas.push(quote! { #l = #seeds, bump });
    }
    add_prop!(Space);
    if let Some(Mut(LabelledProp(l, ()))) = props.remove::<Mut>() {
        metas.push(quote! { #l });
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
        c.into_iter().for_each(|c| {
            metas.push(quote! { #l = #c });
        });
    }

    if metas.is_empty() {
        Default::default()
    } else {
        let o = quote! { #[account( #(#metas),* )] };
        //if init { eprintln!("{}", o); }
        o
    }
}
