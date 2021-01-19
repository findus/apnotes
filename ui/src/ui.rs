extern crate gio;
extern crate gtk;
extern crate apple_notes_rs_lib;
#[macro_use]
extern crate glib;

use gio::prelude::*;
use gtk::prelude::*;
use gtk::ResponseType;
use std::env::args;
use row_data::RowData;
use apple_notes_rs_lib::notes::traits::header_parser::HeaderParser;
use apple_notes_rs_lib::db::DatabaseService;
use gtk::ScrolledWindowBuilder;
use apple_notes_rs_lib::notes::localnote::LocalNote;
use gio::prelude::*;
use gtk::prelude::*;
use gtk::*;


#[cfg(target_family = "unix")]
fn main() {

    let application = gtk::Application::new(
        Some("de.f1ndus.imap-notes"),
        Default::default(),
    ).expect("failed to initialize GTK application");

    application.connect_activate(|app| {
        let window = gtk::ApplicationWindow::new(app);
        window.set_title("Notes");
        window.set_default_size(350, 70);
        window.set_border_width(10);

        let button = Button::new_with_label("Click me!");
        button.connect_clicked(|_| {
            println!("Clicked!");
        });

        let _button2 = Button::new_with_label("Click me!");
        button.connect_clicked(|_| {
            println!("Clicked!");
        });

        let model = gio::ListStore::new(RowData::static_type());

        let list_box =
            ListBoxBuilder::new()
                .vexpand(true)
                .halign(gtk::Align::Fill)
                .valign(gtk::Align::Fill)
                .build();

        list_box.connect_selection_notify_event(|_,_| {
            println!("kkkk");
            Inhibit::default()
        });




        // list_box.override_background_color(StateFlags::NORMAL, Some(&gdk::RGBA::green()));

        let scroll = ScrolledWindowBuilder::new().build();

        use gtk::WrapMode::Char;
        let label = LabelBuilder::new().height_request(50).label("Hi").wrap(true).wrap_mode(pango::WrapMode::Char).build();
        list_box.add(&label);

        let db = ::apple_notes_rs_lib::db::SqliteDBConnection::new();
        let notes = db.fetch_all_notes().unwrap();


        list_box.bind_model(Some(&model),
        clone!(@weak window => @default-panic, move |item| {

            LabelBuilder::new()
                .height_request(50)
                .wrap(true)
                .wrap_mode(pango::WrapMode::Char)
                .label(&*b.body[0].subject())
                .build()
        })
        );

       /* notes.iter().for_each(|b| {
            let label = LabelBuilder::new()
                .height_request(50)
                .wrap(true)
                .wrap_mode(pango::WrapMode::Char)
                .label(&*b.body[0].subject())
                .build();

            list_box.add(&label);
        });*/

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

        let textview = gtk::TextViewBuilder::new().build();

        scroll.add(&list_box);

        pane.add1(&scroll);
        pane.add2(&textview);

        window.add(&pane);


        window.show_all();
    });



    application.run(&[]);
}

mod row_data {
    use super::*;

    use glib::subclass;
    use glib::subclass::prelude::*;
    use glib::translate::*;

    mod imp {
        use super::*;
        use std::cell::RefCell;

        pub struct RowData {
            note: RefCell<String>,
        }

        static PROPERTIES: [subclass::Property; 1] = [
            subclass::Property("note", |note| {
                glib::ParamSpec::string(
                    note,
                    "Note",
                    "Note",
                    None,
                    glib::ParamFlags::READWRITE,
                )
            })
        ];

        impl ObjectSubclass for RowData {
            const NAME: &'static str = "RowData";
            type ParentType = glib::Object;
            type Instance = subclass::simple::InstanceStruct<Self>;
            type Class = subclass::simple::ClassStruct<Self>;

            glib_object_subclass!();

            fn class_init(klass: &mut Self::Class) {
                klass.install_properties(&PROPERTIES);
            }

            fn new() -> Self {
                Self {
                    note: RefCell::new("".to_string())
                }
            }
        }

        impl ObjectImpl for RowData {
            glib_object_impl!();

            fn set_property(&self, _obj: &glib::Object, id: usize, value: &glib::Value) {
                let prop = &PROPERTIES[id];

                match *prop {
                    subclass::Property("note", ..) => {
                        let note = value
                            .get()
                            .expect("type conformity checked by 'Object::setproperty'");
                        self.note.replace(note);
                    },
                    _ => panic!("oops")
                }
            }

            fn get_property(&self, _obj: &glib::Object, id: usize) -> Result<glib::Value, ()> {
                let prop = &PROPERTIES[id];

                match *prop {
                    subclass::Property("note", ..) => Ok(self.note.borrow().to_value()),
                    _ => unimplemented!(),
                }
            }
        }




    }

    glib_wrapper! {
        pub struct RowData(Object<subclass::simple::InstanceStruct<imp::RowData>, subclass::simple::ClassStruct<imp::RowData>, RowDataClass>);

        match fn {
            get_type => || imp::RowData::get_type().to_glib(),
        }
    }

    // Constructor for new instances. This simply calls glib::Object::new() with
    // initial values for our two properties and then returns the new instance
    impl RowData {
        pub fn new(name: &str, count: u32) -> RowData {
            glib::Object::new(Self::static_type(), &[("name", &name)])
                .expect("Failed to create row data")
                .downcast()
                .expect("Created row data is of wrong type")
        }
    }
}

