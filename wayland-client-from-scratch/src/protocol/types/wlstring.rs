use super::roundup_4;

/// The size of the string length prefix in bytes (32-bit integer).
const WL_STRING_PREFIX_LEN: usize = 4;
/// The NUL terminator byte value used in Wayland strings.
const WL_NUL: u8 = 0;

/// Represents a Wayland protocol string type.
///
/// A string, prefixed with a 32-bit integer specifying its length (in bytes),
/// followed by the string contents and a NUL terminator, padded to 32 bits
/// with undefined data. The encoding is not specified, but in practice UTF-8 is used.
///
/// # Specification
/// Strings are transmitted as:
/// - 32-bit length prefix (string bytes + NUL terminator, excluding padding)
/// - String content bytes (UTF-8 encoded)
/// - NUL terminator byte
/// - Padding bytes to reach 32-bit alignment boundary
#[derive(Default)]
pub struct WlString {
    /// The size of the string content in bytes, including NUL terminator but excluding padding.
    ///
    /// This value represents the length of the meaningful string data as it appears
    /// in the length prefix, which includes the actual string bytes and the NUL terminator.
    size: u32,
    /// The string data bytes including NUL terminator and padding.
    ///
    /// This vector contains the UTF-8 string content, a NUL terminator byte,
    /// and padding bytes to reach 32-bit alignment.
    data: Vec<u8>,
}

impl WlString {
    /// Creates a new Wayland string from a Rust string.
    ///
    /// The input string is converted to UTF-8 bytes, a NUL terminator is added,
    /// and the result is padded to 32-bit alignment as required by the protocol.
    ///
    /// # Arguments
    /// * `s` - The string content to store
    pub fn new(s: &str) -> Self {
        let string_bytes = s.as_bytes();

        let mut data = string_bytes.to_vec();
        data.push(WL_NUL);

        let padded_size = roundup_4(data.len());
        data.resize(padded_size, 0);

        // Size is the string bytes + NUL terminator (excluding padding)
        let size = (string_bytes.len() + 1) as u32;

        Self { size, data }
    }

    /// Returns the total buffer size required for serialization.
    ///
    /// This includes both the 4-byte length prefix and the padded string content.
    ///
    /// # Returns
    /// The total number of bytes needed to represent this string in wire format.
    pub fn buffer_size(&self) -> usize {
        WL_STRING_PREFIX_LEN + self.data.len()
    }

    /// Returns the actual string content as a Rust string slice.
    ///
    /// Uses lossy UTF-8 conversion to handle any encoding errors gracefully.
    pub fn as_str(&self) -> &str {
        // The actual string content is everything before the NUL terminator
        // which is at position (self.size - 1) since size includes the NUL
        let string_len = (self.size - 1) as usize;
        std::str::from_utf8(&self.data[..string_len]).unwrap_or("")
    }
}

impl std::fmt::Display for WlString {
    /// Formats the string for human-readable display.
    ///
    /// Shows the wire size and the string content.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "WlString {{ size: {}, data: {} }}",
            self.size,
            self.as_str()
        )
    }
}

impl From<String> for WlString {
    /// Converts a Rust String to a Wayland protocol string.
    fn from(s: String) -> Self {
        Self::new(&s)
    }
}

impl From<&WlString> for String {
    /// Converts a Wayland string to a Rust String.
    ///
    /// Uses lossy UTF-8 conversion to handle any encoding errors gracefully.
    fn from(wls: &WlString) -> String {
        wls.as_str().to_string()
    }
}

impl From<WlString> for Vec<u8> {
    /// Serializes the string into the Wayland wire format.
    ///
    /// Produces a byte vector containing:
    /// - 4-byte length prefix (32-bit integer, string bytes + NUL)
    /// - String content bytes (UTF-8 encoded)
    /// - NUL terminator byte
    /// - Padding bytes to reach 32-bit alignment
    ///
    /// # Returns
    /// A byte vector in the Wayland string wire format.
    fn from(wls: WlString) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(WL_STRING_PREFIX_LEN + wls.data.len());

        // Add 32-bit length prefix (string bytes + NUL, excluding padding)
        buffer.extend_from_slice(&wls.size.to_ne_bytes());

        // Add string content bytes (including NUL terminator and padding)
        buffer.extend_from_slice(&wls.data);

        buffer
    }
}

impl TryFrom<&[u8]> for WlString {
    type Error = anyhow::Error;

    /// Deserializes a Wayland string from the wire format.
    ///
    /// Parses the byte buffer according to the Wayland string specification:
    /// - Reads 32-bit length prefix (string bytes + NUL)
    /// - Extracts and validates string content
    /// - Locates and validates NUL terminator
    /// - Validates buffer contains sufficient data
    ///
    /// # Arguments
    /// * `buf` - The byte buffer containing serialized string data
    ///
    /// # Returns
    /// * `Ok(WlString)` if the buffer contains valid string data
    /// * `Err(anyhow::Error)` if the buffer is malformed or incomplete
    ///
    /// # Errors
    /// Returns an error if:
    /// - Buffer is too short for the length prefix (less than 4 bytes)
    /// - Buffer is too short for the declared string content
    /// - NUL terminator is missing from the string content
    fn try_from(buf: &[u8]) -> anyhow::Result<WlString> {
        if buf.len() < WL_STRING_PREFIX_LEN {
            return Err(anyhow::anyhow!(
                "Buffer too short for WlString length field: expected at least {} bytes, got {}",
                WL_STRING_PREFIX_LEN,
                buf.len()
            ));
        }

        // Extract 32-bit length prefix from first 4 bytes
        // This is the string bytes + NUL terminator (excluding padding)
        let content_len = u32::from_ne_bytes(buf[..WL_STRING_PREFIX_LEN].try_into()?) as usize;

        // Calculate padded length for buffer extraction
        let padded_len = roundup_4(content_len);
        let total_buffer_len = WL_STRING_PREFIX_LEN + padded_len;

        // Check if we have enough data for the content section (with padding)
        if buf.len() < total_buffer_len {
            return Err(anyhow::anyhow!(
                "Buffer too short for WlString content: expected at least {} bytes, got {}",
                total_buffer_len,
                buf.len()
            ));
        }

        // Extract the content section (includes string data, NUL terminator, and padding)
        let content_section = &buf[WL_STRING_PREFIX_LEN..total_buffer_len];

        // Validate NUL terminator is at the expected position
        if content_len > 0 && content_section[content_len - 1] == WL_NUL {
            Ok(WlString {
                size: content_len as u32,
                data: content_section.to_vec(),
            })
        } else {
            Err(anyhow::anyhow!("Missing NUL terminator in WlString"))
        }
    }
}
