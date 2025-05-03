pub(crate) fn to_base_field(field: &str) -> &str {
    field.split_once('[').map_or(field, |(base, _)| base)
}

macro_rules! include_generated {
    ($file:literal) => {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/generated/",
            $file,
            ".rs"
        ));
    };
}

macro_rules! impl_sign_conversions {
    ($as_i:ident, $i:ty, $as_u:ident, $u:ty) => {
        #[doc = concat!("Wrapper for `x as ", stringify!($i), "` that typechecks `x`")]
        #[allow(dead_code, clippy::cast_possible_wrap)]
        #[inline(always)]
        pub(crate) const fn $as_i(x: $u) -> $i {
            x as $i
        }

        #[doc = concat!("Wrapper for `x as ", stringify!($u), "` that typechecks `x`")]
        #[allow(dead_code, clippy::cast_sign_loss)]
        #[inline(always)]
        pub(crate) const fn $as_u(x: $i) -> $u {
            x as $u
        }
    };
}

impl_sign_conversions!(as_i8, i8, as_u8, u8);
impl_sign_conversions!(as_i16, i16, as_u16, u16);
impl_sign_conversions!(as_i32, i32, as_u32, u32);
impl_sign_conversions!(as_i64, i64, as_u64, u64);
impl_sign_conversions!(as_i128, i128, as_u128, u128);
impl_sign_conversions!(as_isize, isize, as_usize, usize);

macro_rules! byte_enum {
    (
        $( #[$attr:meta] )*
        $pub:vis enum $name:ident {
            $(
                $( #[$variant_attr:meta] )*
                $variant:ident = $value:expr
            ),+
            $(,)?
        }
    ) => {
        $( #[$attr] )*
        $pub enum $name {
            $( $( #[$variant_attr] )* $variant = $value ),+
        }

        impl $name {
            #[allow(dead_code)]
            pub(crate) const fn from_byte(byte: u8) -> Option<Self> {
                match byte {
                    $( $value => Some(Self::$variant), )+
                    _ => None,
                }
            }

            #[allow(dead_code)]
            pub(crate) fn from_num_str(s: &str) -> Option<Self> {
                match s {
                    $( stringify!($value) => Some(Self::$variant), )+
                    _ => None,
                }
            }
        }

        impl From<$name> for u8 {
            fn from(from: $name) -> u8 {
                match from {
                    $( $name::$variant => $value ),+
                }
            }
        }
    }
}
