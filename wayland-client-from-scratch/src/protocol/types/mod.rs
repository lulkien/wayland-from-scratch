pub mod wlarray;
pub mod wlstring;

use crate::wl_primitive_type;

#[allow(unused)]
pub use wlarray::WlArray;
pub use wlstring::WlString;

wl_primitive_type!(WlUInt(i32));
wl_primitive_type!(WlInt(i32));
wl_primitive_type!(WlObject(u32));
wl_primitive_type!(WlNewId(u32));
wl_primitive_type!(WlEnum(u32));

pub const WL_TYPE_UINT_LEN: usize = WlUInt::type_size();
pub const WL_TYPE_OBJECT_LEN: usize = WlObject::type_size();
#[allow(dead_code)]
pub const WL_TYPE_NEWID_LEN: usize = WlNewId::type_size();
pub const WL_TYPE_ENUM_LEN: usize = WlEnum::type_size();

/// Rounds a size up to the nearest multiple of 4 for 32-bit alignment.
///
/// Wayland protocol requires many data structures to be 32-bit aligned.
/// This function calculates the padded size needed for proper alignment.
///
/// # Arguments
///
/// * `number` - The original size to align
///
/// # Returns
///
/// The smallest multiple of 4 that is greater than or equal to `number`
///
/// # Examples
///
/// ```
/// assert_eq!(roundup_4(5), 8);
/// assert_eq!(roundup_4(8), 8);
/// assert_eq!(roundup_4(9), 12);
/// ```
fn roundup_4(number: usize) -> usize {
    (number + 3) & !3
}
