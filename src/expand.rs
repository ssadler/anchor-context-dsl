
use std::collections::HashSet;

use crate::{propsets::DynStruct, types::*};
use syn::*;



struct ExpandedDoc(pub KeyedVec<DynStruct<RealAccountProps>>);

pub fn expand_doc(YamlDoc(YamlContexts(contexts), accounts): &YamlDoc) -> Result<ExpandedDoc> {
    let r = contexts.iter().map(|(id, props)| {
        let props = expand_context(props, accounts)?;
        Ok((id, props))
    }).collect::<Result<Vec<_>>>()?;
    panic!("")
}

fn expand_context(
    props: &Vec<ContextProp>,
    accounts: &YamlAccounts
) -> Result<Vec<(syn::Ident, DynStruct<RealAccountProps>)>> {
    props.iter().map(|ctx_prop| {
        match ctx_prop {
            ContextProp::Account { name, args } => Ok((name.clone(), expand_account(name, args, accounts)?))
        }
    }).collect::<Result<Vec<(syn::Ident, DynStruct<RealAccountProps>)>>>()
}


fn expand_account(
    id: &syn::Ident,
    args: &Vec<syn::Ident>,
    accounts: &KeyedVec<DynStruct<ParseAccountProps>>
) -> Result<DynStruct<RealAccountProps>> {
    let account = accounts.get(&id).ok_or_else(|| parse_error!(id.span(), "undefined account"))?;

    let mut depends = HashSet::<syn::Ident>::new();
    let init = args.iter().any(|a| a == "init");

    let mut out = DynStruct::<RealAccountProps>::new();

    account.iter().map(|prop| {
        match prop.1 {
            ParseAccountProps::Constraints(constraints) => todo!(),
            ParseAccountProps::Mut(_) => todo!(),
            ParseAccountProps::Struct(_) => todo!(),
            ParseAccountProps::Seeds(seeds) => todo!(),
            ParseAccountProps::TokenMint(token_mint) => todo!(),
            ParseAccountProps::TokenAuthority(token_authority) => todo!(),
            ParseAccountProps::AccountType(account_type) => todo!(),
            ParseAccountProps::Depends(depends) => todo!(),
            ParseAccountProps::Init(init) => todo!(),
            ParseAccountProps::NoInit(no_init) => todo!(),
        }
    });
    

    Ok(out)
}
