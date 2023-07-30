use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box, Button, Entry, Orientation, Stack, StackSwitcher};
use std::env;

mod q_logic;

const APP_ID: &str = "org.gtk_rs.GPT4CHAT";

fn main() {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(|app| {
        let token_entry = Entry::builder()
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .placeholder_text("Enter your API token here...")
            .build();

        let token_button = Button::builder()
            .label("Save token and start chat")
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();

        let token_box = Box::new(Orientation::Vertical, 6);
        token_box.append(&token_entry);
        token_box.append(&token_button);

        let chat_entry = Entry::builder()
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .placeholder_text("Enter your message here...")
            .build();

        let chat_button = Button::builder()
            .label("Send message")
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();

        let user_message_view = gtk::TextView::builder()
            .editable(false)
            .margin_bottom(20)
            .margin_start(20)
            .margin_end(20)
            .margin_top(20)
            .build();
        
        let assistant_message_view = gtk::TextView::builder()
            .editable(false)
            .margin_bottom(20)
            .margin_start(20)
            .margin_end(20)
            .margin_top(20)
            .build();
        
        let chat_box = Box::new(Orientation::Vertical, 6);
        chat_box.append(&chat_entry);
        chat_box.append(&chat_button);
        chat_box.append(&user_message_view);
        chat_box.append(&assistant_message_view);

        let chat_entry_clone = chat_entry.clone();
        let user_message_view_clone = user_message_view.clone();
        let ass_view_clone = assistant_message_view.clone();

        chat_button.connect_clicked(move |_| {
            // Get the text from the chat_entry
            let user_message = chat_entry_clone.text().to_string();
        
            let assistant_response: String = q_logic::respond(&user_message);

            // Display the user message
            let buffer = user_message_view_clone.buffer();
            buffer.set_text(&user_message);
            let a_buffer = ass_view_clone.buffer();
            a_buffer.set_text(&assistant_response);
            // Clear the chat_entry
            chat_entry_clone.set_text("");
        });

        let stack = Stack::new();
        stack.add_titled(&token_box, Some("token"), "API Token");
        stack.add_titled(&chat_box, Some("chat"), "Chat");

        let stack_switcher = StackSwitcher::builder().stack(&stack).build();

        let vbox = Box::new(Orientation::Vertical, 6);
        vbox.append(&stack_switcher);
        vbox.append(&stack);

        let token_entry_clone = token_entry.clone();
        let stack_clone = stack.clone();
        token_button.connect_clicked(move |_| {
            let token = token_entry_clone.text().to_string();
            env::set_var("OPENAI_API_KEY", &token);
            stack_clone.set_visible_child_name("chat");
        });

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Chat App")
            .child(&vbox)
            .build();

        window.present();
    });

    app.run();
}