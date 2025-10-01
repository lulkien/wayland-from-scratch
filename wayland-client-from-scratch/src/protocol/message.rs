use std::fmt::{self, Display, Formatter};

use anyhow::anyhow;

/// The fixed size of a Wayland message header in bytes (8 bytes).
///
/// Wayland message headers consist of two 32-bit words:
/// - Object ID (32 bits)
/// - Combined size (upper 16 bits) and opcode (lower 16 bits)
pub const WL_MESSAGE_HEADER_LEN: usize = size_of::<u32>() + size_of::<u16>() + size_of::<u16>();

/// Represents the header of a Wayland protocol message.
///
/// Contains routing information and metadata for interpreting Wayland messages.
/// The header is always 8 bytes and precedes the variable-length message data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WlMessageHeader {
    /// The object ID that this message targets or originates from.
    pub(crate) object_id: u32,
    /// The operation code defining the specific request or event type.
    pub(crate) opcode: u16,
    /// The total message size including header and data in bytes.
    pub(crate) size: u16,
}

impl WlMessageHeader {
    /// Returns the total length of the message including header and data.
    fn message_len(&self) -> usize {
        self.size as usize
    }
}

impl From<WlMessageHeader> for Vec<u8> {
    /// Serializes the header into the Wayland wire format.
    ///
    /// Produces an 8-byte vector with native endian encoding:
    /// - Bytes 0-3: object_id
    /// - Bytes 4-5: opcode  
    /// - Bytes 6-7: size
    fn from(header: WlMessageHeader) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(8);

        bytes.extend_from_slice(&header.object_id.to_ne_bytes());
        bytes.extend_from_slice(&header.opcode.to_ne_bytes());
        bytes.extend_from_slice(&header.size.to_ne_bytes());

        bytes
    }
}

impl TryFrom<&[u8]> for WlMessageHeader {
    type Error = anyhow::Error;

    /// Deserializes a header from the wire format.
    ///
    /// # Errors
    /// Returns an error if:
    /// - Buffer is shorter than 8 bytes
    /// - Buffer contains invalid data
    fn try_from(buf: &[u8]) -> anyhow::Result<Self> {
        if buf.len() < WL_MESSAGE_HEADER_LEN {
            return Err(anyhow!(
                "Buffer too short for WlMessageHeader: expected {} bytes, got {}",
                WL_MESSAGE_HEADER_LEN,
                buf.len()
            ));
        }

        let object_id = u32::from_ne_bytes(buf[0..4].try_into()?);
        let opcode = u16::from_ne_bytes(buf[4..6].try_into()?);
        let size = u16::from_ne_bytes(buf[6..8].try_into()?);

        Ok(WlMessageHeader {
            object_id,
            opcode,
            size,
        })
    }
}

impl Display for WlMessageHeader {
    /// Formats the header for human-readable display.
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "WlMessageHeader {{ object_id: {}, opcode: {}, size: {} }}",
            self.object_id, self.opcode, self.size
        )
    }
}

/// A complete Wayland protocol message containing header and data.
pub struct WlMessage {
    /// The message header with routing and metadata.
    pub(crate) header: WlMessageHeader,
    /// The message payload data.
    pub(crate) data: Vec<u8>,
}

impl WlMessage {
    /// Creates a new Wayland message.
    ///
    /// The size field is automatically calculated as header length plus data length.
    pub fn new(object_id: u32, opcode: u16, data: &[u8]) -> WlMessage {
        WlMessage {
            header: WlMessageHeader {
                object_id,
                opcode,
                size: (data.len() + WL_MESSAGE_HEADER_LEN) as u16,
            },
            data: data.to_vec(),
        }
    }
}

impl From<WlMessage> for Vec<u8> {
    /// Serializes the complete message into wire format.
    fn from(msg: WlMessage) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(msg.header.size.into());

        let mut header_raw: Vec<u8> = msg.header.into();
        bytes.append(&mut header_raw);
        bytes.extend_from_slice(&msg.data);

        bytes
    }
}

impl TryFrom<&[u8]> for WlMessage {
    type Error = anyhow::Error;

    /// Deserializes a complete message from wire format.
    ///
    /// # Errors
    /// Returns an error if:
    /// - Buffer is shorter than header length
    /// - Buffer length doesn't match declared message size
    /// - Header contains invalid data
    fn try_from(buf: &[u8]) -> anyhow::Result<WlMessage> {
        if buf.len() < WL_MESSAGE_HEADER_LEN {
            return Err(anyhow!(
                "Buffer too short for WlMessage header: expected at least {} bytes, got {}",
                WL_MESSAGE_HEADER_LEN,
                buf.len()
            ));
        }

        let header: WlMessageHeader = buf[..WL_MESSAGE_HEADER_LEN].try_into()?;

        if buf.len() < header.message_len() {
            return Err(anyhow!(
                "Buffer too short for WlMessage: expected at least {} bytes, got {}",
                header.message_len(),
                buf.len()
            ));
        }

        Ok(WlMessage {
            header,
            data: buf[WL_MESSAGE_HEADER_LEN..].to_vec(),
        })
    }
}

impl Display for WlMessage {
    /// Formats the complete message for human-readable display.
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let data_dump = self
            .data
            .iter()
            .map(|b| format!("0x{:02X}", b))
            .collect::<Vec<String>>()
            .join(", ");

        write!(
            f,
            "WlMessage {{ header: {}, data: [{}] }}",
            self.header, data_dump
        )
    }
}

/// An iterator that parses complete Wayland messages from a byte buffer.
///
/// Consumes messages from the buffer as they are parsed, making it suitable
/// for processing streaming protocol data.
pub struct WlMessageIter {
    buffer: Vec<u8>,
}

impl WlMessageIter {
    /// Creates a new iterator from a byte buffer.
    pub fn new(buffer: Vec<u8>) -> WlMessageIter {
        Self { buffer }
    }

    /// Attempts to parse the next complete message from the buffer.
    ///
    /// Returns `Some(message)` if a complete message is available and valid.
    /// Returns `None` if the buffer contains insufficient or invalid data.
    ///
    /// On success, the parsed message is removed from the internal buffer.
    pub fn next(&mut self) -> Option<WlMessage> {
        // Check if we have enough data for at least a header
        if self.buffer.len() < WL_MESSAGE_HEADER_LEN {
            self.buffer.clear();
            return None;
        }

        // Parse the WlMessageHeader
        let header = WlMessageHeader::try_from(&self.buffer[..WL_MESSAGE_HEADER_LEN]).ok()?;

        // Check if we have the complete message
        if self.buffer.len() < header.message_len() {
            self.buffer.clear();
            return None;
        }

        // Extract and parse the complete message
        match WlMessage::try_from(&self.buffer[..header.message_len()]) {
            Ok(message) => {
                // Successfully parsed - remove the message bytes from buffer
                self.buffer.drain(..header.message_len());
                Some(message)
            }
            Err(_) => {
                // Message data is corrupted - clear buffer
                self.buffer.clear();
                None
            }
        }
    }
}
