## Overview

This project is a COSMIC desktop applet that integrates with Clockify to start/stop and display the current time entry. The UI is built with `libcosmic` (COSMIC’s Iced-based toolkit) and runs as an applet process. It also integrates with Wayland’s `ext_idle_notifier_v1` protocol to observe user idle/resume events.

- **Applet ID**: `com.tim_willebrands.time_tracklet`
- **Entrypoint**: `src/main.rs` runs the applet model `Window` via `cosmic::applet::run`.
- **Primary modules**:
  - `src/window.rs`: Applet model, UI, message/update loop, and Clockify interactions.
  - `src/idle_monitor.rs`: Wayland idle detection (idle/resume) using `ext_idle_notifier_v1`.

## Key Dependencies

- **`libcosmic`**: UI toolkit and applet framework (features: `applet`, `wayland`, `tokio`).
- **`wayland-client` / `wayland-protocols`**: Direct Wayland integration for idle notifications.
- **`once_cell`**: Lazily initialized widget IDs.

See `Cargo.toml` for complete versions and feature flags.

## High-Level Flow

1. Process starts via `cosmic::applet::run::<Window>(())`.
2. `Window::init`:
   - Fetches the current Clockify entry (`fetch_current_entry`).
   - Initializes Wayland idle monitoring (`IdleMonitor::new`) and stores both the monitor state and an `EventQueue`.
   - Prepares initial UI model state.
3. A 1-second subscription (`time::every(1s)`) emits `Message::Tick`:
   - The applet flushes and dispatches Wayland events via the stored `EventQueue` to drive idle/resume callbacks.
4. User actions (button clicks, popup form submit) dispatch messages that update model state and invoke Clockify CLI commands.

## Modules and Responsibilities

### `src/main.rs`
- Declares modules `window` and `idle_monitor`.
- Invokes `cosmic::applet::run::<Window>`.

### `src/window.rs`
- Defines the applet model `Window`:
  - COSMIC `Core` and popup `Id` management.
  - Current task state (`TimeEntry`).
  - Popup form state, debug text (currently the process `PATH`).
  - Idle integration: `idle_monitor: IdleMonitor` and `event_queue: EventQueue<IdleMonitor>`.
- `Message` enum covers UI events, task actions, and idle/resume/tick.
- `impl Application for Window`:
  - `init`:
    - Preloads the current task with `clockify-cli show`.
    - Sets up `IdleMonitor` with idle/resume callbacks.
  - `subscription`: emits `Tick` every second.
  - `update`: handles:
    - Popup open/close using COSMIC popup commands.
    - `StartEntry`/`StopEntry` by spawning `clockify-cli`.
    - `RefreshEntry` to re-fetch the current entry.
    - `UserIdle`/`UserResume` (currently log-only).
    - `Tick` to dispatch Wayland events through `IdleMonitor`.
  - `view`: applet button in the panel displays the current task string, uses `autosize`.
  - `view_window`: popup content with:
    - Text input to start/switch task (submits on Enter).
    - Buttons to Stop and Refresh.
    - Debug info text.
- `fetch_current_entry`:
  - Executes `clockify-cli show --format "{{.Description}}"` and maps output to `TimeEntry`.

### `src/idle_monitor.rs`
- Encapsulates Wayland idle detection:
  - Connects to Wayland (`Connection::connect_to_env`).
  - Initializes registry/queue (`registry_queue_init`).
  - Handles global registry events and binds `wl_seat` and `ext_idle_notifier_v1`.
  - Requests an idle notification with configured timeout (`idle_timeout_ms`).
  - Dispatches protocol events; translates `Idled`/`Resumed` into provided callbacks.
- Returns both the monitor state and its `EventQueue` to the applet, which manually flushes/dispatches each tick.

## UI Architecture

- **Panel button (collapsed state)**: shows current task string, sized via `autosize` to panel height.
- **Popup window**:
  - Input: description field bound to `form_description` and submitted as `StartEntry`.
  - Buttons: `Stop` and `Refresh`.
  - Diagnostics: `debug_text` shows selected environment info.

## Message and Update Flow

- **Popup**:
  - `Message::TogglePopup` creates/destroys popup via `get_popup` / `destroy_popup`.
  - `Message::PopupClosed(Id)` clears internal popup state if IDs match.
- **Clockify actions**:
  - `StartEntry`: runs `clockify-cli in -p <project-id> -d <desc>`, optimistically updates UI, closes popup.
  - `StopEntry`: runs `clockify-cli out`, clears current entry.
  - `RefreshEntry`: re-runs `fetch_current_entry`.
- **Idle**:
  - `UserIdle`/`UserResume`: currently print to stdout; designed for future UX (e.g., confirm resume/stop).
- **Tick**:
  - Flushes and dispatches Wayland events via `IdleMonitor::dispatch_events` to drive idle notifications.

## External Integrations

- **Clockify CLI** (`clockify-cli`):
  - Invoked via `std::process::Command`.
  - `PATH` is augmented to include Homebrew (`/home/linuxbrew/.linuxbrew/bin`).
  - Hard-coded default project ID is used for `StartEntry`.

- **Wayland Idle Protocol** (`ext_idle_notifier_v1`):
  - Requires compositor support for `ext_idle_notifier_v1`.
  - Binds `wl_seat` and the notifier; requests notifications with a timeout.

## Error Handling and Logging

- `TimeEntry` tracks:
  - `NoEntry`, `Entry(String)`, `FetchError(String)`, `CliError(String)`.
  - `fetch_current_entry` returns `CliError` on spawn failures; non-zero exit maps to `NoEntry`.
- Many actions log to stdout/stderr. There is not yet a surfaced UI error state beyond the `TimeEntry` string and the debug section in the popup.

## Build & Run

- Build with `cargo build --release`.
- Install/register the desktop entry (`cosmic-time-tracklet.desktop`) and add the applet in COSMIC Settings.
- Binary name is `cosmic-applet-clockify` (as defined by Cargo package name).

## Assumptions & Limitations

- Uses external `clockify-cli`; no direct HTTP/API client is embedded.
- Hard-coded default project ID for `StartEntry`.
- Synchronous `Command::output()` calls; no async process handling.
- Idle notifier requires Wayland compositor support for `ext_idle_notifier_v1`.
- Minimal user feedback on CLI failures.

## Extension Points

- Persist and configure Clockify workspace/project/user preferences.
- Replace hard-coded project ID with settings or selection UI.
- Async command execution with progress/error surface in the UI.
- Add confirmation UI on `UserResume` to continue current task or stop/switch.
- Enrich panel button with duration/billable indicators and status color.
- Add localization and robust error messages.
