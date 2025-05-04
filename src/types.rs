
use std::collections::HashMap;

use syn::*;

#[derive(Debug)]
pub struct YamlDoc(pub YamlContexts, pub YamlAccounts);

#[derive(Debug)]
pub struct YamlContexts(pub KeyedVec<Vec<ContextProp>>);

pub type YamlAccounts = KeyedVec<DynStruct<ParseAccountProps>>;

use crate::propsets::*;


define_prop!(Space, "space", Expr);
define_prop!(Boxed, "boxed", LitBool);
define_prop!(ZeroCopy, "zero_copy", LitBool);
define_prop!(Mut, "mut", LitBool);
define_prop!(Check, "check", LitStr);
define_prop!(Constraints, "constraints", Vec<Expr>);
define_prop!(AccountType, "type", syn::Path);
define_prop!(Seeds, "seeds", ExprArray);
define_prop!(TokenMint, "token :: mint", Expr);
define_prop!(TokenAuthority, "token :: authority", Expr);
define_prop!(TokenProgram, "token :: token_program", Expr);
define_prop!(AssociatedTokenMint, "associated_token :: mint", Expr);
define_prop!(AssociatedTokenAuthority, "associated_token :: authority", Expr);
define_prop!(AssociatedTokenProgram, "associated_token :: token_program", Expr);
define_prop!(MintAuthority, "mint :: authority", Expr);
define_prop!(MintDecimals, "mint :: decimals", Expr);
define_prop!(MintTokenProgram, "mint :: token_program", Expr);
define_prop!(TransferHookAuthority, "extensions :: transfer_hook :: authority", Expr);
define_prop!(TransferHookProgramId, "extensions :: transfer_hook :: program_id", Expr);
define_prop!(InitIfNeeded, "init_if_needed", LitBool);

define_prop_set!(
    RealAccountPropsSansInit,
    Boxed, ZeroCopy, Space, Check,
    Constraints, Mut, AccountType, Seeds,
    TokenMint, TokenAuthority, TokenProgram,
    AssociatedTokenMint, AssociatedTokenAuthority, AssociatedTokenProgram,
    MintAuthority, MintDecimals, MintTokenProgram,
    TransferHookAuthority, TransferHookProgramId
);

define_prop!(RealInit, "init", ());

extend_set_RealAccountPropsSansInit!(RealAccountProps, RealInit, InitIfNeeded);

define_prop!(Depends, "depends", Vec<Ident>);
define_prop!(Init, "init", DynStruct<RealAccountPropsSansInit>);
define_prop!(NoInit, "noinit", DynStruct<RealAccountPropsSansInit>);

extend_set_RealAccountPropsSansInit!(ParseAccountProps, Depends, Init, NoInit, InitIfNeeded);



#[derive(Clone, Debug)]
pub enum ContextProp {
    Instruction { args: Vec<syn::FnArg> },
    Account { name: syn::Ident, args: Vec<syn::Ident> }
}


#[derive(Debug, Default)]
pub struct KeyedVec<T>(pub Vec<(Ident, T)>);
impl<T> KeyedVec<T> {
    pub fn new() -> Self { KeyedVec(vec![]) }
    pub fn has(&self, id: &Ident) -> bool {
        self.iter().any(|(i,_)| i == id)
    }
    pub fn get(&self, id: &Ident) -> Option<&T> {
        self.iter().find_map(|(i, t)| (id == i).then_some(t))
    }
    pub fn insert(&mut self, id: Ident, val: T) -> Result<()> {
        if self.has(&id) { Err(parse_error!(id.span(), format!("KeyedVec insert already exists: {}", id))) }
        else { Ok(self.0.push((id, val))) }
    }
    pub fn iter(&self) -> impl Iterator<Item=&(Ident, T)> {
        self.0.iter()
    }
}
impl<T> FromIterator<(Ident, T)> for KeyedVec<T> {
    fn from_iter<I: IntoIterator<Item = (Ident, T)>>(iter: I) -> Self {
        KeyedVec(Vec::from_iter(iter))
    }
}
impl<T> IntoIterator for KeyedVec<T> {
    type Item = (Ident, T);
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}


macro_rules! parse_error {
    ($span:expr, $msg:expr) => {
        syn::Error::new($span, $msg)
    };
}
pub(crate) use parse_error;




#[derive(Debug)]
pub struct BuiltContexts(pub KeyedVec<BuiltContext>);
#[derive(Debug)]
pub struct BuiltContext {
    pub accounts: HashMap<Ident, BuiltAccount>,
    pub instruction: Option<Vec<FnArg>>
}
#[derive(Debug)]
pub struct BuiltAccount(pub DynStruct<RealAccountProps>);
