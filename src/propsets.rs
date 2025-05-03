
use std::{collections::HashMap, fmt::Debug};

#[derive(Default, Debug, Clone)]
pub struct DynStruct<Set: PropSet>(HashMap<&'static str, Dyn<Set>>, Set) where Dyn<Set>: Clone;
impl<Set: PropSet> DynStruct<Set> where Dyn<Set>: Clone {
    pub fn has(&self, label: &str) -> bool {
        self.0.contains_key(label)
    }
    pub fn get<P: PropOf<Set>>(&self) -> Option<&P::TYPE> {
        self.0.get(P::LABEL).map(|d| {
            d.downcast_ref::<P>()
        }).flatten()
    }
    pub fn insert<P: PropOf<Set>>(&mut self, val: P::TYPE) -> Option<P::TYPE> {
        self.0.insert(P::LABEL, Dyn::new::<P>(val)).map(|r| r.downcast::<P>()).flatten()
    }
    pub fn insert_dyn(&mut self, label: &'static str, val: Dyn<Set>) -> Option<Dyn<Set>> {
        assert!(Set::has_prop(label), "insert_dyn: wrong prop");
        self.0.insert(label, val)
    }
    pub fn iter(&self) -> impl Iterator<Item=(&str, Set::EnumType)> {
        self.0.iter().map(|(&k, v)| {
            (k, Set::to_enum(k, v))
        })
    }
}


#[derive(Debug)]
pub struct Dyn<Set: PropSet>(pub &'static str, *mut (), Set);
impl<Set: PropSet> Dyn<Set> {
    pub fn new<P: PropOf<Set>>(mut prop: P::TYPE) -> Self {
        Dyn(P::LABEL, &mut prop as *mut P::TYPE as *mut (), Set::default())
    }
    pub fn downcast_ref<P: PropOf<Set>>(&self) -> Option<&P::TYPE> {
        (self.0 == P::LABEL).then(|| unsafe { &*(self.1 as *const P::TYPE) })
    }
    pub fn downcast<P: PropOf<Set>>(self) -> Option<P::TYPE> {
        (self.0 == P::LABEL).then(|| {
            let a = unsafe { Box::<P::TYPE>::from_raw(self.1 as *mut P::TYPE) };
            *a
        })
    }
}

pub trait IsDynProp {
    type TYPE: 'static + Clone;
    const LABEL: &'static str;
}

pub struct DynProp<Set: PropSet> {
    pub label: &'static str,
    pub value: Dyn<Set>,
    pub set: Set
}

pub trait PropSet: Default {
    type EnumType;
    const PROPS: &[&str];
    fn has_prop(label: &str) -> bool;
    fn to_enum(label: &str, value: &Dyn<Self>) -> Self::EnumType;
}

pub trait PropOf<Set: PropSet>: IsDynProp {
    fn to_dyn(self) -> DynProp<Set>;
}

macro_rules! define_prop {
    ($struct_name:ident, $label:literal, $type:ty) => {
        pub struct $struct_name(pub $type);
        impl IsDynProp for $struct_name {
            type TYPE = $type;
            const LABEL: &'static str = $label;
        }
    };
}


macro_rules! define_prop_set {
    ($set:ident, $($prop:ident),+) => {
        define_prop_set!(@ $set, $($prop),+ | $);
    };
    (@ $set:ident, $($prop:ident),+ | $D:tt) => {
        #[derive(Default, Debug, Clone, Copy)]
        pub struct $set;
        $(impl PropOf<$set> for $prop {
            fn to_dyn(self) -> DynProp<$set> {
                DynProp { label: $prop::LABEL, value: Dyn::new::<$prop>(self.0), set: $set }
            }
        })*
        paste::paste! {
            #[derive(Debug)]
            pub enum [<$set Item>] {
                $($prop(<$prop as IsDynProp>::TYPE)),*
            }
            impl PropSet for $set {
                type EnumType = [<$set Item>];
                const PROPS: &[&str] = &[$($prop::LABEL),*];
                fn has_prop(label: &str) -> bool {
                    $(label == $prop::LABEL)||+
                }
                fn to_enum(label: &str, value: &Dyn<$set>) -> Self::EnumType {
                    $( if label == $prop::LABEL {
                        [<$set Item>]::$prop(value.downcast_ref::<$prop>().unwrap().clone())
                    } else )+ { panic!("to_enum failed") }
                }
            }
            macro_rules! [<impl_prop_dispatch_ $set>] {
                ($label:ident, |$T:ident| $expr:expr, $fail:expr) => {
                    $( if $label == $prop::LABEL { type $T = $prop; $expr } else )+ { $fail }
                };
            }
            pub(crate) use [<impl_prop_dispatch_ $set>];

            impl Clone for Dyn<$set> {
                fn clone(&self) -> Self {
                    $( if self.0 == $prop::LABEL {
                        return Dyn::new::<$prop>(self.downcast_ref::<$prop>().unwrap().clone())
                    })*
                    panic!("Dyn clone failed")
                }
            }
        }
    };
}




pub(crate) use define_prop;
pub(crate) use define_prop_set;



