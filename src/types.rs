
use syn::*;

#[derive(Debug)]
pub struct YamlDoc(pub YamlContexts, pub YamlAccounts);

#[derive(Debug)]
pub struct YamlContexts(pub KeyedVec<Vec<ContextProp>>);

pub type YamlAccounts = KeyedVec<DynStruct<ParseAccountProps>>;

use crate::propsets::*;


define_prop!(Mut, "mut", LitBool);
define_prop!(Constraints, "constraints", Vec<LitStr>);
define_prop!(AccountType, "type", LitStr);
define_prop!(Struct, "struct", Ident);
define_prop!(Seeds, "seeds", LitStr);
define_prop!(TokenMint, "token_mint", Ident);
define_prop!(TokenAuthority, "token_authority", Ident);
define_prop!(RealInit, "init", ());

define_prop_set!(
    RealAccountProps,
    Constraints, Mut, AccountType, Struct, Seeds, TokenMint, TokenAuthority, RealInit
);
define_prop_set!(
    RealAccountPropsSansInit,
    Constraints, Mut, AccountType, Struct, Seeds, TokenMint, TokenAuthority
);



define_prop!(Depends, "depends", Vec<Ident>);
define_prop!(Init, "init", DynStruct<RealAccountPropsSansInit>);
define_prop!(NoInit, "noinit", DynStruct<RealAccountPropsSansInit>);

define_prop_set!(
    ParseAccountProps,
    Constraints, Mut, Struct, Seeds, TokenMint, TokenAuthority,
    AccountType, Depends, Init, NoInit
);



#[derive(Debug)]
pub enum ContextProp {
    Account { name: syn::Ident, args: Vec<syn::Ident> }
}


#[derive(Debug, Default)]
pub struct KeyedVec<T>(pub Vec<(Ident, T)>);
impl<T> KeyedVec<T> {
    pub fn new() -> Self { KeyedVec(vec![]) }
    pub fn has(&self, id: &Ident) -> bool {
        self.0.iter().any(|(i,_)| i == id)
    }
    pub fn get(&self, id: &Ident) -> Option<&T> {
        self.iter().find_map(|(i, t)| (id == i).then_some(t))
    }
}
impl<T> std::ops::Deref for KeyedVec<T> {
    type Target = Vec<(Ident, T)>;
    fn deref(&self) -> &Self::Target { &self.0 }
}
impl<T> std::ops::DerefMut for KeyedVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}


macro_rules! parse_error {
    ($span:expr, $msg:literal) => {
        syn::Error::new($span, $msg)
    };
}
pub(crate) use parse_error;



