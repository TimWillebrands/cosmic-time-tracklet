# COSMIC DE Time Tracklet

Welcome to the COSMIC DE Time Tracklet! 🚀 This handy little applet lets you track your time using Clockify as the backend. Whether you're starting a new task, stopping an existing one, or simply refreshing your current entry, this applet has you covered.
## Features 🌟

    Start a Task: Begin tracking time for a new task with just a description.
    Stop the Current Task: End the task you're currently working on.
    Refresh the Current Task: Fetch the latest status of your current task from Clockify.
    Popup UI: A clean and simple popup for managing tasks.

## Installation 🛠️

    Clone this repository:

    git clone https://github.com/yourusername/cosmic-de-time-tracklet.git

    Install Clockify CLI tool:
        Make sure you have Clockify CLI installed and properly configured.

    Build:
        1) Use `cargo build --release`
        2) From the project folder copy the binary we've just compiled to the applet folder. For example using: `sudo cp -f target/release/cosmic-applet-clockify`

    Deploy:
        1) Copy the `cosmic-time-tracklet.desktop` file to `/usr/share/applications`
        2) Open up COSMIC Settings and navigate to Desktop->Panel and add the applet to the panel.

## Usage 🕹️

Once installed and set up, run the applet to track your tasks effortlessly:

    Start a Task: Enter a description in the input box and hit "Submit".
    Stop the Current Task: Press "Stop" to mark your task as completed.
    Refresh the Current Task: Press "Refresh" to pull the latest task details from Clockify.

You can toggle the applet's popup by clicking on the button in the main window.
## How It Works 🧑‍💻

    Fetch Current Task: The applet uses clockify-cli to fetch your current task description or any errors that may occur.
    Start/Stop/Refresh Tasks: We send commands to clockify-cli to start, stop, or refresh tasks. The applet handles these actions asynchronously, ensuring that your interface remains responsive.

## Customization 🔧

Feel free to tweak the code to fit your needs:

    Modify the Clockify CLI commands: If you prefer another way of interacting with Clockify, adjust the commands in the fetch_current_entry function.
    UI Tweaks: Change the layout and styling as desired to fit your design preferences.

## Troubleshooting 🛠️

If something goes wrong (e.g., Clockify CLI crashes), you can see debug information in the popup. This will show any errors that happen during the interaction with the Clockify API.

## Thanks
https://bhh32.com/posts/tutorials/cosmic_applet_tutorial
