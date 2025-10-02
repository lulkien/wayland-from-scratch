#[macro_export]
macro_rules! wl_primitive_type {
    (
        $(#[$meta:meta])*
        $name:ident($ty:ty)
    ) => {
        $(#[$meta])*
        #[allow(unused)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(pub $ty);

        impl $name {
            /// Returns the raw bytes of the value in native endianness.
            pub fn as_bytes(&self) -> [u8; std::mem::size_of::<$ty>()] {
                self.0.to_ne_bytes()
            }

            pub fn to_bytes(&self) -> Vec<u8> {
                self.as_bytes().to_vec()
            }

            /// Creates a new instance from raw bytes in native endianness.
            pub fn from_bytes(bytes: [u8; std::mem::size_of::<$ty>()]) -> Self {
                Self(<$ty>::from_ne_bytes(bytes))
            }

            pub fn get(&self) -> $ty {
                self.0
            }

            #[allow(dead_code)]
            pub const fn type_size() -> usize {
                size_of::<$ty>()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<$name> for $ty {
            fn from(value: $name) -> $ty {
                value.0
            }
        }

        impl From<$name> for Vec<u8> {
            fn from(value: $name) -> Vec<u8> {
                value.to_bytes()
            }
        }
    };
}

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

#[macro_export]
macro_rules! wl_request_opcode {
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
        #[repr(u16)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $name {
            $(
                $(#[$variant_meta])*
                $variant = $value,
            )*
        }

        impl From<$name> for u16 {
            fn from(value: $name) -> u16 {
                value as u16
            }
        }
    };
}

#[macro_export]
macro_rules! wl_request_param {
    (
        $(#[$meta:meta])*
        $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field:ident: $ty:ty
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        pub struct $name {
            $(
                $(#[$field_meta])*
                $field: $ty,
            )*
        }

        impl $name {
            /// Creates new parameters with the specified values.
            #[allow(unused)]
            pub(super) fn new($($field: $ty),*) -> Self {
                Self {
                    $($field),*
                }
            }
        }

        impl From<$name> for Vec<u8> {
            /// Serializes the parameters into the Wayland wire format.
            fn from(args: $name) -> Vec<u8> {
                let mut buffer = Vec::new();
                $(
                    buffer.extend_from_slice(&args.$field.to_bytes());
                )*
                buffer
            }
        }
    };
}
