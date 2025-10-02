#[macro_export]
macro_rules! wl_struct {
    ($name:ident {
        $($field:ident: $ty:ty),* $(,)?
    }) => {
        #[derive(Debug, Clone)]
        pub struct $name {
            $(pub $field: $ty),*
        }
    };
}
