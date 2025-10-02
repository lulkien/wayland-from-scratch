use std::fmt::Display;

use crate::protocol::types::{WL_TYPE_UINT_LEN, WlString, WlUInt};
use anyhow::anyhow;

/// Represents a global object advertisement from the Wayland registry.
///
/// This structure contains the complete information about a global object
/// that is available for binding by the client. Each global represents
/// a specific interface implementation provided by the compositor.
///
/// # Specification Reference
/// ```xml
/// <event name="global">
///   <description summary="announce global object">
///     Notify the client of global objects.
///     The event notifies the client that a global object with
///     the given name is now available, and it implements the
///     given version of the given interface.
///   </description>
///   <arg name="name" type="uint" summary="numeric name of the global object"/>
///   <arg name="interface" type="string" summary="interface implemented by the object"/>
///   <arg name="version" type="uint" summary="interface version"/>
/// </event>
/// ```
pub struct Global {
    /// The unique numeric identifier for this global object.
    ///
    /// This name is used when binding to the global object via the `bind` request.
    /// Each global object has a unique name within the registry session.
    pub name: WlUInt,

    /// The interface type implemented by this global object.
    ///
    /// This string identifies the specific Wayland interface (e.g., "wl_compositor",
    /// "wl_seat") that this global object provides. Clients use this to determine
    /// what functionality is available and how to interact with the object.
    pub interface: WlString,

    /// The version number of the interface implementation.
    ///
    /// Wayland interfaces are versioned to allow for protocol evolution. Higher
    /// versions may introduce new requests, events, or enum values while maintaining
    /// backward compatibility. Clients should check this version to determine which
    /// interface features are available.
    pub version: WlUInt,
}

impl TryFrom<&[u8]> for Global {
    type Error = anyhow::Error;

    /// Deserializes a `wl_registry.global` event from the Wayland wire format.
    ///
    /// Parses the binary buffer according to the `wl_registry.global` event specification:
    /// - 32-bit unsigned integer for the global name
    /// - Wayland string for the interface name
    /// - 32-bit unsigned integer for the interface version
    ///
    /// # Arguments
    /// * `buf` - The byte buffer containing the serialized global event data
    ///
    /// # Returns
    /// * `Ok(WlRegistryGlobal)` if the buffer contains valid global event data
    /// * `Err(anyhow::Error)` if the buffer is malformed or incomplete
    ///
    /// # Buffer Layout
    /// The global event data is structured as:
    /// - Bytes 0-3: `name` (u32) - Unique numeric identifier for the global
    /// - Bytes 4+: `interface` (WlString) - Interface type name with length prefix
    /// - Bytes 4+interface.buffer_len(): `version` (u32) - Interface version number
    ///
    /// # Errors
    /// Returns an error if:
    /// - Buffer is too short for the name field (less than 4 bytes)
    /// - Buffer is too short for the interface string parsing
    /// - Buffer is too short for the version field after parsing the interface
    /// - The interface string contains invalid data or missing NUL terminator
    fn try_from(buf: &[u8]) -> anyhow::Result<Global> {
        // Extract name(u32) from buffer - the unique numeric identifier
        if buf.len() < WL_TYPE_UINT_LEN {
            return Err(anyhow!(
                "Buffer too short for WlRegistryGlobal name: expected {} bytes, got {}",
                WL_TYPE_UINT_LEN,
                buf.len()
            ));
        }
        let name = WlUInt::from_bytes(buf[..WL_TYPE_UINT_LEN].try_into()?);

        // Extract interface(WlString) from buffer - the interface type name
        let interface_start_pos = WL_TYPE_UINT_LEN;
        let interface: WlString = buf[interface_start_pos..].try_into()?;

        // Extract version(u32) from buffer - the interface version number
        let version_start_pos = interface_start_pos + interface.buffer_size();
        let version_end_pos = version_start_pos + WL_TYPE_UINT_LEN;
        if buf.len() < version_end_pos {
            return Err(anyhow!(
                "Buffer too short for WlRegistryGlobal version: expected {} bytes, got {}",
                version_end_pos,
                buf.len()
            ));
        }
        let version = WlUInt::from_bytes(buf[version_start_pos..version_end_pos].try_into()?);

        Ok(Global {
            name,
            interface,
            version,
        })
    }
}

impl Display for Global {
    /// Formats the global object information for human-readable display.
    ///
    /// Provides a structured view of the global object advertisement including
    /// its numeric name, interface type, and version number. This is particularly
    /// useful for debugging and logging during protocol discovery.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "WlRegistryGlobal {{ name: {}, interface: {}, version: {} }}",
            self.name, self.interface, self.version
        )
    }
}

/// Handles a `wl_registry.global` event announcing available global objects.
///
/// This function processes global advertisement events from the registry,
/// which notify the client about interfaces that are available for binding.
/// These events are typically received in an initial burst when the registry
/// is first created, followed by additional events as globals are added or
/// removed during the session.
///
/// # Arguments
/// * `buf` - The raw byte buffer containing the global event data
///
/// # Returns
/// * `Ok(())` if the event was successfully parsed and processed
/// * `Err(anyhow::Error)` if the event data is malformed or cannot be parsed
///
/// # Protocol Behavior
/// - Global events are emitted for each available global when the registry is created
/// - Additional global events may be sent during the session for hot-plugged devices
/// - Clients typically respond to these events by binding to globals they need
/// - The initial event burst can be concluded with a `wl_display.sync` request
///
/// # Typical Usage
/// Clients use this information to:
/// - Identify available compositor functionality
/// - Determine which interfaces to bind based on application needs
/// - Check interface versions to use appropriate feature sets
/// - Track available resources for dynamic environments
pub(super) fn handle_wl_registry_global(buf: &[u8]) -> anyhow::Result<()> {
    let global: Global = buf.try_into()?;

    println!("{global}");

    Ok(())
}
