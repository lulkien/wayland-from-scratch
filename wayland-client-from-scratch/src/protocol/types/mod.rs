pub mod wlarray;
pub mod wlstring;

#[allow(unused)]
pub use wlarray::WlArray;
pub use wlstring::WlString;

#[allow(dead_code)]
type WlUint = u32;

#[allow(dead_code)]
type WlInt = i32;

#[allow(dead_code)]
type WlObject = u32;

#[allow(dead_code)]
type WlNewId = u32;

#[allow(dead_code)]
type WlEnum = u32;

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
