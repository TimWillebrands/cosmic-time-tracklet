# Scratchpad

- 2025-09-19 00:00 UTC — Read src/idle_monitor.rs, src/window.rs, Cargo.toml, README.md.
- 2025-09-19 00:00 UTC — Noted wayland-client = 0.29 and using libcosmic cctk wrappers.
- 2025-09-19 00:00 UTC — Found IdleMonitor uses ext_idle_notifier_v1 + wl_seat; dispatch only flush/dispatch_pending.
- 2025-09-19 00:00 UTC — Found callbacks in Window::init ignore returned App message; not sent to runtime.
- 2025-09-19 00:00 UTC — Suspect event loop starvation: no blocking dispatch, only dispatch_pending every 1s Tick.
- 2025-09-19 00:00 UTC — Plan to fix: send messages via core.shell.publish(App(Message::...)), ensure queue dispatches (roundtrip/dispatch) and track idle/resume.
