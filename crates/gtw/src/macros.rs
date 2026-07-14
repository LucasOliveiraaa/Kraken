pub(crate) trait GatewayWrapper {
    type Raw: ?Sized;
}

/// Converts a gateway-side desc (which may hold live resource wrappers)
/// into the contract-side desc (which holds only Ids), so the raw
/// contract trait can consume it.
pub(crate) trait ToContract {
    type Contract;
    fn to_contract(&self) -> Self::Contract;
}

macro_rules! create_gateway {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $trait:path
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone)]
        $vis struct $name {
            _device: $crate::Device,
            raw: std::sync::Arc<dyn $trait>,
        }

        impl $crate::macros::GatewayWrapper for $name {
            type Raw = dyn $trait;
        }

        impl $name {
            pub fn from_raw(device: $crate::Device, raw: std::sync::Arc<dyn $trait>) -> Self {
                Self { _device: device, raw }
            }

            #[inline]
            pub(crate) fn raw(&self) -> std::sync::Arc<dyn $trait> {
                self.raw.clone()
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($name)).finish()
            }
        }

        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                std::sync::Arc::ptr_eq(&self.raw, &other.raw)
            }
        }

        impl Eq for $name {}

        impl std::hash::Hash for $name {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                std::sync::Arc::as_ptr(&self.raw).hash(state);
            }
        }
    };
}

macro_rules! create_handle_wrapper {
    ($name:ident, $gateway:ty, $handle:ty) => {
        paste::paste! {
            #[derive(Debug, PartialEq, Eq, Hash)]
            #[allow(dead_code)]
            struct [<Inner $name>] {
                gtw: $gateway,
                handle: $handle,
            }
        }

        paste::paste! {
            #[derive(Debug, Clone, PartialEq, Eq, Hash)]
            pub struct $name {
                inner: std::sync::Arc<[<Inner $name>]>,
            }
        }

        impl $name {
            pub fn from_handle(gtw: $gateway, handle: $handle) -> Self {
                paste::paste! {
                    Self {
                        inner: std::sync::Arc::new([<Inner $name>] { gtw, handle }),
                    }
                }
            }

            #[allow(dead_code)]
            pub(crate) fn handle(&self) -> $handle {
                self.inner.handle
            }

            #[allow(dead_code)]
            fn gtw(&self) -> &$gateway {
                &self.inner.gtw
            }

            #[allow(dead_code)]
            fn raw_gtw(&self) -> std::sync::Arc<<$gateway as $crate::macros::GatewayWrapper>::Raw> {
                self.inner.gtw.raw()
            }
        }
    };

    ($name:ident, $gateway:ty, $handle:ty, $destroy:ident) => {
        create_handle_wrapper!($name, $gateway, $handle);

        paste::paste! {
            impl Drop for [<Inner $name>] {
                fn drop(&mut self) {
                    let _ = self.gtw.raw().$destroy(self.handle);
                }
            }
        }
    };
}

// macro_rules! create_handle_methods {
//     (
//         $(
//             fn $name:ident($arg:ident : &$desc_ty:ty) -> $wrapper:ident;
//         )*
//     ) => {
//         $(
//             pub fn $name(&self, $arg: &$desc_ty) -> contract::GpuResult<$wrapper> {
//                 let contract_desc = $arg.to_contract();
//                 let handle = self.raw().$name(&contract_desc)?;
//                 Ok($wrapper::from_handle(self.clone(), handle))
//             }
//         )*
//     };
// }

// macro_rules! create_several_handle_methods {
//     (
//         $(
//             fn $name:ident(
//                 $($arg:ident : $arg_ty:ty),* $(,)?
//             ) -> $wrapper:ident;
//         )*
//     ) => {
//         $(
//             pub fn $name(
//                 &self,
//                 $($arg: $arg_ty),*
//             ) -> contract::GpuResult<Vec<$wrapper>> {
//                 let handles = self.raw().$name($($arg.into()),*)?;
//                 Ok(handles.iter().map(|handle| $wrapper::from_handle(self.clone(), *handle)).collect())
//             }
//         )*
//     };
// }

macro_rules! create_handle_methods {
    (
        $(
            fn $name:ident(
                $( $(#[$argattr:ident])? $arg:ident : $arg_ty:ty ),* $(,)?
                $(; forward: [ $($fwd:ident),* $(,)? ])?
            ) -> $ret:tt;
        )*
    ) => {
        $(
            create_handle_methods!(@method
                $name,
                [ $( $(#[$argattr])? $arg : $arg_ty ),* ],
                [ $( $($fwd),* )? ],
                $ret
            );
        )*
    };

    // ---- single-wrapper return: `-> Wrapper` ----
    (@method $name:ident,
        [ $( $(#[$argattr:ident])? $arg:ident : $arg_ty:ty ),* ],
        [ $($fwd:ident),* ],
        $wrapper:ident
    ) => {
        pub fn $name(&self, $($arg: $arg_ty),*) -> contract::GpuResult<$wrapper> {
            let handle = self.raw().$name(
                $( create_handle_methods!(@raw_arg $(#[$argattr])? $arg) ),*
            )?;
            Ok($wrapper::from_handle(
                self.clone(),
                $( $fwd.clone(), )*
                handle,
            ))
        }
    };

    // ---- Vec<wrapper> return: `-> [Wrapper]` ----
    (@method $name:ident,
        [ $( $(#[$argattr:ident])? $arg:ident : $arg_ty:ty ),* ],
        [ $($fwd:ident),* ],
        [$wrapper:ident]
    ) => {
        pub fn $name(&self, $($arg: $arg_ty),*) -> contract::GpuResult<Vec<$wrapper>> {
            let handles = self.raw().$name(
                $( create_handle_methods!(@raw_arg $(#[$argattr])? $arg) ),*
            )?;
            Ok(handles.iter().map(|handle| {
                $wrapper::from_handle(
                    self.clone(),
                    $( $fwd.clone(), )*
                    *handle,
                )
            }).collect())
        }
    };

    // ---- per-argument conversion for the raw() call ----
    (@raw_arg #[desc]   $arg:ident) => { &$arg.to_contract() };
    (@raw_arg #[wrapper]   $arg:ident) => { $arg.handle() };
    (@raw_arg #[desc_slice] $arg:ident) => {
        &$arg.iter().map(|d| d.to_contract()).collect::<Vec<_>>()
    };
    (@raw_arg #[into]   $arg:ident) => { $arg.into() };
    (@raw_arg #[handle] $arg:ident) => { $arg.handle() };
    (@raw_arg           $arg:ident) => { $arg };
}
