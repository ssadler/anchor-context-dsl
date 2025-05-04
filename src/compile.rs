
use crate::{propsets::DynStruct, types::*};
use syn::*;
use quote::*;


pub fn compile(BuiltContexts(contexts): BuiltContexts) -> proc_macro2::TokenStream {
    let r = contexts.into_iter().map(compile_context).collect::<Vec<_>>();
    
    quote! { #(#r)* }
}

fn compile_context((name, ctx): (Ident, BuiltContext)) -> proc_macro2::TokenStream {

    let accounts = ctx.accounts.into_iter().map(compile_account);

    let instruction = ctx.instruction.map(|i| {
        quote! { #[instruction(#(#i),*)] }
    }).unwrap_or_default();

    quote! {
        #[derive(Accounts)]
        #instruction
        struct #name<'info> {
            #(#accounts),*
        }
    }
}

fn compile_account((name, BuiltAccount(mut props)): (Ident, BuiltAccount)) -> proc_macro2::TokenStream {

    let typ = compile_type(&mut props);
    let metas = compile_metas(&mut props);

    let leftover = Vec::from_iter(props.keys());
    if leftover.len() > 0 {
        panic!("leftover tokens: {:?}", leftover);
        //return syn::Error::into_compile_error(parse_error!(name.span(), "leftover tokens"))
    }


    quote! {
        #metas
        pub #name: #typ
    }
}

fn compile_type(props: &mut DynStruct<RealAccountProps>) -> proc_macro2::TokenStream {
    let t = props.remove::<AccountType>();

    let is_zero_copy = props.get::<ZeroCopy>().map(|ZeroCopy(z)| z.value()).unwrap_or(false);

    let mut o = match t {
        None => { panic!("accountType or struct required") },
        Some(AccountType(ty)) => match (ty.segments.len(), ty.segments[0].arguments.is_none())  {
            (1, true) => {
                if is_zero_copy {
                    quote! { AccountLoader<'info, #ty> }
                } else {
                    quote! { Account<'info, #ty> }
                }
            }
            _ => quote! { #ty }
        },
    };

    if props.remove::<Boxed>().map(|ob| ob.value()).unwrap_or(false) {
        o = quote! { Box<#o> };
    }

    eprintln!("{}", o);

    if let Some(Check(s)) = props.remove::<Check>() {
        //o = quote! { #[doc = "CHECK"] #o };
    }

    o
}


fn compile_metas(props: &mut DynStruct<RealAccountProps>) -> proc_macro2::TokenStream {
    let mut metas = Vec::<proc_macro2::TokenStream>::new();

    macro_rules! add_prop {
        ($prop:ident, $label:path) => {
            if let Some($prop(p)) = props.remove::<$prop>() {
                metas.push(quote! { $label = #p });
            }
        };
    }

    let mut init = false;

    if let Some(Seeds(seeds)) = props.remove::<Seeds>() {
        metas.push(quote! { seeds = #seeds, bump });
    }
    if let Some(RealInit(())) = props.remove::<RealInit>() {
        init = true;
        metas.push(quote! { init, payer = payer });
    }
    add_prop!(Space, space);
    if let Some(Mut(p)) = props.remove::<Mut>() {
        if p.value() {
            metas.push(quote! { mut });
        }
    }
    if let Some(ZeroCopy(p)) = props.remove::<ZeroCopy>() {
        if p.value() {
            metas.push(quote! { zero_copy });
        }
    }

    // token::
    add_prop!(TokenMint, token::mint);
    add_prop!(TokenAuthority, token::authority);
    add_prop!(TokenProgram, token::program);

    // associated_token::
    add_prop!(AssociatedTokenMint, associated_token::mint);
    add_prop!(AssociatedTokenAuthority, associated_token::authority);
    add_prop!(AssociatedTokenProgram, associated_token::program);

    // mint::
    add_prop!(MintAuthority, mint::authority);
    add_prop!(MintDecimals, mint::decimals);
    add_prop!(MintTokenProgram, mint::token_program);

    // extensions::
    add_prop!(TransferHookAuthority, extensions::transfer_hook::authority);
    add_prop!(TransferHookProgramId, extensions::transfer_hook::program_id);

    if let Some(InitIfNeeded(b)) = props.remove::<InitIfNeeded>() {
        if b.value() {
            metas.push(quote! { init_if_needed });
        }
    }
    if let Some(Constraints(c)) = props.remove::<Constraints>() {
        c.into_iter().for_each(|c| {
            metas.push(quote! { constraint = #c });
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
