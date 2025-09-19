/*
 * This guy knows the things:
 *  https://smithay.github.io/book/client/general/registry.html
 */
use cosmic::cctk::wayland_client::{
    globals::{registry_queue_init, GlobalList, GlobalListContents},
    protocol::{wl_keyboard, wl_pointer, wl_registry, wl_seat},
    Connection, Dispatch, EventQueue, QueueHandle,
};

use cosmic::cctk::wayland_protocols::ext::idle_notify::v1::client::{
    ext_idle_notification_v1::{Event, ExtIdleNotificationV1},
    ext_idle_notifier_v1::{Event as NotifyEvent, ExtIdleNotifierV1},
};
use std::error::Error;
use std::thread;

/// The IdleMonitor encapsulates the idle-notification object.
pub struct IdleMonitor {
    on_idle: Box<dyn FnMut() + Send + 'static>, // Changed to FnMut and Boxed
    on_resumed: Box<dyn FnMut() + Send + 'static>, // Changed to FnMut and Boxed
    idle_notifier: Option<ExtIdleNotifierV1>,
    idle_notification: Option<ExtIdleNotificationV1>, // Store this to keep it alive and receive events
    seat: Option<wl_seat::WlSeat>, // Option because seat might not be available immediately or at all
    pointer: Option<wl_pointer::WlPointer>,
    keyboard: Option<wl_keyboard::WlKeyboard>,
    idle_timeout_ms: u32,
    _queue: QueueHandle<IdleMonitor>,
    _globals: GlobalList, // Maintain registry lifetime
    _conn: Connection,    // Maintain connection lifetime
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

        // Initialize registry WITH OUR QUEUE HANDLE
        let (globals, mut event_queue) = registry_queue_init::<IdleMonitor>(&conn)
            .map_err(|e| format!("registryRegistry init failed: {}", e))?;

        let qh = event_queue.handle();
        //let globals_list = globals.contents().clone_list(); 
        //let registry = globals.registry();

        let mut state = Self {
            on_idle: Box::new(on_idle),       // Box the callback
            on_resumed: Box::new(on_resumed), // Box the callback
            idle_timeout_ms,
            idle_notifier: None,
            idle_notification: None,
            seat: None,
            pointer: None,
            keyboard: None,
            _queue: qh,
            _conn: conn,
            _globals: globals,
        };

        // Perform initial roundtrip to get registry events
        event_queue.roundtrip(&mut state)?;

        Ok((state, event_queue))
    }

    

    fn request_idle_notification(&mut self, qh: &QueueHandle<IdleMonitor>) {
        if let (Some(notifier), Some(seat)) = (self.idle_notifier.as_ref(), self.seat.as_ref()) {
            // Request idle notification only if both notifier and seat are available.
            // We use `idle_timeout_ms` from the struct.
            self.idle_notification =
                Some(notifier.get_idle_notification(self.idle_timeout_ms, seat, qh, ()));
        }
    }

    /// Spawns a background thread that owns the Wayland connection and event queue,
    /// and continuously dispatches events, invoking the provided callbacks.
    pub fn spawn<FIdle, FResumed>(
        idle_timeout_ms: u32,
        mut on_idle: FIdle,
        mut on_resumed: FResumed,
    ) -> std::io::Result<thread::JoinHandle<()>>
    where
        FIdle: FnMut() + Send + 'static,
        FResumed: FnMut() + Send + 'static,
    {
        thread::Builder::new()
            .name("idle-monitor-loop".into())
            .spawn(move || {
                // Build monitor and queue on this thread to satisfy thread-affinity.
                let (mut state, mut event_queue) = IdleMonitor::new(
                    idle_timeout_ms,
                    move || on_idle(),
                    move || on_resumed(),
                )
                .expect("Failed to initialize IdleMonitor");

                loop {
                    // Block until there are events, then dispatch.
                    if let Err(err) = event_queue.blocking_dispatch(&mut state) {
                        eprintln!("Wayland dispatch error: {}", err);
                        break;
                    }
                }
            })
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
        println!("wl_registry...?");
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            println!("wl_registry dispatch happened!!!");
            match interface.as_str() {
                "wl_seat" => {
                    // Bind to the wl_seat global
                    let seat = registry.bind::<wl_seat::WlSeat, _, _>(name, version, qh, ());
                    state.seat = Some(seat);
                    // Also bind pointer and keyboard to observe input events
                    if let Some(seat_ref) = state.seat.as_ref() {
                        state.pointer = Some(seat_ref.get_pointer(qh, ()));
                        state.keyboard = Some(seat_ref.get_keyboard(qh, ()));
                    }
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

impl Dispatch<wl_pointer::WlPointer, ()> for IdleMonitor {
    fn event(
        state: &mut IdleMonitor,
        _pointer: &wl_pointer::WlPointer,
        _event: cosmic::cctk::wayland_client::protocol::wl_pointer::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<IdleMonitor>,
    ) {
        // Any pointer activity should be treated as resume/user activity.
        (state.on_resumed)();
    }
}

impl Dispatch<wl_keyboard::WlKeyboard, ()> for IdleMonitor {
    fn event(
        state: &mut IdleMonitor,
        _keyboard: &wl_keyboard::WlKeyboard,
        _event: cosmic::cctk::wayland_client::protocol::wl_keyboard::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<IdleMonitor>,
    ) {
        // Any keyboard activity should be treated as resume/user activity.
        (state.on_resumed)();
    }
}
