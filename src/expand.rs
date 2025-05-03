
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

    let props = account.iter().cloned().map(|prop| {
        match prop {
            AccountProp::Depends(ids) => {
                depends.extend(ids.clone());
                None
            },
            AccountProp::Type(s) => Some(ExpandedAccountProp::Type(s)),
            AccountProp::Mut(m) => Some(ExpandedAccountProp::Mut(m)),
            AccountProp::Struct(s) => Some(ExpandedAccountProp::Struct(s)),
            AccountProp::Constraints(c) => Some(ExpandedAccountProp::Constraints(c)),
            _ => panic!("")
        }
    });
    

    Ok(vec![])
}
