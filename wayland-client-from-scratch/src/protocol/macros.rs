#[macro_export]
macro_rules! wl_enum {
    (
        $(#[$meta:meta])*
        $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident = $value:expr
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[repr(u32)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $name {
            $(
                $(#[$variant_meta])*
                $variant = $value,
            )*
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        $name::$variant => write!(f, "{}::{}", stringify!($name), stringify!($variant)),
                    )*
                }
            }
        }

        impl TryFrom<u32> for $name {
            type Error = anyhow::Error;

            fn try_from(value: u32) -> anyhow::Result<Self> {
                match value {
                    $(
                        $value => Ok($name::$variant),
                    )*
                    _ => Err(anyhow::anyhow!("Invalid {} value: {}", stringify!($name), value)),
                }
            }
        }
    };
}
