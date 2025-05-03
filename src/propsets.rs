
use std::{collections::HashMap, fmt::Debug};

#[derive(Debug, Clone)]
pub struct DynStruct<Set: PropSet>(HashMap<&'static str, Set>);
impl<Set: PropSet> DynStruct<Set> {
    pub fn new() -> Self { DynStruct(HashMap::new()) }
    pub fn has(&self, label: &str) -> bool {
        self.0.contains_key(label)
    }
    pub fn get<P: PropOf<Set>>(&self) -> Option<&P> {
        self.0.get(P::LABEL).map(P::downcast_ref)
    }
    pub fn insert<P: PropOf<Set>>(&mut self, val: P) -> Option<P> {
        self.0.insert(P::LABEL, val.into()).map(P::downcast)
    }
    pub fn insert_dyn(&mut self, prop: Set) -> Option<Set> {
        self.0.insert(prop.label(), prop)
    }
    pub fn iter(&self) -> impl Iterator<Item=(&str, Set)> {
        self.0.clone().into_iter().map(|(k, v)| { (k, v) })
    }
}


pub trait IsDynProp {
    type TYPE: 'static + Clone;
    const LABEL: &'static str;
}


pub trait PropSet: Clone {
    const PROPS: &[&str];
    fn has_prop(label: &str) -> bool;
    fn label(&self) -> &'static str;
}

pub trait PropOf<Set: PropSet>: IsDynProp + Into<Set> {
    fn downcast(set: Set) -> Self;
    fn downcast_ref(set: &Set) -> &Self;
}

macro_rules! define_prop {
    ($struct_name:ident, $label:literal, $type:ty) => {
        #[derive(Debug, Clone)]
        pub struct $struct_name(pub $type);
        impl IsDynProp for $struct_name {
            type TYPE = $type;
            const LABEL: &'static str = $label;
        }
        impl $struct_name {
            pub fn unwrap(self) -> $type { self.0 }
        }
    };
}


macro_rules! define_prop_set {
    ($set:ident, $($prop:ident),+) => {
        define_prop_set!(@ $set, $($prop),+ | $);
    };
    (@ $set:ident, $($prop:ident),+ | $D:tt) => {
        #[derive(Debug, Clone)]
        pub enum $set { $($prop($prop)),* }
        $(impl PropOf<$set> for $prop {
            fn downcast_ref(set: &$set) -> &Self {
                match set { $set::$prop(o) => o, _ => panic!("downcast_ref failed") }
            }
            fn downcast(set: $set) -> Self {
                match set { $set::$prop(o) => o, _ => panic!("downcast_ref failed") }
            }
        })*

        impl PropSet for $set {
            const PROPS: &[&str] = &[$($prop::LABEL),*];
            fn has_prop(label: &str) -> bool {
                $(label == $prop::LABEL)||+
            }
            fn label(&self) -> &'static str {
                match self { $($set::$prop(_) => $prop::LABEL),* }
            }
        }

        $(impl Into<$set> for $prop {
            fn into(self) -> $set {
                $set::$prop(self)
            }
        })*

        paste::paste! {
            #[allow(unused_macros)]
            macro_rules! [<impl_prop_dispatch_ $set>] {
                ($label:ident, |$T:ident| $expr:expr, $fail:expr) => {
                    $( if $label == $prop::LABEL { type $T = $prop; $expr } else )+ { $fail }
                };
            }
            #[allow(unused_imports)]
            pub(crate) use [<impl_prop_dispatch_ $set>];
        }
    };
}




pub(crate) use define_prop;
pub(crate) use define_prop_set;



