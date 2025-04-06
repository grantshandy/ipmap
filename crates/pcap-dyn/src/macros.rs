#[macro_export]
macro_rules! impl_dyn_fn {
    (fn $fn_name:ident($($arg_name:ident : $arg_ty:ty),*) $(-> $ret_ty:ty)?) => {
        pub struct $fn_name;

        impl DynCFunc for $fn_name {
            const NAME: &'static str = stringify!($fn_name);
            type Signature = extern "C" fn($($arg_ty),*) $(-> $ret_ty)?;
        }
    };
}

#[macro_export]
macro_rules! generate_bindings {
    (
        $file:expr,
        $struct_name:ident,
        $(
            fn $fn_name:ident($($arg_name:ident : $arg_ty:ty),*) $(-> $ret_ty:ty)?;
        )+
    ) => {
        use libc::*;

        pub(crate) trait DynCFunc {
            const NAME: &'static str;
            type Signature;
        }

        $(
            $crate::impl_dyn_fn!(fn $fn_name($($arg_name : $arg_ty),*) $(-> $ret_ty)?);
        )+

        pub struct $struct_name<'t> {
            lib: libloading::Library,
            $(
                pub $fn_name: libloading::Symbol<'t, <$fn_name as DynCFunc>::Signature>,
            )+
        }

        impl<'t> $struct_name<'t> {
            pub unsafe fn init() -> Result<$struct_name<'t>, libloading::Error> {
                let lib = unsafe { libloading::Library::new($file)? };

                Ok($struct_name {
                    $(
                        $fn_name: unsafe { std::mem::transmute(
                            lib.get::<<$fn_name as DynCFunc>::Signature>(<$fn_name as DynCFunc>::NAME.as_bytes())?
                        ) },
                    )+
                    lib
                })
            }
        }
    };
}
