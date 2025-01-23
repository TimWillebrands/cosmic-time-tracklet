mod window;

// Many thanks to: https://bhh32.com/posts/tutorials/cosmic_applet_tutorial

// Import the applet model (Window)
use crate::window::Window;

// The main function returns a cosmic::iced::Result that is returned from
// the run function that's part of the applet module.
fn main() -> cosmic::iced::Result {
    cosmic::applet::run::<Window>(())?;

    Ok(())
}
