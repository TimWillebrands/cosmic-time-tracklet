// Mandatory COSMIC imports
use cosmic::app::Core;
use cosmic::iced::{
    platform_specific::shell::commands::popup::{destroy_popup, get_popup},
    widget::row,
    window::Id,
    Limits, Task,
};
use cosmic::iced::{Alignment, Length};
use cosmic::iced_runtime::core::window;
use cosmic::Element;
// Widgets we're going to use
use cosmic::widget::{
    autosize, button, container, list_column, settings, text, text_input, vertical_space,
};
use once_cell::sync::Lazy;
use std::process::Command;
use std::{env, fmt};

// Every COSMIC Application and Applet MUST have an ID
const ID: &str = "com.tim_willebrands.time_tracklet";

static AUTOSIZE_MAIN_ID: Lazy<cosmic::widget::Id> =
    Lazy::new(|| cosmic::widget::Id::new("autosize-main-tracklet"));

/*
*  Every COSMIC model must be a struct data type.
*  Mandatory fields for a COSMIC Applet are core and popup.
*  Core is the core settings that allow it to interact with COSMIC
*  and popup, as you'll see later, is the field that allows us to open
*  and close the applet.
*
*  Next we have our custom field that we will manipulate the value of based
*  on the message we send.
*/
pub struct Window {
    core: Core,
    popup: Option<Id>,
    task_title: TimeEntry,
    form_description: Option<String>,
    debug_text: Option<String>,
}
/*
*  Define our error types. These may be customized for our error handling cases.
*  Now we will be able to write our own errors, defer to an underlying error
*  implementation, or do something in between.
*/
#[derive(Debug, Clone)]
enum TimeEntry {
    NoEntry,
    Entry(String),
    FetchError(String),
    CliError(String),
}

impl fmt::Display for TimeEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match self {
            TimeEntry::NoEntry => "- NO CURRENT TASK -".to_string(),
            TimeEntry::Entry(desc) => format!("[Task] {}", desc),
            TimeEntry::FetchError(err) => format!("[FetchErr] {}", err),
            TimeEntry::CliError(err) => format!("[CliErr] {}", err),
        };
        write!(f, "{}", msg)
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    TogglePopup,     // Mandatory for open and close the applet
    PopupClosed(Id), // Mandatory for the applet to know if it's been closed

    StopEntry,
    StartEntry,
    UpdateFormDesc(String),
    RefreshEntry,
}

impl cosmic::Application for Window {
    /*
     *  Executors are a mandatory thing for both COSMIC Applications and Applets.
     *  They're basically what allows for multi-threaded async operations for things that
     *  may take too long and block the thread the GUI is running on. This is also where
     *  Tasks take place.
     */
    type Executor = cosmic::SingleThreadExecutor;
    type Flags = (); // Honestly not sure what these are for.
    type Message = Message; // These are setting the application messages to our Message enum
    const APP_ID: &'static str = ID; // This is where we set our const above to the actual ID

    // Setup the immutable core functionality.
    fn core(&self) -> &Core {
        &self.core
    }

    // Set up the mutable core functionality.
    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    // Initialize the applet
    /*
     *  The parameters are the Core and flags (again not sure what to do with these).
     *  The function returns our model struct initialized and an Option<Task>, in this case
     *  there is no command so it returns a None value with the type of Task in its place.
     */
    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<cosmic::app::Message<Self::Message>>) {
        let current_entry = fetch_current_entry();
        let task = match current_entry.clone() {
            TimeEntry::Entry(t) => t,
            _ => "".to_string(),
        };

        let env = format!(
            "PATH: {}",
            env::var("PATH").unwrap_or_else(|_| "No PATH found".to_string())
        );

        let window = Window {
            core, // Set the incoming core
            popup: None,
            task_title: current_entry.clone(),
            form_description: Some(task.clone()),
            debug_text: Some(env),
        };

        (window, Task::none())
    }

    // Create what happens when the applet is closed
    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        // Pass the PopupClosed message to the update function
        Some(Message::PopupClosed(id))
    }

    // Here is the update function, it's the one that handles all of the messages that
    // are passed within the applet.
    fn update(&mut self, message: Self::Message) -> Task<cosmic::app::Message<Self::Message>> {
        // match on what message was sent
        match message {
            // Handle the TogglePopup message
            Message::TogglePopup => {
                // Close the popup
                return if let Some(popup_id) = self.popup.take() {
                    destroy_popup(popup_id)
                } else {
                    // Create and "open" the popup
                    let new_id = Id::unique();
                    self.popup.replace(new_id);

                    let mut popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None,
                        None,
                        None,
                    );

                    popup_settings.positioner.size_limits = Limits::NONE
                        .max_width(372.0)
                        .min_width(300.0)
                        .min_height(200.0)
                        .max_height(1080.0);

                    get_popup(popup_settings)
                };
            }
            // Unset the popup field after it's been closed
            Message::PopupClosed(popup_id) => {
                if self.popup.as_ref() == Some(&popup_id) {
                    self.popup = None;
                }
            }
            Message::StopEntry => {
                let brew_path = "/home/linuxbrew/.linuxbrew/bin";
                let _output = Command::new("clockify-cli") // Replace with your actual CLI tool
                    .env(
                        "PATH",
                        format!("{}:{}", brew_path, env::var("PATH").unwrap()),
                    ) // Ensure the stupid brew bin path is in our env
                    .arg("out")
                    .output(); // This runs the command and gets the output

                self.form_description = None;
                self.task_title = TimeEntry::NoEntry;
            }
            Message::StartEntry => {
                if self.form_description.is_some() {
                    let brew_path = "/home/linuxbrew/.linuxbrew/bin";
                    let _output = Command::new("clockify-cli") // Replace with your actual CLI tool
                        .env(
                            "PATH",
                            format!("{}:{}", brew_path, env::var("PATH").unwrap()),
                        ) // Ensure the stupid brew bin path is in our env
                        .arg("in")
                        .arg("-d")
                        .arg(self.form_description.clone().unwrap())
                        .output(); // This runs the command and gets the output

                    // This would be an optimistic update I s'pose
                    let t = TimeEntry::Entry(self.form_description.clone().unwrap());
                    self.task_title = t;
                    self.form_description = None;
                    self.popup = None;
                }
            }
            Message::UpdateFormDesc(description) => {
                self.form_description = Some(description);
            }
            Message::RefreshEntry => {
                self.task_title = fetch_current_entry();
                self.form_description = None;
            }
        }
        Task::none() // Again not doing anything that requires multi-threading here.
    }

    /*
     *  For an applet, the view function describes what an applet looks like. There's a
     *  secondary view function (view_window) that shows the widgets in the popup when it's
     *  opened.
     */
    fn view(&self) -> Element<Self::Message> {
        let button = button::custom(Element::from(
            row!(
                self.core.applet.text(self.task_title.to_string()),
                container(vertical_space().height(Length::Fixed(
                    (self.core.applet.suggested_size(true).1
                        + 2 * self.core.applet.suggested_padding(true)) as f32
                )))
            )
            .align_y(Alignment::Center),
        ))
        .padding([0, 8])
        .width(Length::Shrink)
        .on_press(Message::TogglePopup);

        autosize::autosize(button, AUTOSIZE_MAIN_ID.clone()).into()
    }

    // The actual GUI window for the applet. It's a popup.
    fn view_window(&self, _id: Id) -> Element<Self::Message> {
        // A text box to show if we've enabled or disabled anything in the model
        let content_list = list_column()
            .padding(5)
            .spacing(0)
            .add(settings::item(
                "Switch task",
                text_input(
                    "description",
                    match &self.form_description {
                        Some(t) => t.clone(),
                        None => "".to_string(),
                    },
                )
                .on_input(Message::UpdateFormDesc)
                .on_submit(Message::StartEntry),
            ))
            .add(settings::item(
                "Stop current entry",
                button::text("Stop").on_press(Message::StopEntry),
            ))
            .add(settings::item(
                "Refresh entry",
                button::text("Refresh").on_press(Message::RefreshEntry),
            ))
            .add(settings::item(
                "Is anything wrong?",
                text(match &self.debug_text {
                    Some(text) => text,
                    None => "",
                }),
            ));

        // Set the widget content list as the popup_container for the applet
        self.core.applet.popup_container(content_list).into()
    }
}

fn fetch_current_entry() -> TimeEntry {
    let brew_path = "/home/linuxbrew/.linuxbrew/bin";
    let output = Command::new("clockify-cli") // Replace with your actual CLI tool
        .env(
            "PATH",
            format!("{}:{}", brew_path, env::var("PATH").unwrap()),
        ) // Ensure the stupid brew bin path is in our env
        .arg("show")
        .arg("--format")
        .arg("{{.Description}}")
        .output();
    // This runs the command and gets the output
    //clockify-cli show --format "{{.Description}}"

    match output {
        Ok(output) => {
            if output.status.success() {
                TimeEntry::Entry(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                TimeEntry::NoEntry
            }
        }
        Err(e) => TimeEntry::CliError(format!("Crash!!: {}", e)),
    }
}
