pub mod delete_id;
pub mod error;

use anyhow::anyhow;

use crate::protocol::message::WlMessage;

/// Represents the event types that can be emitted by the Wayland display object.
///
/// The Wayland display is the core global object and special singleton that handles
/// internal Wayland protocol features. It serves as the entry point for clients to
/// connect to the compositor and manage protocol-level operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// Indicates a fatal (non-recoverable) error has occurred in the protocol.
    ///
    /// This event is sent when a serious error occurs, typically in response to a
    /// client request. The error details include the object where the error occurred,
    /// an interface-specific error code, and a descriptive message for debugging.
    ///
    /// # Event Arguments
    /// - `object_id`: The object where the error occurred
    /// - `code`: Interface-specific error code
    /// - `message`: Human-readable error description
    Error = 0,

    /// Acknowledges object ID deletion and allows safe ID reuse.
    ///
    /// This internal event is used by the object ID management system. When a client
    /// deletes an object it created, the server sends this event to confirm it has
    /// processed the deletion. Upon receipt, the client knows it can safely reuse
    /// the object ID for new objects.
    ///
    /// # Event Arguments
    /// - `id`: The deleted object ID that can now be reused
    DeleteId = 1,
}

impl TryFrom<u16> for Event {
    type Error = anyhow::Error;

    /// Attempts to convert a raw opcode value into a structured `WlDisplayEvent`.
    ///
    /// # Arguments
    /// * `value` - The opcode value from the message header
    ///
    /// # Returns
    /// * `Ok(WlDisplayEvent)` if the opcode corresponds to a known display event type
    /// * `Err(anyhow::Error)` if the opcode is unrecognized
    ///
    /// # Protocol Context
    /// The display object uses opcode 0 for error notifications and opcode 1 for
    /// delete ID acknowledgments as defined in the Wayland core protocol specification.
    fn try_from(value: u16) -> anyhow::Result<Event> {
        match value {
            0 => Ok(Event::Error),
            1 => Ok(Event::DeleteId),
            _ => Err(anyhow!("Invalid wl_display event opcode: {}", value)),
        }
    }
}

/// Dispatches incoming Wayland display events to their appropriate handler functions.
///
/// This function serves as the main entry point for processing events targeted at
/// the core display singleton object (object ID 1). It handles protocol-level events
/// that affect the entire client-compositor connection.
///
/// # Arguments
/// * `msg` - The complete Wayland message containing both header and payload data
///
/// # Returns
/// * `Ok(())` if the event was successfully processed
/// * `Err(anyhow::Error)` if the event opcode is invalid or event processing fails
///
/// # Event Routing
/// * `Error` events are routed to `error::handle_wl_display_error` for fatal error handling
/// * `DeleteId` events are routed to `delete_id::handle_wl_display_delete_id` for ID management
///
/// # Protocol Significance
/// The display object is fundamental to Wayland protocol operation:
/// - It's the first object clients interact with when connecting
/// - It provides access to the global registry via `get_registry`
/// - It enables synchronization between client and server via `sync`
/// - It manages object ID lifecycle and error reporting
///   Events on this object typically indicate critical connection state changes.
pub fn handle_wl_display_event(msg: WlMessage) -> anyhow::Result<()> {
    // Decode the event type from the message opcode
    let event_code: Event = msg.header.opcode.try_into()?;

    // Route the event to the appropriate handler based on type
    match event_code {
        Event::Error => error::handle_wl_display_error(&msg.data),
        Event::DeleteId => delete_id::handle_wl_display_delete_id(&msg.data),
    }
}
