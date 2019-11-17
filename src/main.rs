extern crate gtk;
extern crate gio;
extern crate curl;
extern crate core;
extern crate mailparse;

use gtk::prelude::*;
use gio::prelude::*;

use gtk::{Application, ApplicationWindow, Button, Orientation};

mod imap;


fn main() {
    let application = Application::new(
        Some("de.f1ndus.imap-notes"),
        Default::default(),
    ).expect("failed to initialize GTK application");

    application.connect_activate(|app| {
        let window = ApplicationWindow::new(app);
        window.set_title("Notes");
        window.set_default_size(350, 70);

        let button = Button::new_with_label("Click me!");
        button.connect_clicked(|_| {
            println!("Clicked!");
        });

        let button2 = Button::new_with_label("Click me!");
        button.connect_clicked(|_| {
            println!("Clicked!");
        });


        let boxx = gtk::Box::new(Orientation::Vertical, 2);
        boxx.add(&button);
        boxx.add(&button2);

        window.add(&boxx);


        window.show_all();
    });



    application.run(&[]);
}
