
#[cfg(target_family = "windows")]
fn main() {

}
extern crate apple_notes_rs_lib;
#[cfg(target_family = "unix")]
extern crate gtk;
#[cfg(target_family = "unix")]
extern crate gio;
#[cfg(target_family = "unix")]
extern crate curl;
#[cfg(target_family = "unix")]
extern crate core;
#[cfg(target_family = "unix")]
extern crate mailparse;
#[cfg(target_family = "unix")]
extern crate gdk;
#[cfg(target_family = "unix")]
extern crate apple_notes_rs;
#[cfg(target_family = "unix")]
use gtk::prelude::*;
#[cfg(target_family = "unix")]
use gio::prelude::*;
#[cfg(target_family = "unix")]
use gtk::{Application, ApplicationWindow, Button, ListBoxBuilder, LabelBuilder};
use apple_notes_rs_lib::apple_imap::*;
use apple_notes_rs_lib::note::{HeaderParser};

#[cfg(target_family = "unix")]
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

        let _button2 = Button::new_with_label("Click me!");
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
            let label = LabelBuilder::new().label(b.mail_headers.subject().to_string().as_ref()).build();
            list_box.add(&label);
        });

        //let context = webkit2gtk::WebContext::get_default().unwrap();
        //let webView = webkit2gtk::WebView::new_with_context(&context);

        //GRID

        let pane = gtk::PanedBuilder::new()
            .vexpand(true)
            .hexpand(true)
            .build();

        let _boxx = gtk::BoxBuilder::new()
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
