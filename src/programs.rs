
use std::collections::HashMap;

use crate::types::*;
use crate::propsets::*;
use syn::*;



pub fn add_programs_to_context(accounts: &mut HashMap<Ident, BuiltAccount>) {

    let mut sys = false;
    let mut token = false;
    let mut token2022 = false;
    let mut ata = false;

    for (_, acc) in accounts.iter() {

        let init = acc.has::<RealInit>() ||
            acc.get::<InitIfNeeded>().map(|i| i.1.value()).unwrap_or_default();
        sys |= init;

        let is_mut = acc.has::<Mut>();


        if init || is_mut {
            ata |= acc.has::<AssociatedTokenMint>();
            if let Some(AssociatedTokenProgram(p)) = acc.get() {
                token |= p.1 == syn::parse_quote! { token_program };
                token2022 |= p.1 == syn::parse_quote! { token_program_2022 };
            } else {
                token |= ata
            }

            if let Some(TokenProgram(p)) = acc.get() {
                token |= p.1 == syn::parse_quote! { token_program };
                token2022 |= p.1 == syn::parse_quote! { token_program_2022 };
            } else {
                token |= acc.has::<TokenMint>();
            }

            if let Some(MintTokenProgram(p)) = acc.get() {
                token |= p.1 == syn::parse_quote! { token_program };
                token2022 |= p.1 == syn::parse_quote! { token_program_2022 };
            } else {
                token |= acc.has::<MintAuthority>();
            }
        }
    }

    let span = proc_macro2::Span::call_site();

    macro_rules! add_account {
        ($name:literal, $type:path $(,$prop:ident)*) => {
            let p = syn::parse_quote! { $type };
            let label = PropLabel::from_str($name, span);
            let mut account = DynStruct::new();
            account.insert(AccountType(LabelledProp(label, p)));
            $(account.insert($prop);)*
            accounts.entry(proc_macro2::Ident::new($name, span))
                .or_insert(BuiltAccount(account));
        };
    }
    
    if sys {
        add_account!("system_program", Program<'info, System>);
        let _mut = Mut(LabelledProp(PropLabel::from_str("mut", span), ()));
        add_account!("payer", Signer<'info>, _mut);
    }

    if token {
        add_account!("token_program", Program<'info, Token>);
    }

    if token2022 {
        add_account!("token_program_2022", Program<'info, Token2022>);
    }

    if ata {
        add_account!("associated_token_program", Program<'info, AssociatedToken>);
    }
}
