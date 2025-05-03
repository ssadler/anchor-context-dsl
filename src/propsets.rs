
use std::{collections::HashMap, fmt::Debug};

#[derive(Default, Debug, Clone)]
pub struct DynStruct<Set: PropSet>(HashMap<&'static str, DynProp<Set>>, Set) where DynProp<Set>: Clone;
impl<Set: PropSet> DynStruct<Set> where DynProp<Set>: Clone {
    pub fn has(&self, label: &str) -> bool {
        self.0.contains_key(label)
    }
    pub fn get<P: PropOf<Set>>(&self) -> Option<&P> {
        self.0.get(P::LABEL).map(|d| {
            d.downcast_ref::<P>()
        }).flatten()
    }
    pub fn insert<P: PropOf<Set>>(&mut self, val: P) -> Option<P> {
        self.0.insert(P::LABEL, DynProp::new::<P>(val)).map(|r| r.downcast::<P>()).flatten()
    }
    pub fn insert_dyn(&mut self, prop: DynProp<Set>) -> Option<DynProp<Set>> {
        self.0.insert(prop.label, prop)
    }
    pub fn iter(&self) -> impl Iterator<Item=(&str, Set::EnumType)> {
        self.0.clone().into_iter().map(|(k, v)| { (k, Set::to_enum(k, v)) })
    }
}


#[derive(Debug)]
pub struct DynProp<Set: PropSet> {
    pub label: &'static str,
    pub ptr: *mut (),
    pub set: Set
}
impl<Set: PropSet> DynProp<Set> {
    pub fn new<P: PropOf<Set>>(mut prop: P) -> Self {
        let ptr = &mut prop as *mut P as *mut ();
        DynProp { label: P::LABEL, ptr, set: Set::default() }
    }
    pub fn downcast_ref<P: PropOf<Set>>(&self) -> Option<&P> {
        (self.label == P::LABEL).then(|| unsafe { &*(self.ptr as *const P) })
    }
    pub fn downcast<P: PropOf<Set>>(self) -> Option<P> {
        (self.label == P::LABEL).then(|| {
            * unsafe { Box::<P>::from_raw(self.ptr as *mut P) }
        })
    }
    pub fn into<B: PropSet>(self) -> DynProp<B> {
        panic!("")
    }
}

pub trait IsDynProp {
    type TYPE: 'static + Clone;
    const LABEL: &'static str;
}


pub trait PropSet: Default {
    type EnumType;
    const PROPS: &[&str];
    fn has_prop(label: &str) -> bool;
    fn to_enum(label: &str, value: DynProp<Self>) -> Self::EnumType;
}

pub trait PropOf<Set: PropSet>: IsDynProp {
    fn to_dyn(self) -> DynProp<Set>;
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
        #[derive(Default, Debug, Clone, Copy)]
        pub struct $set;
        $(impl PropOf<$set> for $prop {
            fn to_dyn(self) -> DynProp<$set> {
                DynProp::new::<Self>(self)
            }
        })*
        paste::paste! {
            #[derive(Debug)]
            pub enum [<$set Item>] {
                $($prop($prop)),*
            }
            impl [<$set Item>] {
                pub fn to_dyn(self) -> DynProp<$set> {
                    match self {
                        $([<$set Item>]::$prop(o) => DynProp::<$set>::new::<$prop>(o)),*
                    }
                }
            }

            impl PropSet for $set {
                type EnumType = [<$set Item>];
                const PROPS: &[&str] = &[$($prop::LABEL),*];
                fn has_prop(label: &str) -> bool {
                    $(label == $prop::LABEL)||+
                }
                fn to_enum(label: &str, value: DynProp<$set>) -> Self::EnumType {
                    $( if label == $prop::LABEL {
                        [<$set Item>]::$prop(value.downcast::<$prop>().unwrap())
                    } else )+ { panic!("to_enum failed") }
                }
            }
            macro_rules! [<impl_prop_dispatch_ $set>] {
                ($label:ident, |$T:ident| $expr:expr, $fail:expr) => {
                    $( if $label == $prop::LABEL { type $T = $prop; $expr } else )+ { $fail }
                };
            }
            pub(crate) use [<impl_prop_dispatch_ $set>];

            impl Clone for DynProp<$set> {
                fn clone(&self) -> Self {
                    $( if self.label == $prop::LABEL {
                        let a: $prop = self.downcast_ref::<$prop>().unwrap().clone();
                        return DynProp::new::<$prop>(a)
                    })*
                    panic!("Dyn clone failed")
                }
            }
        }
    };
}




pub(crate) use define_prop;
pub(crate) use define_prop_set;



