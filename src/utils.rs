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
