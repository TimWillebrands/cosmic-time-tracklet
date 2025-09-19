# Scratchpad

- 2025-09-19 00:00 UTC — Read src/idle_monitor.rs, src/window.rs, Cargo.toml, README.md.
- 2025-09-19 00:00 UTC — Noted wayland-client = 0.29 and using libcosmic cctk wrappers.
- 2025-09-19 00:00 UTC — Found IdleMonitor uses ext_idle_notifier_v1 + wl_seat; dispatch only flush/dispatch_pending.
- 2025-09-19 00:00 UTC — Found callbacks in Window::init ignore returned App message; not sent to runtime.
- 2025-09-19 00:00 UTC — Suspect event loop starvation: no blocking dispatch, only dispatch_pending every 1s Tick.
- 2025-09-19 00:05 UTC — Added IdleMonitor::spawn to own queue and blocking_dispatch in bg thread.
- 2025-09-19 00:05 UTC — Bridged events via mpsc channel to Window; drain channel on Tick every 200ms.
- 2025-09-19 00:05 UTC — Removed direct wayland-client/protocols deps from Cargo.toml to avoid version mix.
- 2025-09-19 00:05 UTC — Recorded idle start timestamp for later prompt logic.
- 2025-09-19 00:12 UTC — Build link error: missing -lxkbcommon; need libxkbcommon-dev and a linker (cc or ld.lld).
- 2025-09-19 00:15 UTC — Created Cursor rule: .cursor/rules/project-structure.mdc (alwaysApply)
- 2025-09-19 00:16 UTC — Created Cursor rule: .cursor/rules/multi-instance-cosmic-applet.mdc (alwaysApply)
- 2025-09-19 00:16 UTC — Created Cursor rule: .cursor/rules/rust-style-for-cosmic.mdc (alwaysApply, globs=*.rs)
- 2025-09-19 00:22 UTC — Refined multi-instance rule: banned Wayland handles in Window; added logging, tick cadence, file path, UI style examples.
- 2025-09-19 00:23 UTC — Scoped rust-style rule to *.rs via globs (manual apply retained).
- 2025-09-19 00:24 UTC — Updated ARCHITECTURE.md to reflect background idle thread and Tick channel draining.
 - 2025-09-19 00:30 UTC — Silenced warnings: removed unused DispatchError import; deleted unused dispatch_events; underscored unused IdleMonitor fields; removed unused FetchError variant; renamed unused wl_seat dispatch param. Clean build with zero warnings.
 - 2025-09-19 00:40 UTC — Diagnosis: `GlobalList` owns `wl_registry` events; our `Dispatch<wl_registry, GlobalListContents>` won't fire. Need to bind `wl_seat`/`ext_idle_notifier_v1` via `GlobalList` after initial roundtrip and rely on `wl_pointer`/`wl_keyboard` and `ExtIdleNotificationV1` events for activity.
