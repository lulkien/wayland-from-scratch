use crate::{
    protocol::{
        WlObjectId,
        message::{WlMessage, WlMessageIter},
        registry::event::handle_wl_registry_event,
        types::{WlNewId, WlString},
    },
    wl_request_opcode, wl_request_param,
};

use super::event::handle_wl_display_event;

use std::{
    convert::TryInto,
    io::{Read, Write},
    os::unix::net::UnixStream,
};

use anyhow::anyhow;

wl_request_opcode! {
    /// Represents the request types that can be sent to the Wayland display object.
    ///
    /// The display object supports core protocol management requests that enable
    /// clients to synchronize with the server and discover available interfaces.
    Opcode {
        /// Creates a synchronization point with the compositor.
        /// Returns a callback object that fires when all previous requests have been processed.
        Sync = 0,

        /// Retrieves the global registry object for interface discovery.
        /// This is typically the first request clients make after connecting.
        GetRegistry = 1,
    }
}

wl_request_param! {
    TestParam {
        string: WlString,
    }
}

wl_request_param! {
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
    SyncParam {
        /// The object ID to assign to the newly created wl_callback object.
        /// The compositor will destroy this object after firing the callback.
        new_id: WlNewId,
    }
}

wl_request_param! {
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
    RequestParam {
        /// The object ID to assign to the newly created wl_registry object.
        /// This registry will receive global advertisement events from the compositor.
        new_id: WlNewId,
    }
}

/// Sends a `wl_display.get_registry` request to the compositor and processes the response.
///
/// This function implements the core bootstrap sequence for Wayland clients. It requests
/// the global registry object from the display, which provides access to all available
/// global interfaces offered by the compositor.
///
/// # Arguments
/// * `stream` - The Unix socket stream connected to the Wayland compositor
/// * `new_id` - The object ID to assign to the newly created registry object
///
/// # Returns
/// * `Ok(())` if the request was successfully sent and all response events processed
/// * `Err(anyhow::Error)` if any I/O operation fails or protocol errors occur
///
/// # Protocol Sequence
/// 1. Serializes the `get_registry` request with the specified new object ID
/// 2. Sends the request message to the compositor via the Unix socket
/// 3. Reads the compositor's response (typically a burst of global advertisement events)
/// 4. Processes all incoming events, routing them to appropriate handlers
///
/// # Expected Response Events
/// After a successful `get_registry` request, the compositor will typically send:
/// - A `wl_registry.global` event for each currently available global object
/// - Potentially other protocol management events on the display object
/// - The initial event burst concludes when all current globals have been advertised
///
/// # Resource Management
/// According to the Wayland specification, the server-side resources consumed by
/// `get_registry` can only be released when the client disconnects. Clients should
/// invoke this request infrequently to avoid wasting server memory.
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
pub fn get_registry(stream: &mut UnixStream, new_id: WlNewId) -> anyhow::Result<()> {
    // Serialize get_registry request parameters into protocol format
    let register_data: Vec<u8> = RequestParam::new(new_id).into();

    // Construct the complete Wayland protocol message
    let message = WlMessage::new(
        WlObjectId::Display.into(),
        Opcode::GetRegistry.into(),
        &register_data,
    );

    // Send the message to the compositor over the Unix socket
    let write_buf: Vec<u8> = message.into();
    let written_len = stream.write(&write_buf)?;

    // Verify the entire message was transmitted successfully
    if write_buf.len() != written_len {
        return Err(anyhow!(
            "Failed to write complete wl_display_get_registry message: expected {} bytes, wrote {} bytes",
            write_buf.len(),
            written_len
        ));
    }

    // Read compositor response containing events and potential errors
    // Uses a fixed buffer size that should accommodate typical initial global bursts
    let mut read_buf: [u8; 4096] = [0; 4096];
    let read_len = stream.read(&mut read_buf)?;

    // Process all incoming events using a message iterator
    // The iterator handles message boundaries and buffer management
    let mut event_iter = WlMessageIter::new(read_buf[..read_len].into());
    loop {
        let event = event_iter.next();
        if event.is_none() {
            break;
        }

        let event = event.unwrap();
        let event_object: WlObjectId = event.header.object_id.try_into()?;

        // Route events to appropriate handlers based on the target object type
        match event_object {
            WlObjectId::Display => {
                // Handle display-level events (errors, sync callbacks, etc.)
                handle_wl_display_event(event)?
            }
            WlObjectId::Registry => {
                // Handle registry events (global advertisements, removals)
                // This is the primary expected response to get_registry
                handle_wl_registry_event(event)?
            }
            _ => {
                // Unexpected object type - this may indicate a protocol violation
                // or an extension interface we haven't implemented yet
                unimplemented!(
                    "Unexpected object type in get_registry response: {:?}",
                    event_object as u32
                )
            }
        }
    }

    Ok(())
}
