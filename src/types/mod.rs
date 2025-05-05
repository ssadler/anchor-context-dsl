
use std::collections::HashMap;
use syn::{spanned::Spanned, *};


#[derive(Debug)]
pub struct YamlDoc(pub YamlContexts, pub YamlAccounts);

#[derive(Debug)]
pub struct YamlContexts(pub KeyedVec<Vec<ContextProp>>);

pub type YamlAccounts = KeyedVec<DynStruct<ParseAccountProps>>;

use crate::propsets::*;



#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AccountArg(pub syn::Ident);
impl std::ops::Deref for AccountArg {
    type Target = syn::Ident;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}



#[derive(Debug, Clone)]
pub struct PropLabel {
    pub label: &'static str,
    pub path: syn::Path
}
impl PropLabel {
    pub fn new_from_string(label: String, span: proc_macro2::Span) -> PropLabel {
        let label = Box::leak(label.to_owned().into_boxed_str());
        PropLabel::from_str(label, span)
    }
    pub fn from_str(label: &'static str, span: proc_macro2::Span) -> PropLabel {
        let id = syn::Ident::new(label, span);
        let path = syn::Path::from(id);
        PropLabel { label, path }
    }
    pub fn from_path(path: syn::Path) -> Self {
        let s = quote::quote!(#path).to_string();
        let label = Box::leak(s.to_owned().into_boxed_str());
        PropLabel { label, path }
    }
    pub fn span(&self) -> proc_macro2::Span {
        self.path.span()
    }
}
impl<T> PartialEq<T> for PropLabel where T: ?Sized + AsRef<str>,
{
    fn eq(&self, other: &T) -> bool {
        self.label == other.as_ref()
    }
}
impl quote::ToTokens for PropLabel {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(self.path.to_token_stream());
    }
}

impl From<proc_macro2::Ident> for PropLabel {
    fn from(value: proc_macro2::Ident) -> Self {
        Self::new_from_string(value.to_string(), value.span())
    }
}




#[derive(Debug, Clone)]
pub struct LabelledProp<T>(pub PropLabel, pub T);
impl<T> std::ops::Deref for LabelledProp<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

macro_rules! define_labelled_prop {
    ($struct:ident, $label:literal, $type:ty) => {
        #[derive(Debug, Clone)]
        pub struct $struct(pub LabelledProp<$type>);
        impl $struct {
            #[allow(dead_code)]
            pub fn unwrap(self) -> LabelledProp<$type> { self.0 }
        }
        impl std::ops::Deref for $struct {
            type Target = LabelledProp<$type>;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        define_prop!($struct, $label);
    };
}

define_labelled_prop!(Space, "space", Expr);
define_labelled_prop!(Boxed, "boxed", LitBool);
define_labelled_prop!(ZeroCopy, "zero_copy", LitBool);
define_labelled_prop!(Check, "check", LitStr);
define_labelled_prop!(Constraints, "constraints", Vec<Expr>);
define_labelled_prop!(AccountType, "type", syn::Path);
define_labelled_prop!(Seeds, "seeds", ExprArray);
define_labelled_prop!(TokenMint, "token :: mint", Expr);
define_labelled_prop!(TokenAuthority, "token :: authority", Expr);
define_labelled_prop!(TokenProgram, "token :: token_program", Expr);
define_labelled_prop!(AssociatedTokenMint, "associated_token :: mint", Expr);
define_labelled_prop!(AssociatedTokenAuthority, "associated_token :: authority", Expr);
define_labelled_prop!(AssociatedTokenProgram, "associated_token :: token_program", Expr);
define_labelled_prop!(MintAuthority, "mint :: authority", Expr);
define_labelled_prop!(MintDecimals, "mint :: decimals", Expr);
define_labelled_prop!(MintTokenProgram, "mint :: token_program", Expr);
define_labelled_prop!(TransferHookAuthority, "extensions :: transfer_hook :: authority", Expr);
define_labelled_prop!(TransferHookProgramId, "extensions :: transfer_hook :: program_id", Expr);
define_labelled_prop!(InitIfNeeded, "init_if_needed", LitBool);


define_prop_set!(
    RealAccountPropsSansInit,
    Boxed, ZeroCopy, Space, Check,
    Constraints, AccountType, Seeds,
    TokenMint, TokenAuthority, TokenProgram,
    AssociatedTokenMint, AssociatedTokenAuthority, AssociatedTokenProgram,
    MintAuthority, MintDecimals, MintTokenProgram,
    TransferHookAuthority, TransferHookProgramId
);

define_labelled_prop!(RealInit, "init", ());

extend_set_RealAccountPropsSansInit!(RealAccountProps, RealInit, InitIfNeeded, Mut);

define_labelled_prop!(Mut, "mut", ());
define_labelled_prop!(Depends, "depends", Vec<Ident>);
define_labelled_prop!(Init, "init", DynStruct<RealAccountPropsSansInit>);
define_labelled_prop!(NoInit, "noinit", DynStruct<RealAccountPropsSansInit>);
define_labelled_prop!(ConditionalProps, "if", AccountConditionalProps<RealAccountPropsSansInit>);

extend_set_RealAccountPropsSansInit!(
    ParseAccountProps,
    Depends, Init, NoInit, InitIfNeeded, ConditionalProps
);


/*
 * if <arg>: ...
 * else:     ...
 */
#[derive(Clone, Debug)]
pub struct AccountConditionalProps<Set: PropSet> {
    pub arg: AccountArg,
    pub _if: DynStruct<Set>,
    pub _else: DynStruct<Set>
}


#[derive(Clone, Debug)]
pub enum ContextProp {
    Instruction { args: Vec<syn::FnArg> },
    Account { name: syn::Ident, args: Vec<AccountArg> }
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
impl std::ops::Deref for BuiltAccount {
    type Target = DynStruct<RealAccountProps>;
    fn deref(&self) -> &Self::Target { &self.0 }
}
impl std::ops::DerefMut for BuiltAccount {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl BuiltAccount {
    pub fn is_token_2022(&self) -> bool {
        let tp2022 = syn::parse_quote! { token_program_2022 };
        self.get().map(|AssociatedTokenProgram(p)| p.1 == tp2022) == Some(true) ||
        self.get().map(|TokenProgram(p)|           p.1 == tp2022) == Some(true) ||
        self.get().map(|MintTokenProgram(p)|       p.1 == tp2022) == Some(true)
    }
    pub fn is_ro(&self) -> bool {
        !self.has::<Mut>() &&
        !self.get().map(|InitIfNeeded(p)| p.1.value()).unwrap_or_default() &&
        !self.has::<RealInit>()
    }
}
