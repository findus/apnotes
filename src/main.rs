extern crate gtk;
extern crate gio;
extern crate curl;
extern crate core;
extern crate mailparse;
extern crate gdk;

mod mail;
mod notes;

use crate::mail::fetcher::*;

use gtk::prelude::*;
use gio::prelude::*;
use crate::notes::*;
use notes::note::NoteTrait;

use gtk::{Application, ApplicationWindow, Button, Orientation, ListBoxBuilder, ListBox, LabelBuilder, StateFlags};

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
                .vexpand(true)
                .halign(gtk::Align::Fill)
                .valign(gtk::Align::Fill)
                .build();

        // list_box.override_background_color(StateFlags::NORMAL, Some(&gdk::RGBA::green()));

        let label = LabelBuilder::new().label("Hi").build();
        list_box.add(&label);

        let mut session = login();
        println!("MEEEEEM");
        let folders = list_note_folders(&mut session);
        let foldername = folders.iter().last().unwrap().to_string();
        let _messages = get_messages_from_foldersession(&mut session, foldername);
        _messages.iter().for_each(|b| {
            let label = LabelBuilder::new().label(b.subject().to_string().as_ref()).build();
            list_box.add(&label);
        });

        //let context = webkit2gtk::WebContext::get_default().unwrap();
        //let webView = webkit2gtk::WebView::new_with_context(&context);

        //GRID

        let pane = gtk::PanedBuilder::new()
            .vexpand(true)
            .hexpand(true)
            .build();

        let boxx = gtk::BoxBuilder::new()
            .halign(gtk::Align::Fill)
            .valign(gtk::Align::Fill)
            .build();

        let button = gtk::ButtonBoxBuilder::new().name("Click").build();

        pane.add1(&list_box);
        pane.add2(&button);

        window.add(&pane);


        window.show_all();
    });



    application.run(&[]);
}
