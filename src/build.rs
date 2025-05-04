
use std::collections::HashMap;
use syn::*;
use crate::{propsets::DynStruct, types::*};



pub fn build_contexts(YamlDoc(YamlContexts(contexts), accounts): &YamlDoc) -> Result<BuiltContexts> {
    contexts.iter().map(|(id, props)| {
        let props = build_context(props, accounts)?;
        Ok((id.clone(), props))
    }).collect::<Result<_>>().map(BuiltContexts)
}

type ContextDependencies = HashMap<Ident, Vec<Ident>>;

fn build_context(
    props: &Vec<ContextProp>,
    account_defs: &YamlAccounts
) -> Result<BuiltContext> {
    let mut instruction = None;
    let mut depends = props.iter().cloned().filter_map(|ctx_prop| {
        match ctx_prop {
            ContextProp::Account { name, args } => Some((name, args)),
            ContextProp::Instruction { args } => {
                instruction = Some(args);
                None
            }
        }
    }).collect::<ContextDependencies>();

    let mut accounts = HashMap::new();

    loop {
        let r = depends.clone().into_iter().find(|(k, _)| !accounts.contains_key(k));

        if let Some((id, args)) = r {
            let account = account_defs.get(&id).ok_or_else(|| parse_error!(id.span(), "undefined account"))?;
            accounts.insert(id.clone(), build_account(account, args, &mut depends)?);
            continue;
        }
        break;
    };

    Ok(BuiltContext { accounts, instruction })
}


fn build_account(
    account: &DynStruct<ParseAccountProps>,
    args: Vec<syn::Ident>,
    dependencies: &mut ContextDependencies
) -> Result<BuiltAccount> {

    let mut out = DynStruct::<RealAccountProps>::new();
    let is_init = args.iter().any(|i| i == "init");
    let mut init_props = DynStruct::<RealAccountPropsSansInit>::new();

    account.iter().for_each(|prop| {
        match_case_RealAccountPropsSansInit!(prop.1, ParseAccountProps,
            oo => { out.insert(oo); },
            ParseAccountProps::InitIfNeeded(o) => { out.insert(o); },
            ParseAccountProps::Init(o) => { if is_init { init_props = o.unwrap(); } },
            ParseAccountProps::NoInit(o) => { if !is_init { init_props = o.unwrap(); } },
            ParseAccountProps::Depends(o) => {
                o.unwrap().into_iter().for_each(|d| { dependencies.entry(d).or_insert(vec![]); });
            },
        )
    });

    init_props.iter().for_each(|prop| { out.insert_dyn(prop.1.into()); });

    if is_init {
        out.insert(RealInit(()));
    }

    Ok(BuiltAccount(out))
}
