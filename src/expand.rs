
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

    type It = ParseAccountPropsItem;

    let mut out: DynStruct<RealAccountProps> = Default::default();

    account.iter().map(|prop| {
        match prop.1 {
            ParseAccountPropsItem::Constraints(constraints) => todo!(),
            ParseAccountPropsItem::Mut(_) => todo!(),
            ParseAccountPropsItem::Struct(_) => todo!(),
            ParseAccountPropsItem::Seeds(seeds) => todo!(),
            ParseAccountPropsItem::TokenMint(token_mint) => todo!(),
            ParseAccountPropsItem::TokenAuthority(token_authority) => todo!(),
            ParseAccountPropsItem::AccountType(account_type) => todo!(),
            ParseAccountPropsItem::Depends(depends) => todo!(),
            ParseAccountPropsItem::Init(init) => todo!(),
            ParseAccountPropsItem::NoInit(no_init) => todo!(),
        }
    });
    

    Ok(Default::default())
}
