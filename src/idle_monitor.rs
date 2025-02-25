/*
 * This guy knows the things:
 *  https://smithay.github.io/book/client/general/registry.html
 */
use cosmic::cctk::wayland_client::{
    globals::{registry_queue_init, GlobalListContents},
    protocol::{wl_registry, wl_seat},
    Connection, Dispatch, DispatchError, EventQueue, QueueHandle,
};
use cosmic::cctk::wayland_protocols::ext::idle_notify::v1::client::{
    ext_idle_notification_v1::{Event, ExtIdleNotificationV1},
    ext_idle_notifier_v1::{Event as NotifyEvent, ExtIdleNotifierV1},
};
use std::error::Error;

/// The IdleMonitor encapsulates the idle-notification object.
pub struct IdleMonitor {
    on_idle: Box<dyn FnMut() + Send + 'static>, // Changed to FnMut and Boxed
    on_resumed: Box<dyn FnMut() + Send + 'static>, // Changed to FnMut and Boxed
    idle_notifier: Option<ExtIdleNotifierV1>,
    idle_notification: Option<ExtIdleNotificationV1>, // Store this to keep it alive and receive events
    seat: Option<wl_seat::WlSeat>, // Option because seat might not be available immediately or at all
    idle_timeout_ms: u32,
    queue: QueueHandle<IdleMonitor>
}

impl IdleMonitor {
    /// Creates a new IdleMonitor.
    ///
    /// Returns an `IdleMonitor` that will notify when the user has been idle for 5 minutes.
    pub fn new<FIdle, FResumed>(
        idle_timeout_ms: u32,
        on_idle: FIdle,
        on_resumed: FResumed,
    ) -> Result<(Self, EventQueue<IdleMonitor>), Box<dyn Error>>
    where
        FIdle: FnMut() + Send + 'static,    // Callback type constraint
        FResumed: FnMut() + Send + 'static, // Callback type constraint
    {
        // Connect to the Wayland display.
        let conn = Connection::connect_to_env()?;
        let queue = conn.new_event_queue::<IdleMonitor>();
        let ret = registry_queue_init::<IdleMonitor>(&conn);
        let (_globals, _qh) = ret.map_err(|e| format!("Failed to initialize registry: {}", e))?;
        let qh = queue.handle(); // Get the handle here
                                 //
        let s = Self {
            on_idle: Box::new(on_idle),       // Box the callback
            on_resumed: Box::new(on_resumed), // Box the callback
            idle_timeout_ms,
            idle_notifier: None,
            idle_notification: None,
            seat: None,
            queue: qh,
        };

        Ok((s, queue))
    }

    pub fn dispatch_events(&mut self, queue: &mut EventQueue<IdleMonitor>) -> Result<usize, DispatchError> {
        queue.dispatch_pending(self)
    }

    fn request_idle_notification(&mut self, qh: &QueueHandle<IdleMonitor>) {
        if let (Some(notifier), Some(seat)) = (self.idle_notifier.as_ref(), self.seat.as_ref()) {
            // Request idle notification only if both notifier and seat are available.
            // We use `idle_timeout_ms` from the struct.
            self.idle_notification =
                Some(notifier.get_idle_notification(self.idle_timeout_ms, seat, qh, ()));
        }
    }
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for IdleMonitor {
    fn event(
        state: &mut IdleMonitor,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _data: &GlobalListContents,
        _conn: &Connection,
        qh: &QueueHandle<IdleMonitor>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            println!("wl_registry! happened!!!");
            match interface.as_str() {
                "wl_seat" => {
                    // Bind to the wl_seat global
                    let seat = registry.bind::<wl_seat::WlSeat, _, _>(name, version, qh, ());
                    state.seat = Some(seat);
                    // Now that we have a seat, check if we also have the idle notifier and can set up notification
                    state.request_idle_notification(qh);
                }
                "ext_idle_notifier_v1" => {
                    if version >= 1 {
                        // Check version if necessary, version 1 is likely what you want
                        let idle_notifier =
                            registry.bind::<ExtIdleNotifierV1, _, _>(name, version, qh, ());
                        state.idle_notifier = Some(idle_notifier);
                        // Now that we have the idle notifier, check if we also have a seat and can set up notification
                        state.request_idle_notification(qh);
                    } else {
                        eprintln!(
                            "ext_idle_notifier_v1 global found but version is too old: {}",
                            version
                        );
                        // Handle older version or ignore
                    }
                }
                _ => {
                    // Ignore other globals
                }
            }
        }
    }
}

impl Dispatch<ExtIdleNotificationV1, ()> for IdleMonitor {
    fn event(
        state: &mut IdleMonitor,
        _proxy: &ExtIdleNotificationV1,
        event: Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<IdleMonitor>,
    ) {
        match event {
            Event::Idled => (state.on_idle)(),
            Event::Resumed => (state.on_resumed)(),
            _ => {
                println!("Unknown ext_idle_notification event: {:?}", event);
            }
        }
    }
}

impl Dispatch<ExtIdleNotifierV1, ()> for IdleMonitor {
    fn event(
        _state: &mut IdleMonitor,
        _idle_notifier: &ExtIdleNotifierV1,
        _event: NotifyEvent,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<IdleMonitor>,
    ) {
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for IdleMonitor {
    fn event(
        _state: &mut IdleMonitor,
        _seat: &wl_seat::WlSeat,
        _event: cosmic::cctk::wayland_client::protocol::wl_seat::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<IdleMonitor>,
    ) {
    }
}
