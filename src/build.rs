
use std::collections::HashMap;
use syn::*;
use crate::{programs::add_programs_to_context, propsets::DynStruct, types::*};



pub fn build_contexts(YamlDoc(YamlContexts(contexts), accounts): &YamlDoc) -> Result<BuiltContexts> {
    contexts.iter().map(|(id, props)| {
        let props = build_context(props, accounts)?;
        Ok((id.clone(), props))
    }).collect::<Result<_>>().map(BuiltContexts)
}

type ContextDependencies = HashMap<Ident, Vec<AccountArg>>;

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

    add_programs_to_context(&mut accounts);

    Ok(BuiltContext { accounts, instruction })
}


fn build_account(
    account: &DynStruct<ParseAccountProps>,
    args: Vec<AccountArg>,
    dependencies: &mut ContextDependencies
) -> Result<BuiltAccount> {

    let mut out = DynStruct::<RealAccountProps>::new();
    let is_init = args.iter().find(|i| i.0 == "init");
    let mut added_props = DynStruct::<RealAccountPropsSansInit>::new();

    for prop in account.iter().cloned() {
        match_case_RealAccountPropsSansInit!(prop, ParseAccountProps,
            o => { out.insert(o); },

            ParseAccountProps::InitIfNeeded(o) => { if is_init.is_none() { out.insert(o); } },
            ParseAccountProps::ConditionalProps(c) => {
                let y = args.contains(&c.arg);
                added_props.update(if y { c._if.clone() } else { c._else.clone() });
            },

            // TODO: Carry spans
            ParseAccountProps::Init(Init(o)) => {
                if is_init.is_some() { added_props.update(o.1); }
            },
            ParseAccountProps::NoInit(NoInit(o)) => {
                if is_init.is_none() { added_props.update(o.1); }
            },
            ParseAccountProps::Depends(Depends(o)) => {
                o.1.into_iter().for_each(|d| { dependencies.entry(d).or_insert(vec![]); });
            },
        )
    }

    added_props.iter().cloned().for_each(|prop| { out.insert_dyn(prop.into()); });

    if let Some(id) = is_init {
        out.insert(RealInit(LabelledProp(id.0.clone().into(), ())));
    }

    if let Some(p) = args.iter().find(|i| i.0 == "mut") {
        out.insert(Mut(LabelledProp(p.0.clone().into(), ())));
    }

    Ok(BuiltAccount(out))
}

