/*
 * This guy knows the things:
 *  https://smithay.github.io/book/client/general/registry.html
 */
use cosmic::cctk::wayland_client::{
    globals::{registry_queue_init, Global, GlobalListContents},
    protocol::wl_registry,
    Connection, Dispatch, QueueHandle,
};
use cosmic::cctk::wayland_protocols::ext::idle_notify::v1::client::{
    ext_idle_notification_v1::{self, ExtIdleNotificationV1},
    ext_idle_notifier_v1::ExtIdleNotifierV1,
};
//use wayland_protocols::ext::idle_notify::v1::client::ext_idle_notification_v1::Event;
use std::error::Error;

/// The IdleMonitor encapsulates the idle-notification object.
pub struct IdleMonitor {
    //idle_notification: ExtIdleNotificationV1, // Use Main<ExtIdleNotificationV1>
}

impl IdleMonitor {
    /// Creates a new IdleMonitor.
    ///
    /// Returns an `IdleMonitor` that will notify when the user has been idle for 5 minutes.
    pub fn new() -> Result<Self, Box<dyn Error>> {
        // Connect to the Wayland display.
        let conn = Connection::connect_to_env()?;
        let queue = conn.new_event_queue::<IdleMonitor>();
        let qh = queue.handle();
        let (globals, queue) = registry_queue_init::<IdleMonitor>(&conn).unwrap();

        Ok(Self { })
    }
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for IdleMonitor {
    fn event(
        state: &mut IdleMonitor,
        proxy: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        // The `GlobalListContents` is a container with an up-to-date list of
        // the currently existing globals
        data: &GlobalListContents,
        conn: &Connection,
        qhandle: &QueueHandle<IdleMonitor>,
    ) {
    }
}
