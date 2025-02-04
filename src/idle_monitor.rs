use std::error::Error;
use wayland_client::{
    protocol::wl_seat::WlSeat, 
    Display, GlobalManager, 
};
use wayland_protocols::ext::idle_notify::v1::client::{
    ext_idle_notifier_v1::ExtIdleNotifierV1,
    ext_idle_notification_v1::{self, ExtIdleNotificationV1},
};

/// The IdleMonitor encapsulates the idle-notification object.
pub struct IdleMonitor {
    idle_notification: ExtIdleNotificationV1,
}

impl IdleMonitor {
    /// Creates a new IdleMonitor.
    ///
    /// * `display` - A reference to the Wayland display.
    /// * `qh` - A reference to a QueueHandle.
    /// * `globals` - A reference to a GlobalManager that has already performed an initial roundtrip.
    ///
    /// Returns an `IdleMonitor` that will notify when the user has been idle for 5 minutes.
    pub fn new(
        display: &Display,
        globals: &GlobalManager,
    ) -> Result<Self, Box<dyn Error>> {
        // Connect to the Wayland display.
        let display = Display::connect_to_env()?;
        
        // Create an event queue.
        let mut event_queue = display.create_event_queue();
        let qh = event_queue.handle();

        // Bind to the ext_idle_notify global. This returns an error if not available.
        let idle_notifier = globals
            .instantiate_exact::<ExtIdleNotifierV1>(1)
            .map_err(|_| "ext_idle_notify_v1 is not available on this compositor.")?;
        
        // Retrieve a wl_seat; this example uses the first available seat.
        let seat = globals.instantiate_exact::<WlSeat>(1)?;
        
        // Create an idle notification with a 5-minute timeout (300000 ms).
        let idle_notification = idle_notifier.get_idle_notification(300000, &seat, qh, ());
        
        Ok(Self { idle_notification })
    }

    /// Runs a blocking dispatch loop for idle events.
    ///
    /// Call this method to process incoming idle events. Depending on your
    /// application, you might want to integrate this dispatch into your main
    /// event loop or run it on a separate thread.
    pub fn run(&self, event_queue: &mut wayland_client::EventQueue) -> Result<(), Box<dyn Error>> {
        loop {
            event_queue.blocking_dispatch(&mut IdleHandler)?;
        }
    }
}

///// A simple dispatch handler for idle notifications.
//struct IdleHandler;
//
//impl<D> Dispatch<ExtIdleNotificationV1, ()> for IdleHandler {
//    fn event(
//        _state: &mut D,
//        _proxy: &ExtIdleNotificationV1,
//        event: ext_idle_notification_v1::Event,
//        _data: &(),
//        _conn: &wayland_client::Connection,
//        _qh: &QueueHandle<D>,
//    ) {
//        match event {
//            ext_idle_notification_v1::Event::Idled => {
//                println!("User has been idle for at least 5 minutes.");
//            }
//            ext_idle_notification_v1::Event::Resumed => {
//                println!("User activity resumed.");
//            }
//            _ => {}
//        }
//    }
//}
