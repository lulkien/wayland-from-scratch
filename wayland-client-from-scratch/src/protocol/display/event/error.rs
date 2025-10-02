use std::fmt::{self, Display, Formatter};
use std::mem::size_of;

use anyhow::anyhow;

use crate::protocol::types::WlString;

const WL_DISPLAY_ERROR_OBJECT_LEN: usize = size_of::<u32>();
const WL_DISPLAY_ERROR_CODE_LEN: usize = size_of::<u32>();

/// Represents the specific error codes that can be reported by the Wayland display.
///
/// These are global error values that can be emitted in response to any server request
/// and indicate fundamental protocol violations or system failures.
#[derive(Debug, Clone, Copy)]
pub enum WlDisplayErrorId {
    /// The server couldn't find the specified object.
    /// This typically occurs when a client references an object that has been destroyed
    /// or was never properly created.
    InvalidObject = 0,

    /// The requested method doesn't exist on the specified interface or the request was malformed.
    /// This indicates either an interface version mismatch or a protocol encoding error.
    InvalidMethod = 1,

    /// The server is out of memory and cannot fulfill the request.
    /// This is a critical system-level failure that may require client termination.
    NoMemory = 2,

    /// An implementation error occurred in the compositor.
    /// This indicates a bug in the compositor implementation rather than a client protocol error.
    ImplementationError = 3,
}

impl Display for WlDisplayErrorId {
    /// Formats the error code as a human-readable string for debugging and logging.
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidObject => write!(f, "WlDisplayError::InvalidObject"),
            Self::InvalidMethod => write!(f, "WlDisplayError::InvalidMethod"),
            Self::NoMemory => write!(f, "WlDisplayError::NoMemory"),
            Self::ImplementationError => write!(f, "WlDisplayError::ImplementationError"),
        }
    }
}

impl TryFrom<u32> for WlDisplayErrorId {
    type Error = anyhow::Error;

    /// Converts a raw error code value to a structured `WlDisplayErrorId`.
    ///
    /// # Arguments
    /// * `value` - The raw error code from the protocol message
    ///
    /// # Returns
    /// * `Ok(WlDisplayErrorId)` if the code corresponds to a known error type
    /// * `Err(anyhow::Error)` if the code is unrecognized
    ///
    /// # Protocol Context
    /// The Wayland core protocol defines these four global error codes that can be
    /// returned for any interface, providing consistent error handling across the protocol.
    fn try_from(value: u32) -> anyhow::Result<WlDisplayErrorId> {
        match value {
            0 => Ok(Self::InvalidObject),
            1 => Ok(Self::InvalidMethod),
            2 => Ok(Self::NoMemory),
            3 => Ok(Self::ImplementationError),
            _ => Err(anyhow!("Unknown WlDisplay error code: {}", value)),
        }
    }
}

/// Represents a complete fatal error event from the Wayland display.
///
/// This structure contains all the information from a wl_display.error event,
/// including the object where the error occurred, the specific error code,
/// and a human-readable message for debugging purposes.
pub struct WlDisplayError {
    /// The object ID where the error occurred, typically the target of a failed request.
    object_id: u32,

    /// The specific type of error that occurred.
    code: WlDisplayErrorId,

    /// A brief description of the error, intended for debugging convenience.
    /// The content and format of this message is implementation-defined.
    message: WlString,
}

impl Display for WlDisplayError {
    /// Formats the complete error information for display and logging.
    ///
    /// # Output Format
    /// `WlDisplayError { object_id: <id>, code: <code>, message: "<message>" }`
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "WlDisplayError {{ object_id: {}, code: {}, message: {} }}",
            self.object_id, self.code, self.message
        )
    }
}

impl TryFrom<&[u8]> for WlDisplayError {
    type Error = anyhow::Error;

    /// Parses a raw byte buffer into a structured `WlDisplayError`.
    ///
    /// # Arguments
    /// * `buf` - The byte buffer containing the serialized error event data
    ///
    /// # Returns
    /// * `Ok(WlDisplayError)` if the buffer contains valid error data
    /// * `Err(anyhow::Error)` if the buffer is malformed or incomplete
    ///
    /// # Buffer Layout
    /// The error event data is structured as:
    /// - Bytes 0-3: `object_id` (u32) - The object where the error occurred
    /// - Bytes 4-7: `code` (u32) - The error code
    /// - Remaining bytes: `message` (WlString) - The error description string
    ///
    /// # Protocol Specification
    /// This follows the wl_display.error event format defined in the Wayland protocol:
    /// ```xml
    /// <event name="error">
    ///   <arg name="object_id" type="object" summary="object where the error occurred"/>
    ///   <arg name="code" type="uint" summary="error code"/>
    ///   <arg name="message" type="string" summary="error description"/>
    /// </event>
    /// ```
    fn try_from(buf: &[u8]) -> anyhow::Result<WlDisplayError> {
        // Extract object_id(u32) from buffer
        if buf.len() < WL_DISPLAY_ERROR_OBJECT_LEN {
            return Err(anyhow!(
                "Buffer too short for WlDisplayError object_id: expected {} bytes, got {}",
                WL_DISPLAY_ERROR_OBJECT_LEN,
                buf.len()
            ));
        }
        let object_id = u32::from_ne_bytes(buf[0..size_of::<u32>()].try_into()?);

        // Extract code(u32) from buffer
        let code_start_pos = WL_DISPLAY_ERROR_OBJECT_LEN;
        if buf.len() < code_start_pos + WL_DISPLAY_ERROR_CODE_LEN {
            return Err(anyhow!(
                "Buffer too short for WlDisplayError code: expected {} bytes, got {}",
                size_of::<u32>(),
                buf.len()
            ));
        }
        let code_raw = u32::from_ne_bytes(buf[0..size_of::<u32>()].try_into()?);
        let code = WlDisplayErrorId::try_from(code_raw)?;

        // Parse error message string - human-readable description
        let message_start_pos = code_start_pos + WL_DISPLAY_ERROR_CODE_LEN;
        let message: WlString = buf[message_start_pos..].try_into()?;

        Ok(WlDisplayError {
            object_id,
            code,
            message,
        })
    }
}

/// Handles a fatal error event from the Wayland display.
///
/// This function processes wl_display.error events, which indicate non-recoverable
/// protocol errors. It parses the error information, logs it for debugging, and
/// returns an error to signal that the connection should be terminated.
///
/// # Arguments
/// * `buf` - The raw byte buffer containing the error event data
///
/// # Returns
/// * `Err(anyhow::Error)` - Always returns an error since these are fatal conditions
///
/// # Behavior
/// - Parses the error event into structured data
/// - Prints the error to stderr for debugging visibility
/// - Returns an error to propagate the fatal condition upward
///
/// # Protocol Significance
/// According to the Wayland specification, error events are fatal and non-recoverable.
/// When a client receives this event, it should typically terminate the connection
/// as the protocol state may be compromised.
pub(super) fn handle_wl_display_error(buf: &[u8]) -> anyhow::Result<()> {
    let error = WlDisplayError::try_from(buf)?;

    // Log the fatal error for debugging purposes
    eprintln!("[FATAL] Wayland protocol error: {}", error);

    // Propagate the error to signal that the connection should be terminated
    Err(anyhow!("Fatal Wayland protocol error: {}", error))
}
