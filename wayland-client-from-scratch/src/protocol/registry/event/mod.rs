use anyhow::anyhow;

use crate::protocol::message::WlMessage;

pub mod global;
pub mod global_remove;

/// Represents the event types that can be emitted by the Wayland registry object.
///
/// The Wayland registry is the singleton global registry object that advertises
/// available global objects to clients. These global objects represent actual
/// server resources (like input devices) or singleton objects providing extension
/// functionality. The registry emits events to notify clients of available globals
/// and their removal due to device hotplugs, reconfiguration, or other system events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// Announces the availability of a new global object.
    ///
    /// This event notifies the client that a global object with the given name is
    /// now available, implementing the specified version of the given interface.
    /// Clients typically respond by creating a proxy object using the bind request.
    ///
    /// # Event Arguments
    /// - `name`: Unique numeric identifier for the global object
    /// - `interface`: The interface type implemented by the object  
    /// - `version`: The interface version supported by the object
    Global = 0,

    /// Announces the removal of a previously advertised global object.
    ///
    /// This event notifies the client that the global identified by the given name
    /// is no longer available. If the client bound to this global, it should destroy
    /// the corresponding object. The object remains technically valid but requests
    /// to it will be ignored until destruction, preventing race conditions between
    /// global removal and pending client requests.
    ///
    /// # Event Arguments  
    /// - `name`: Numeric name of the global object being removed
    GlobalRemove = 1,
}

impl From<Event> for u16 {
    /// Converts a `WlRegistryEvent` variant to its corresponding protocol opcode.
    ///
    /// # Returns
    /// The numeric opcode value used in Wayland protocol messages for this event type.
    fn from(value: Event) -> u16 {
        value as u16
    }
}

impl TryFrom<u16> for Event {
    type Error = anyhow::Error;

    /// Attempts to convert a raw opcode value into a structured `WlRegistryEvent`.
    ///
    /// # Arguments
    /// * `value` - The opcode value from the message header
    ///
    /// # Returns
    /// * `Ok(WlRegistryEvent)` if the opcode corresponds to a known registry event type
    /// * `Err(anyhow::Error)` if the opcode is unrecognized
    ///
    /// # Protocol Context
    /// The registry object uses opcodes 0 for global announcements and 1 for global
    /// removal notifications as defined in the Wayland protocol specification.
    fn try_from(value: u16) -> anyhow::Result<Self> {
        match value {
            0 => Ok(Event::Global),
            1 => Ok(Event::GlobalRemove),
            _ => Err(anyhow!("Invalid wl_registry event opcode: {}", value)),
        }
    }
}

/// Dispatches incoming Wayland registry events to their appropriate handler functions.
///
/// This function serves as the main entry point for processing events targeted at
/// the registry object. It decodes the event type from the message opcode and routes
/// the message data to the corresponding event handler.
///
/// # Arguments
/// * `msg` - The complete Wayland message containing both header and payload data
///
/// # Returns
/// * `Ok(())` if the event was successfully processed
/// * `Err(anyhow::Error)` if the event opcode is invalid or event processing fails
///
/// # Event Routing
/// * `Global` events are routed to `global::handle_wl_registry_global`
/// * `GlobalRemove` events are routed to `global_remove::handle_wl_registry_global_remove`
///
/// # Protocol Behavior
/// When a client first creates a registry object, it receives an initial burst of
/// `Global` events for all currently available globals. The client can mark the end
/// of this initial burst by using `wl_display.sync` after calling `wl_display.get_registry`.
/// Subsequent global additions and removals are communicated via additional events.
pub fn handle_wl_registry_event(msg: WlMessage) -> anyhow::Result<()> {
    // Decode the event type from the message opcode
    let event_code: Event = msg.header.opcode.try_into()?;

    // Route the event to the appropriate handler based on type
    match event_code {
        Event::Global => global::handle_wl_registry_global(&msg.data),
        Event::GlobalRemove => global_remove::handle_wl_registry_global_remove(&msg.data),
    }
}
