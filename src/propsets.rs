use std::{collections::HashMap, fmt::Debug};

#[derive(Debug, Clone)]
pub struct DynStruct<Set: PropSet>(HashMap<&'static str, Set>);
impl<Set: PropSet> DynStruct<Set> {
    pub fn new() -> Self {
        DynStruct(HashMap::new())
    }
    pub fn has<P: PropOf<Set>>(&self) -> bool {
        self.0.contains_key(P::LABEL)
    }
    pub fn get<P: PropOf<Set>>(&self) -> Option<&P> {
        self.0.get(P::LABEL).map(P::downcast_ref)
    }
    pub fn get_mut<P: PropOf<Set>>(&mut self) -> Option<&mut P> {
        self.0.get_mut(P::LABEL).map(P::downcast_ref_mut)
    }
    pub fn insert<P: PropOf<Set>>(&mut self, val: P) -> Option<P> {
        self.0.insert(P::LABEL, val.into()).map(P::downcast)
    }
    pub fn insert_dyn(&mut self, prop: Set) -> Option<Set> {
        self.0.insert(prop.label(), prop)
    }
    pub fn iter(&self) -> impl Iterator<Item = &Set> {
        Set::PROPS.iter().map(|s| self.0.get(*s)).flatten()
    }
    pub fn remove<P: PropOf<Set>>(&mut self) -> Option<P> {
        self.0.remove(P::LABEL).map(P::downcast)
    }
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.0.keys().cloned()
    }
    pub fn update(&mut self, other: DynStruct<Set>) {
        self.0.extend(other.0);
    }
}

pub trait IsProp {
    const OPT: bool;
    const LABEL: &'static str;
}

pub trait PropSet: Clone + Debug {
    const PROPS: &'static [&'static str];
    fn has_prop(label: &str) -> bool;
    fn label(&self) -> &'static str;
}

pub trait PropOf<Set: PropSet>: IsProp + Into<Set> {
    const INDEX: usize;
    fn downcast(set: Set) -> Self;
    fn downcast_ref(set: &Set) -> &Self;
    fn downcast_ref_mut(set: &mut Set) -> &mut Self;
}

macro_rules! define_prop {
    ($struct_name:ident, $label:literal) => {
        impl IsProp for $struct_name {
            const OPT: bool = false;
            const LABEL: &'static str = $label;
        }
    };
}

macro_rules! define_prop_set {
    ($set:ident, $($prop:ident),+) => {
        define_prop_set!(!set $set, $($prop),+ | $);
    };
    (!set $set:ident, $($prop:ident),+ | $D:tt) => {

        #[derive(Debug, Clone)]
        pub enum $set { $($prop($prop)),* }


        impl PropSet for $set {
            const PROPS: &'static [&'static str] = &[$($prop::LABEL),*];
            fn has_prop(label: &str) -> bool {
                Self::PROPS.iter().any(|&l| l == label)
            }
            fn label(&self) -> &'static str {
                match self { $($set::$prop(_) => $prop::LABEL),* }
            }
        }

        define_prop_set!(!idx 0, $set, $($prop),*);

        $(impl Into<$set> for $prop {
            fn into(self) -> $set { $set::$prop(self) }
        })*


        paste::paste! {
            #[allow(unused_macros)]
            macro_rules! [<prop_dispatch_ $set>] {
                ($label:ident, |$T:ident| $expr:expr, $fail:expr) => {
                    $( if $label == $prop::LABEL { type $T = $prop; $expr } else )+ { $fail }
                };
            }
            #[allow(unused_imports)]
            pub(crate) use [<prop_dispatch_ $set>];

            #[allow(unused_macros)]
            macro_rules! [<match_case_ $set>] {
                ($input:expr, $set2:ident, $o:pat => $rest:tt, $D($arms:tt)*) => {
                    match $input {
                        $($set2::$prop($o) => $rest,)*
                        $D($arms)*
                    }
                }
            }
            #[allow(unused_macros, unused_imports)]
            pub(crate) use [<match_case_ $set>];


            #[allow(unused_macros, unused_imports)]
            macro_rules! [<extend_set_ $set>] {
                ($set2:ident, $D($prop2:ident),+) => {
                    define_prop_set!($set2 $(,$prop)* $D(,$prop2)*);
                    impl Into<$set2> for $set {
                        fn into(self) -> $set2 {
                            match self {
                                $($set::$prop(o) => $set2::$prop(o)),*
                            }
                        }
                    }
                    impl TryInto<$set> for $set2 {
                        type Error = ();
                        fn try_into(self) -> std::result::Result<$set, Self::Error> {
                            match self {
                                $($set2::$prop(o) => Ok($set::$prop(o)),)*
                                _ => Err(())
                            }
                        }
                    }
                };
            }

            #[allow(unused_macros)]
            macro_rules! [<prop_match_ $set>] {
                ($p:ident, |$inner:ident| $expr:expr) => {
                    match $p { $($set::$prop($inner) => $expr),* }
                };
            }
            #[allow(unused_imports)]
            pub(crate) use [<prop_match_ $set>];
        }
    };


    /*
     * Define PropOf recursively to get an incremental INDEX
     */
    (!idx $i:expr, $set:ident, $prop:ident $(,$rest:ident)*) => {
        impl PropOf<$set> for $prop {
            const INDEX: usize = $i;
            fn downcast(set: $set) -> Self {
                match set { $set::$prop(o) => o, _ => panic!("downcast failed") }
            }
            fn downcast_ref(set: &$set) -> &Self {
                match set { $set::$prop(o) => o, _ => panic!("downcast_ref failed") }
            }
            fn downcast_ref_mut(set: &mut $set) -> &mut Self {
                match set { $set::$prop(o) => o, _ => panic!("downcast_ref_mut failed") }
            }
        }
        define_prop_set!(!idx $i+1, $set $(,$rest)*);
    };
    (!idx $i:expr, $set:ident) => {};
}

pub(crate) use define_prop;
pub(crate) use define_prop_set;






