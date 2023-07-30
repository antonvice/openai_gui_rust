mod q_logic;

use gtk::prelude::*;
use gtk::{glib, Application, ApplicationWindow, Button, Entry};
use std::env;
const APP_ID: &str = "org.gtk_rs.GPT4CHAT";

fn main() -> glib::ExitCode {
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run()
}

fn build_ui(app: &Application) {
    // Create an Entry (text field) to get user input
    let entry = Entry::builder()
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .placeholder_text("Enter text here...")
        .build();

    // Create a button with label and margins
    let button = Button::builder()
        .label("Press me!")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let reveal_button = Button::builder()
        .label("PYour code will display here!")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    // Create a vertical box to hold the entry and the button
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 6);

    // Pack the entry and the button into the box
    vbox.append(&entry);
    vbox.append(&button);
    vbox.append(&reveal_button);

    // Create a window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("My GTK App")
        .child(&vbox)
        .build();

    // Connect to "clicked" signal of `button`
    button.connect_clicked(move |button| {   
        // Get the text from the entry
        let token = entry.text().to_string();
        env::set_var("OPENAI_API_KEY", &token);
        // Set the label to "Saved!" after the button has been clicked on
        button.set_label("API KEY Saved");
        reveal_button.set_label(&format!("Your code: {}", entry.text()));
        // Use the `token` variable globally or for further processing
        println!("User input: {}", token);
    });

    // Present window
    window.present();
}