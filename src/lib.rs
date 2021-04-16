use proc_macro::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::GenericParam;
use syn::Token;
use syn::TypeParam;
use syn::Visibility;
use syn::{parse_macro_input, DeriveInput};


/// A macro that generates structs for limiting "users"
///
/// SOUNDNESS: The struct this is attached to MUST NOT have any impls with functions that return `Self`, those functions MUST BE replaced with functions that return the Unbound struct generated by this macro.
/// This also includes traits like `Default`, which become impossible to implement soundly, this means the struct this is applied on MUST NOT derive or impl Default or other traits that have functions that return Self.

#[proc_macro_attribute]
pub fn one_user(attr: TokenStream, input: TokenStream) -> TokenStream {
    let sinput = input.to_string();
    //println!("{:?}", input);

    let input = parse_macro_input!(input as DeriveInput);

    {
        let d = if let syn::Data::Struct(d) = input.data {
            d
        } else {
            panic!("Macro only works on structs at the moment!");
        };
        let d = d.fields;

        assert!(
            d.iter().any(|field| match field.vis {
                Visibility::Public(_) => false,
                _ => true,
            }),
            "Struct must have at least one private field!"
        );
    }

    let name = input.ident;
    let where_clause_preds = input.generics.where_clause.map(|x|x.predicates);


    let generics_defs = {
        let mut generics_defs = input.generics.params;
        if !generics_defs.is_empty() {
            generics_defs.push_punct(syn::token::Comma::default());
        }
        generics_defs
    };


    let generics: Punctuated<GenericParam, Token![,]> = {
        let mut generics = generics_defs
            .clone()
            .into_iter()
            .map(|x| {
                match x {
                    GenericParam::Const(val) => GenericParam::Type(TypeParam::from(val.ident)), // Hack to extract ident of const generics for use in the code
                    _ => x,
                }
            })
            .collect::<Punctuated<GenericParam, _>>();
        if !generics.is_empty() {
            generics.push_punct(syn::token::Comma::default());
        }
        generics
    };

    //let where_clause = input.
    // Names for stuff    
    let mod_name = format_ident!("{}_binder", name.to_string().to_lowercase());

    let bouncer_name = format_ident!("{}Bouncer", name.to_string());
    let unbound_name = format_ident!("Unbound{}", name.to_string());
    let bound_name = format_ident!("Bound{}", name.to_string());
    let mut_bound_name = format_ident!("MutBound{}", name.to_string());
    
    // Prelude
    let (num_slots, pub_defs): (usize, _) = if attr.is_empty() {
        (
            1,
            quote! {
                pub type #bouncer_name = #mod_name::BOUNCER<0>;
                pub type #unbound_name<#generics_defs> = #mod_name::Unbound<#generics>;
                pub type #bound_name<'bound_lifetime, #generics_defs> = #mod_name::Bound<'bound_lifetime, #generics 0>;
                pub type #mut_bound_name<'bound_lifetime, #generics_defs> = #mod_name::MutBound<'bound_lifetime, #generics 0>;
            },
        )
    } else {
        (
            attr.to_string()
                .parse()
                .expect("Expecting either no args or a single numerical arg, the number of slots!"),
            quote! {
                  pub type #bouncer_name<const SLOT: usize> = #mod_name::BOUNCER<SLOT>;
                  pub type #unbound_name<#generics_defs> = #mod_name::Unbound<#generics>;
                  pub type #bound_name<'bound_lifetime, #generics_defs const SLOT: usize> = #mod_name::Bound<'bound_lifetime, #generics SLOT>;
                  pub type #mut_bound_name<'bound_lifetime, #generics_defs const SLOT: usize> = #mod_name::MutBound<'bound_lifetime, #generics SLOT>;
            },
        )
    };

    // Main bulk of code here
    let out = quote! {
        mod #mod_name {
            use super::*;
            const NSLOTS: usize = #num_slots;
            type Usable<#generics_defs> = super::#name<#generics>;

            pub trait OnBind {
                fn on_bind<const SLOT: usize>(&self);
            }

            use bitvec::prelude::*;
            use std::{
                ops::{Deref, DerefMut},
                sync::{atomic::AtomicUsize, Mutex},
            };

            lazy_static! {
                static ref BOUNCER_GUARD: Mutex<BitArr!(for NSLOTS, in Msb0, u8)> = Mutex::new(BitArray::zeroed()); // BOUNCER_GUARD is private, this is important because we don't want somebody take()-ing the intialised OnceCell, leaving it uninitialised, and being able to call new() again on BOUNCER again and have two BOUNCERs
                /// SAFTEY: LAST_BOUND is unreliable, don't rely on it for correctness
                pub static ref LAST_SLOT: AtomicUsize = AtomicUsize::new(0);
            }

            pub struct BOUNCER<const SLOT: usize>(()); // NOTE: () is private, this is important so that the only way to get a BOUNCER instance is to use new()

            impl<const SLOT: usize> BOUNCER<SLOT> {
                /// IMPORTANT: Only one bouncer can exist ever
                /// SAFETY: We are the only ones who can access BOUNCER_GUARD because it is private and we can use that to make sure that we only create one BOUNCER ever
                #[inline]
                pub fn new() -> Self {
                    if SLOT >= NSLOTS {
                        panic!("Bouncer slot should be available, it was not!");
                    }
                    let mut lck = BOUNCER_GUARD.try_lock().expect("Acquring lock to create bouncer!");
                    if lck.get(SLOT).expect("Bouncer slot should be available, it was not!") == false {
                        lck.set(SLOT, true);
                        BOUNCER(())
                    } else {
                        panic!("Bouncer already created!");
                    }
                }
            }

            // Because there only ever exists one bouncer a &mut to a BOUNCER must be unique, so thre can only ever exist one Bound
            pub struct MutBound<'bound_lifetime, #generics_defs const SLOT: usize>(&'bound_lifetime mut Usable<#generics>, &'bound_lifetime mut BOUNCER<SLOT>) where #where_clause_preds;

            impl<#generics_defs const SLOT: usize> Deref for MutBound<'_, #generics SLOT> 
            where #where_clause_preds {
                type Target = Usable<#generics>;

                #[inline]
                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }

            impl<#generics_defs const SLOT: usize> DerefMut for MutBound<'_, #generics SLOT> 
            where #where_clause_preds {
                #[inline]
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.0
                }
            }

            pub struct Bound<'bound_lifetime, #generics_defs const SLOT: usize>(&'bound_lifetime Usable<#generics>, &'bound_lifetime mut BOUNCER<SLOT>) where #where_clause_preds;

            impl<#generics_defs const SLOT: usize> Deref for Bound<'_, #generics SLOT>
            where #where_clause_preds {
                type Target = Usable<#generics>;

                #[inline]
                fn deref(&self) -> &Self::Target { 
                    &self.0
                }
            }

            pub struct Unbound<#generics_defs>(Usable<#generics>)
            where
                Usable<#generics>: OnBind, // Usable is private, this is important because it means to get a Usable you must go through bind which goes through a Bound which requires a &mut BOUNCER, whichs is unique, so no matter how many Unbound there are, there will only ever be one Bound at a time
                #where_clause_preds; 

            impl<#generics_defs> Unbound<#generics> 
            where #where_clause_preds {
                #[inline]
                pub fn from(val: Usable<#generics>) -> Unbound<#generics> {
                    Unbound(val)
                } // Takes a Usable and makes it an Unbound, this is fine since Usable can control how it's constructed and return an Unbound(Usable) instead of a Usable so there is no way a normal user can make a Usable without it being Unbound
                #[inline]
                pub fn bind_mut<'bound_lifetime, const SLOT: usize>(&'bound_lifetime mut self, bn: &'bound_lifetime mut BOUNCER<SLOT>) -> MutBound<'bound_lifetime, #generics SLOT> {
                    self.0.on_bind::<SLOT>();
                    LAST_SLOT.store(SLOT, core::sync::atomic::Ordering::Relaxed);
                    MutBound(&mut self.0, bn)
                }

                #[inline]
                pub fn bind<'bound_lifetime, const SLOT: usize>(&'bound_lifetime self, bn: &'bound_lifetime mut BOUNCER<SLOT>) -> Bound<'bound_lifetime, #generics SLOT> {
                    self.0.on_bind::<SLOT>();
                    LAST_SLOT.store(SLOT, core::sync::atomic::Ordering::Relaxed);
                    Bound(&self.0, bn)
                }
            }

        }
    };

    // println!("{}", out.to_string());

    let out = format!("{}\n{}\n{}", pub_defs.to_string(), out.to_string(), sinput);

    return out.parse().expect("Generated valid tokens!");
}
