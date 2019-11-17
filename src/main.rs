extern crate gtk;
extern crate gio;
extern crate curl;
extern crate core;
extern crate mailparse;
extern crate gdk;

#[macro_use]
extern crate cascade;

use gtk::prelude::*;
use gio::prelude::*;

use gtk::{Application, ApplicationWindow, Button, Orientation, ListBoxBuilder, ListBox, LabelBuilder, StateFlags};

mod imap;

use imap::Note::NoteTrait;


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

        let list_box =
            ListBoxBuilder::new()
                .halign(gtk::Align::Fill)
                .valign(gtk::Align::Fill)
                .build();

        list_box.override_background_color(StateFlags::NORMAL, Some(&gdk::RGBA::green()));

        let label = LabelBuilder::new().label("Hi").build();
        list_box.add(&label);

        let mut session = imap::login();
        println!("MEEEEEM");
        let folders = imap::list_note_folders(&mut session);
        let foldername = folders.iter().last().unwrap().to_string();
        let _messages = imap::get_messages_from_foldersession(&mut session, foldername);
        _messages.iter().for_each(|b| {
            let label = LabelBuilder::new().label(b.subject().to_string().as_ref()).build();
            list_box.add(&label);
        });

        //GRID

        let boxx = gtk::BoxBuilder::new()
            .halign(gtk::Align::Fill)
            .valign(gtk::Align::Fill)
            .build();

        let grid = gtk::GridBuilder::new()
            .column_spacing(12)
            .row_spacing(3)
            .valign(gtk::Align::Fill)
            .halign(gtk::Align::Fill)
            .build();


        let label = gtk::LabelBuilder::new().label("Click").build();

        let event_box = cascade! {
            gtk::EventBoxBuilder::new()
                .can_focus(false)
                .hexpand(true)
                .vexpand(true)
                .events(gdk::EventMask::BUTTON_PRESS_MASK)
                .build();
            ..add(&cascade! {
                gtk::GridBuilder::new()
                .valign(gtk::Align::Fill)
                .hexpand(true)
                .vexpand(true)
                    .column_spacing(12)
                    .row_spacing(3)
                    .build();
                ..attach(&list_box, 1, 0, 1, 2);
                ..attach(&label, 0, 0, 1, 1);
            });
        };

        event_box.override_background_color(StateFlags::NORMAL, Some(&gdk::RGBA::blue()));

        window.add(&event_box);


        window.show_all();
    });



    application.run(&[]);
}
