use super::roundup_4;

/// The size of the array length prefix in bytes (32-bit integer).
const WL_ARRAY_PREFIX_LEN: usize = size_of::<u32>();

/// Represents a Wayland protocol array type.
///
/// A blob of arbitrary data, prefixed with a 32-bit integer specifying its length
/// (in bytes), then the verbatim contents of the array, padded to 32 bits with
/// undefined data.
///
/// # Specification
/// Arrays are transmitted as:
/// - 32-bit length prefix (number of bytes in the array, excluding padding)
/// - Array data bytes (verbatim content)
/// - Padding bytes to reach 32-bit alignment boundary
///
/// Unlike strings, arrays do not include a NUL terminator.
pub struct WlArray {
    /// The size of the array data in bytes, excluding padding.
    ///
    /// This value represents the length of the meaningful array data as it appears
    /// in the length prefix, which excludes any padding bytes.
    size: u32,
    /// The actual array data bytes.
    ///
    /// This vector contains the array content and has already been padded to
    /// 32-bit alignment as required by the Wayland protocol specification.
    data: Vec<u8>,
}

impl WlArray {
    /// Creates a new Wayland array from byte data.
    ///
    /// The input data is automatically padded to 32-bit alignment as required
    /// by the Wayland protocol specification.
    ///
    /// # Arguments
    /// * `buffer` - The raw byte data to store in the array
    pub fn new(buffer: &[u8]) -> Self {
        let data = buffer.to_vec();
        let padded_size = roundup_4(data.len());

        let mut padded_data = data;
        padded_data.resize(padded_size, 0);

        Self {
            size: buffer.len() as u32,
            data: padded_data,
        }
    }

    /// Returns the total buffer size required for serialization.
    ///
    /// This includes both the 4-byte length prefix and the padded array content.
    ///
    /// # Returns
    /// The total number of bytes needed to represent this array in wire format.
    pub fn buffer_size(&self) -> usize {
        WL_ARRAY_PREFIX_LEN + self.data.len()
    }

    /// Returns the actual array data as a slice, excluding padding.
    ///
    /// # Returns
    /// A slice containing the meaningful array data without padding bytes.
    pub fn as_slice(&self) -> &[u8] {
        &self.data[..self.size as usize]
    }
}

impl std::fmt::Display for WlArray {
    /// Formats the array for human-readable display.
    ///
    /// Shows the array size and the hexadecimal representation of the data bytes.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data_dump = self
            .as_slice()
            .iter()
            .map(|b| format!("0x{:02X}", b))
            .collect::<Vec<String>>()
            .join(", ");

        write!(
            f,
            "WlArray {{ size: {}, data: [ {} ] }}",
            self.size, data_dump
        )
    }
}

impl From<WlArray> for Vec<u8> {
    /// Serializes the array into the Wayland wire format.
    ///
    /// Produces a byte vector containing:
    /// - 4-byte length prefix (32-bit integer, array bytes excluding padding)
    /// - Array data bytes (padded to 32-bit alignment)
    ///
    /// # Returns
    /// A byte vector in the Wayland array wire format.
    fn from(array: WlArray) -> Self {
        let mut buffer = Vec::with_capacity(WL_ARRAY_PREFIX_LEN + array.data.len());

        // Add 32-bit length prefix (array bytes excluding padding)
        buffer.extend_from_slice(&array.size.to_ne_bytes());

        // Add array content bytes (already padded during construction)
        buffer.extend_from_slice(&array.data);

        buffer
    }
}

impl TryFrom<&[u8]> for WlArray {
    type Error = anyhow::Error;

    /// Deserializes a Wayland array from the wire format.
    ///
    /// Parses the byte buffer according to the Wayland array specification:
    /// - Reads 32-bit length prefix (array bytes excluding padding)
    /// - Extracts array data and calculates required padding
    /// - Validates buffer contains sufficient data
    ///
    /// # Arguments
    /// * `buffer` - The byte buffer containing serialized array data
    ///
    /// # Returns
    /// * `Ok(WlArray)` if the buffer contains valid array data
    /// * `Err(anyhow::Error)` if the buffer is malformed or incomplete
    ///
    /// # Errors
    /// Returns an error if:
    /// - Buffer is too short for the length prefix (less than 4 bytes)
    /// - Buffer is too short for the declared array content
    fn try_from(buffer: &[u8]) -> anyhow::Result<WlArray> {
        if buffer.len() < WL_ARRAY_PREFIX_LEN {
            return Err(anyhow::anyhow!(
                "Buffer too short for WlArray length field: expected at least {} bytes, got {}",
                WL_ARRAY_PREFIX_LEN,
                buffer.len()
            ));
        }

        // Extract 32-bit length prefix from first 4 bytes
        // This is the array bytes excluding padding
        let content_len = u32::from_ne_bytes(buffer[..WL_ARRAY_PREFIX_LEN].try_into()?) as usize;

        // Calculate padded length for buffer extraction
        let padded_len = roundup_4(content_len);
        let total_buffer_len = WL_ARRAY_PREFIX_LEN + padded_len;

        if buffer.len() < total_buffer_len {
            return Err(anyhow::anyhow!(
                "Buffer too short for WlArray content: expected at least {} bytes, got {}",
                total_buffer_len,
                buffer.len()
            ));
        }

        // Extract content of the array from buffer (including padding)
        let data = buffer[WL_ARRAY_PREFIX_LEN..total_buffer_len].to_vec();

        Ok(WlArray {
            size: content_len as u32,
            data,
        })
    }
}
