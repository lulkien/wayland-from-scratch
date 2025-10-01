use std::fmt::{self, Display, Formatter};

const WL_STRING_PREFIX_LEN: usize = 4;
const WL_NUL: u8 = 0;

/// Represents a Wayland protocol string.
///
/// Wayland strings are serialized with:
/// - 32-bit length prefix (in bytes)
/// - String content in UTF-8 encoding
/// - NUL terminator byte
/// - Padding to 32-bit alignment with undefined data
///
/// This struct stores the decoded length and data for efficient manipulation.
#[derive(Default)]
pub struct WlString {
    /// The length of the string content in bytes (excluding NUL terminator and padding)
    len: u32,
    /// The UTF-8 encoded string data
    data: Vec<u8>,
}

impl WlString {
    /// Calculates the total buffer size required for serializing this Wayland string.
    ///
    /// This method returns the complete byte count needed to represent this string
    /// in the Wayland wire format. Since the `len` field already contains the
    /// rounded-up length of the content (including NUL terminator and padding),
    /// the total size is simply the 4-byte length prefix plus the stored length.
    ///
    /// # Returns
    ///
    /// The total number of bytes required to serialize this string in Wayland protocol format
    ///
    /// # Calculation
    /// The formula used is: `4 + len`
    /// Where:
    /// - `4` bytes for the 32-bit length prefix
    /// - `len` bytes for the aligned content
    ///
    /// # Examples
    /// ```
    /// // Assuming len field contains the rounded-up size including NUL and padding
    /// let wl_string = WlString { len: 4, data: vec![b'h', b'i'] }; // "hi" -> 2 content + 1 NUL + 1 padding = 4
    /// assert_eq!(wl_string.buffer_len(), 8); // 4 (prefix) + 4 (content) = 8
    ///
    /// let wl_string = WlString { len: 8, data: vec![b'h', b'e', b'l', b'l', b'o'] }; // "hello" -> 5 content + 1 NUL + 2 padding = 8
    /// assert_eq!(wl_string.buffer_len(), 12); // 4 (prefix) + 8 (content) = 12
    ///
    /// let wl_string = WlString { len: 4, data: vec![] }; // "" -> 0 content + 1 NUL + 3 padding = 4
    /// assert_eq!(wl_string.buffer_len(), 8); // 4 (prefix) + 4 (content) = 8
    /// ```
    ///
    /// # Protocol Context
    /// This method is essential for pre-allocating buffers when serializing multiple
    /// Wayland messages. Since the `len` field already accounts for the aligned size
    /// of the content (including NUL terminator and padding), the calculation is
    /// straightforward and efficient.
    pub fn buffer_len(&self) -> usize {
        4 + self.len as usize
    }
}

impl From<String> for WlString {
    /// Converts a Rust String to a Wayland protocol string.
    ///
    /// This conversion encodes the string as UTF-8 and calculates the appropriate length
    /// for the Wayland protocol format.
    ///
    /// # Arguments
    ///
    /// * `s` - The Rust string to convert
    ///
    /// # Returns
    ///
    /// A `WlString` ready for serialization to the Wayland wire format
    fn from(s: String) -> Self {
        let data = s.into_bytes();
        let len = data.len() as u32;
        Self { len, data }
    }
}

impl From<&WlString> for String {
    /// Converts a reference to a Wayland protocol string to a Rust String.
    ///
    /// This conversion attempts to interpret the stored byte data as UTF-8.
    /// If the data is not valid UTF-8, the lossy conversion will be used,
    /// replacing invalid sequences with the Unicode replacement character (�).
    ///
    /// This method preserves the original WlString while allowing conversion
    /// to a Rust String for display or processing.
    ///
    /// # Arguments
    ///
    /// * `wls` - A reference to the Wayland string to convert
    ///
    /// # Returns
    ///
    /// A Rust String containing the decoded content
    ///
    /// # Examples
    ///
    /// ```
    /// let wl_string = WlString::from("Hello, Wayland!".to_string());
    /// let rust_string = String::from(&wl_string);
    /// assert_eq!(rust_string, "Hello, Wayland!");
    /// // wl_string is still available for use
    /// ```
    fn from(wls: &WlString) -> String {
        String::from_utf8_lossy(&wls.data).into_owned()
    }
}

impl From<WlString> for Vec<u8> {
    /// Serializes a Wayland string to the wire format.
    ///
    /// Produces a byte vector containing:
    /// 1. 32-bit length prefix (native endian)
    /// 2. String content bytes
    /// 3. NUL terminator byte
    /// 4. Padding bytes to reach 32-bit alignment
    ///
    /// # Arguments
    ///
    /// * `wls` - The Wayland string to serialize
    ///
    /// # Returns
    ///
    /// Byte vector in Wayland wire format
    fn from(wls: WlString) -> Vec<u8> {
        let mut result = Vec::new();

        // Add 32-bit length prefix
        result.extend_from_slice(&wls.len.to_ne_bytes());

        // Add string content bytes
        result.extend_from_slice(&wls.data);

        // Add mandatory NUL terminator
        result.push(WL_NUL);

        // Pad to 32-bit alignment boundary
        let current_len = result.len();
        let padded_len = roundup_4(current_len);
        result.resize(padded_len, 0);

        result
    }
}

impl TryFrom<&[u8]> for WlString {
    type Error = anyhow::Error;

    /// Deserializes a Wayland string from wire format bytes.
    ///
    /// Parses the byte buffer according to the Wayland string specification:
    /// - Reads 32-bit length prefix
    /// - Extracts string data of specified length
    /// - Verifies NUL terminator presence
    ///
    /// # Arguments
    ///
    /// * `buf` - Byte buffer containing the serialized Wayland string
    ///
    /// # Returns
    ///
    /// `Ok(WlString)` if deserialization succeeds
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Buffer is too short for length field
    /// - Buffer is too short for declared string length
    /// - NUL terminator is missing
    fn try_from(buf: &[u8]) -> anyhow::Result<WlString> {
        if buf.len() < WL_STRING_PREFIX_LEN {
            return Err(anyhow::anyhow!(
                "Buffer too short for WlString length field: expected at least 4 bytes, got {}",
                buf.len()
            ));
        }

        // Extract 32-bit length prefix from first 4 bytes
        let total_content_len =
            u32::from_ne_bytes(buf[0..WL_STRING_PREFIX_LEN].try_into()?) as usize;
        let buffer_len = WL_STRING_PREFIX_LEN + total_content_len;

        // Check if we have enough data for the content section
        if buf.len() < buffer_len {
            return Err(anyhow::anyhow!(
                "Buffer too short for WlString content: expected at least {} bytes, got {}",
                buffer_len,
                buf.len()
            ));
        }

        // Extract the content section (includes string data, NUL terminator, and padding)
        let content_section = &buf[WL_STRING_PREFIX_LEN..buffer_len];

        // Find the NUL terminator within the content section
        if let Some(null_pos) = content_section.iter().position(|&x| x == WL_NUL) {
            // Extract only the actual string data (excluding NUL terminator and padding)
            let string_data = content_section[..null_pos].to_vec();

            Ok(WlString {
                len: total_content_len as u32,
                data: string_data,
            })
        } else {
            Err(anyhow::anyhow!("Missing NUL terminator in WlString"))
        }
    }
}

impl Display for WlString {
    /// Formats the Wayland string with complete structural information for debugging and display.
    ///
    /// This implementation provides a detailed view of the Wayland string's internal state,
    /// showing both the protocol-level length field and the actual string content. This is
    /// essential for protocol debugging where both metadata and content need to be visible.
    ///
    /// # Output Format
    /// `WlString { len: <protocol_length>, string: "<content>" }`
    ///
    /// Where:
    /// - `protocol_length` is the 32-bit length value from the Wayland wire format
    /// - `content` is the actual string data converted to UTF-8 for display
    ///
    /// # Behavior
    /// - Displays the exact protocol length as stored in the length field
    /// - Shows the string content with proper UTF-8 conversion using lossy handling
    /// - Wraps the string content in quotes for clear visual delineation
    /// - Handles empty strings and strings with invalid UTF-8 sequences gracefully
    ///
    /// # Examples
    /// ```
    /// let wl_string = WlString::from("Hello Wayland".to_string());
    /// println!("{}", wl_string);
    /// // Prints: WlString { len: 13, data: "Hello Wayland" }
    ///
    /// let empty_wl_string = WlString::from("".to_string());
    /// println!("{}", empty_wl_string);
    /// // Prints: WlString { len: 0, data: "" }
    ///
    /// // Even with invalid UTF-8, the display remains safe
    /// let mut invalid_wl_string = WlString::from("test".to_string());
    /// invalid_wl_string.data = vec![0xFF, 0xFE]; // Invalid UTF-8
    /// println!("{}", invalid_wl_string);
    /// // Prints: WlString { len: 2, data: "��" }
    /// ```
    ///
    /// # Protocol Context
    /// In the Wayland protocol, strings are transmitted with a 32-bit length prefix
    /// followed by UTF-8 encoded data, a NUL terminator, and padding to 32-bit alignment.
    /// This display format helps identify discrepancies between the declared length
    /// and actual content, which is crucial for debugging protocol implementation issues.
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // Convert the internal byte data to a displayable string using lossy conversion
        // This ensures we can display even malformed UTF-8 data from buggy implementations
        let content = String::from_utf8_lossy(&self.data);

        // Format with both protocol length and string content
        write!(f, "WlString {{ len: {}, data: \"{}\" }}", self.len, content)
    }
}

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
