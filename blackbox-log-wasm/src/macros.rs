macro_rules! wasm_export {
    (free $fn:ident : $type:ty) => {
        wasm_export! {
            fn $fn(_x: owned $type) {}
        }
    };

    ($( $(#[$attr:meta])* fn $fn:ident($($arg:ident : $pass:ident $arg_type:ty),* $(,)?) $(-> $return:ty)? $body:block )+) => {$(
        #[no_mangle]
        #[allow(non_snake_case, improper_ctypes_definitions)]
        $(#[$attr])*
        unsafe extern "C" fn $fn($($arg : <$arg_type as crate::WasmFfi>::Ffi),*) $(-> <$return as crate::WasmFfi>::Ffi)? {
            $( #[allow(unused_mut)] let mut $arg = <$arg_type as crate::FromWasmFfi>::from_ffi($arg); )*

            let return_value $(: $return)? = {
                $( wasm_export!(_pass $pass $arg $arg_type); )*
                $body
            };

            $( wasm_export!(_forget $pass $arg); )*

            crate::IntoWasmFfi::into_ffi(return_value)
        }
    )+};

    (_pass owned $arg:ident $type:ty) => {};
    (_pass ref $arg:ident $type:ty) => { let $arg: &$type = &$arg; };
    (_pass ref_mut $arg:ident $type:ty) => { let $arg: &mut $type = &mut $arg; };

    (_forget owned $arg:ident) => {};
    (_forget $ref:ident $arg:ident) => { Box::into_raw($arg) };
}

macro_rules! impl_boxed_wasm_ffi {
    ($t:ty) => {
        impl crate::WasmFfi for Box<$t> {
            type Ffi = *mut $t;
        }

        impl crate::IntoWasmFfi for Box<$t> {
            #[inline(always)]
            fn into_ffi(self) -> Self::Ffi {
                Box::into_raw(self)
            }
        }

        impl crate::FromWasmFfi for Box<$t> {
            #[inline(always)]
            unsafe fn from_ffi(ffi: Self::Ffi) -> Self {
                Box::from_raw(ffi)
            }
        }
    };
}

macro_rules! impl_structural {
    ($(
        #[repr(C)]
        $(#[$attr:meta])*
        $pub:vis struct $name:ident {
            $( $field:ident : $field_type:path ),+ $(,)?
        }
    )+) => {$(
        $(#[$attr])*
        #[repr(C)]
        $pub struct $name {
            $( $field : $field_type ),+
        }

        // SAFETY: bounds guarantee all fields also impl Structural, and repr(C)
        // guarantees a known memory layout
        unsafe impl crate::Structural for $name
        where
            $( $field_type: crate::Structural ),+
        {}
    )+};
    (
        #[repr(transparent)]
        $(#[$attr:meta])*
        $pub:vis struct $name:ident($field:path);
    ) => {
        $(#[$attr])*
        #[repr(transparent)]
        $pub struct $name($field);

        // SAFETY: bound guanantees the field also impls Structural, but it must
        // also be repr(C)
        unsafe impl crate::Structural for $name where $field: crate::Structural {}
    };
    (
        #[repr($repr:ty)]
        $(#[$attr:meta])*
        $pub:vis enum $name:ident {
            $($variant:ident $(= $value:expr)?),+ $(,)?
        }
    ) => {
        $(#[$attr])*
        #[repr($repr)]
        $pub enum $name {
            $($variant $(= $value)?),+
        }

        // SAFETY: bound guarantees the repr is `Structural`
        unsafe impl crate::Structural for $name where $repr: crate::Structural {}
    };
}
