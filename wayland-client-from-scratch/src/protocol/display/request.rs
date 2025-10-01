use anyhow::anyhow;

/// Represents the request types that can be sent to the Wayland display object.
///
/// The display object supports core protocol management requests that enable
/// clients to synchronize with the server and discover available interfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WlDisplayRequest {
    /// Creates a synchronization point with the compositor.
    /// Returns a callback object that fires when all previous requests have been processed.
    Sync = 0,

    /// Retrieves the global registry object for interface discovery.
    /// This is typically the first request clients make after connecting.
    GetRegistry = 1,
}

impl From<WlDisplayRequest> for u16 {
    /// Converts a `WlDisplayRequest` variant to its corresponding protocol opcode.
    ///
    /// # Returns
    /// The numeric opcode value used in Wayland protocol messages for this request type.
    fn from(request: WlDisplayRequest) -> u16 {
        request as u16
    }
}

/// Parameters for the `wl_display.sync` request.
///
/// This request creates a synchronization barrier between client and server.
/// The compositor will emit a 'done' event on the returned callback object
/// when all previous requests have been processed, ensuring ordered execution.
///
/// # Specification Reference
/// ```xml
/// <request name="sync">
///   <description summary="asynchronous roundtrip">
///     The sync request asks the server to emit the 'done' event
///     on the returned wl_callback object. Since requests are
///     handled in-order and events are delivered in-order, this can
///     be used as a barrier to ensure all previous requests and the
///     resulting events have been handled.
///   </description>
///   <arg name="callback" type="new_id" interface="wl_callback"
///        summary="callback object for the sync request"/>
/// </request>
/// ```
pub struct WlDisplaySyncParam {
    /// The object ID to assign to the newly created wl_callback object.
    /// The compositor will destroy this object after firing the callback.
    new_id: u32,
}

#[allow(unused)]
impl WlDisplaySyncParam {
    /// Creates new synchronization parameters with the specified callback object ID.
    ///
    /// # Arguments
    /// * `new_id` - The object ID for the new wl_callback object
    pub(super) fn new(new_id: u32) -> Self {
        Self { new_id }
    }

    /// Returns the object ID assigned to the synchronization callback.
    pub(super) fn new_id(&self) -> u32 {
        self.new_id
    }
}

impl From<WlDisplaySyncParam> for Vec<u8> {
    /// Serializes the synchronization parameters into the Wayland wire format.
    ///
    /// # Returns
    /// A 4-byte vector containing the new_id in native endianness.
    ///
    /// # Wire Format
    /// The sync request carries a single argument:
    /// - Bytes 0-3: `new_id` (u32) - The ID for the new callback object
    fn from(args: WlDisplaySyncParam) -> Vec<u8> {
        args.new_id.to_ne_bytes().to_vec()
    }
}

impl TryFrom<&[u8]> for WlDisplaySyncParam {
    type Error = anyhow::Error;

    /// Deserializes synchronization parameters from the Wayland wire format.
    ///
    /// # Arguments
    /// * `buf` - The byte buffer containing serialized parameter data
    ///
    /// # Returns
    /// * `Ok(WlDisplaySyncParam)` if the buffer contains valid parameter data
    /// * `Err(anyhow::Error)` if the buffer is malformed or incomplete
    ///
    /// # Buffer Requirements
    /// The buffer must contain exactly 4 bytes representing the new_id value.
    fn try_from(buf: &[u8]) -> anyhow::Result<WlDisplaySyncParam> {
        if buf.len() != 4 {
            return Err(anyhow!(
                "Invalid buffer length for WlDisplaySyncParam: expected 4 bytes, got {}",
                buf.len()
            ));
        }

        Ok(WlDisplaySyncParam {
            new_id: u32::from_ne_bytes(buf.try_into()?),
        })
    }
}

/// Parameters for the `wl_display.get_registry` request.
///
/// This request creates a registry object that allows the client to discover
/// and bind to global objects available from the compositor. It is the
/// fundamental mechanism for interface discovery in the Wayland protocol.
///
/// # Specification Reference
/// ```xml
/// <request name="get_registry">
///   <description summary="get global registry object">
///     This request creates a registry object that allows the client
///     to list and bind the global objects available from the
///     compositor.
///   </description>
///   <arg name="registry" type="new_id" interface="wl_registry"
///        summary="global registry object"/>
/// </request>
/// ```
pub struct WlDisplayGetRegisterParam {
    /// The object ID to assign to the newly created wl_registry object.
    /// This registry will receive global advertisement events from the compositor.
    new_id: u32,
}

#[allow(unused)]
impl WlDisplayGetRegisterParam {
    /// Creates new registry parameters with the specified registry object ID.
    ///
    /// # Arguments
    /// * `new_id` - The object ID for the new wl_registry object
    pub(super) fn new(new_id: u32) -> Self {
        Self { new_id }
    }

    /// Returns the object ID assigned to the registry object.
    pub(super) fn new_id(&self) -> u32 {
        self.new_id
    }
}

impl From<WlDisplayGetRegisterParam> for Vec<u8> {
    /// Serializes the registry parameters into the Wayland wire format.
    ///
    /// # Returns
    /// A 4-byte vector containing the new_id in native endianness.
    ///
    /// # Wire Format
    /// The get_registry request carries a single argument:
    /// - Bytes 0-3: `new_id` (u32) - The ID for the new registry object
    fn from(args: WlDisplayGetRegisterParam) -> Vec<u8> {
        args.new_id.to_ne_bytes().to_vec()
    }
}

impl TryFrom<&[u8]> for WlDisplayGetRegisterParam {
    type Error = anyhow::Error;

    /// Deserializes registry parameters from the Wayland wire format.
    ///
    /// # Arguments
    /// * `buf` - The byte buffer containing serialized parameter data
    ///
    /// # Returns
    /// * `Ok(WlDisplayGetRegisterParam)` if the buffer contains valid parameter data
    /// * `Err(anyhow::Error)` if the buffer is malformed or incomplete
    ///
    /// # Buffer Requirements
    /// The buffer must contain exactly 4 bytes representing the new_id value.
    fn try_from(buf: &[u8]) -> anyhow::Result<WlDisplayGetRegisterParam> {
        if buf.len() != 4 {
            return Err(anyhow!(
                "Invalid buffer length for WlDisplayGetRegisterParam: expected 4 bytes, got {}",
                buf.len()
            ));
        }

        Ok(WlDisplayGetRegisterParam {
            new_id: u32::from_ne_bytes(buf.try_into()?),
        })
    }
}
