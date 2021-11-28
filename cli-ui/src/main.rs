extern crate apnotes_lib;
extern crate itertools;
extern crate log;
extern crate diesel;

mod ui;

use std::sync::{mpsc, Mutex, Arc};
use std::time::{Duration};
use std::{thread};
use apnotes_lib::db::{SqliteDBConnection};
use std::thread::{sleep, JoinHandle};
use crate::Outcome::{Success, Failure, End, Busy};
use apnotes_lib::AppleNotes;
use std::sync::mpsc::{
    Sender,
    Receiver
};
use crossterm::event::KeyEvent;
use crate::ui::{
    UiState,
    Ui
};
use tui::style::Color;
use tui::widgets::List;

enum Event<I> {
    Input(I),
    Tick,
    OutCome(Outcome)
}

enum Task {
    Sync,
    End,
    Test,
    NewNote(String)
}

enum Outcome {
    Success(String),
    Failure(String),
    Busy(),
    End()
}

struct AppStuff {
    action_receiver: Receiver<Task>,
    event_sender: Sender<Event<KeyEvent>>
}

struct App {
    apple_notes: Arc<Mutex<AppleNotes>>,
    app_stuff: Arc<Mutex<AppStuff>>
}

impl App {

    pub fn new(action_receiver: Receiver<Task>, event_sender: Sender<Event<KeyEvent>>) -> App {

        let profile = apnotes_lib::get_user_profile();
        let db_connection = SqliteDBConnection::new();
        let connection = Box::new(db_connection);
        let app = apnotes_lib::AppleNotes::new(profile.unwrap(), connection);

        let app = App {
            apple_notes: Arc::new(Mutex::new(app)),
            app_stuff: Arc::new(Mutex::new(AppStuff {
                action_receiver,
                event_sender
            }))
        };

        app

    }

    //TODO entries nil check
    pub fn start_action_event_loop(& self) -> JoinHandle<()> {
        let app_stuff = Arc::clone(&self.app_stuff);
        let apple_notes =  Arc::clone(&self.apple_notes);

        thread::spawn( move || {

            // Indicator if a task ist currently active
            let active = Arc::new(Mutex::new(false));

            loop {

                let app_stuff = app_stuff.lock().unwrap();

                let action_rx = &app_stuff.action_receiver;
                let event_tx = &app_stuff.event_sender;

                let next_action = action_rx.recv().unwrap();

                if *active.lock().unwrap() == false {
                    *active.lock().unwrap() = true;
                    let active = active.clone();
                    let event_tx = event_tx.clone();
                    let app_lock = Arc::clone(&apple_notes);

                    if matches!(next_action,Task::End) {
                        event_tx.send(Event::OutCome(End())).unwrap();
                        break;
                    } else {

                        thread::spawn( move || {
                            match next_action {
                                Task::NewNote(name) => {
                                    let d = app_lock.lock().unwrap();
                                    match d.create_new_note(&name, &String::new()) {
                                        Ok(_) => {
                                            event_tx.send(Event::OutCome(Success("New note created".to_string()))).unwrap();
                                        }
                                        Err(e) => {
                                            event_tx.send(Event::OutCome(Failure(format!("Could not create note: {}", e)))).unwrap();
                                        }
                                    }
                                }
                                Task::Sync => {
                                    let d = app_lock.lock().unwrap();
                                    match d.sync_notes() {
                                        Ok(result) => {
                                            if result.iter().find(|syncresult| syncresult.result.is_err()).is_some() {
                                                event_tx.send(Event::OutCome(Failure(format!("Sync error: Could not sync all notes")))).unwrap();
                                            } else {
                                                event_tx.send(Event::OutCome(Success("Synced!".to_string()))).unwrap();
                                            }
                                        }
                                        Err(e) => {
                                            event_tx.send(Event::OutCome(Failure(format!("Sync error: {}", e)))).unwrap();
                                        }
                                    }
                                }
                                Task::End => {

                                },
                                Task::Test => {
                                    sleep(Duration::new(2,0));
                                    event_tx.send(Event::OutCome(Success(format!("Test!")))).unwrap();
                                }
                            }

                            *active.lock().unwrap() = false;
                        });
                    }
                } else {
                    event_tx.send(Event::OutCome(Busy())).unwrap();
                };

            }

        })
    }

}

fn main() {

    let (event_sender, event_receiver) = mpsc::channel();
    let (action_tx, action_rx) = mpsc::channel::<Task>();

    let app = App::new(action_rx, event_sender.clone());

    let handle = app.start_action_event_loop();

    let ui_state = UiState {
        action_sender: action_tx,
        event_receiver,
        event_sender: Arc::new(Mutex::new(event_sender))
    };

    let mut ui = Ui {
        note_list_state: Default::default(),
        end: false,
        color: Color::Reset,
        status: "Started".to_string(),
        app: app.apple_notes,
        ui_state: ui_state,
        entries: vec![],
        keyword: None,
        items: vec![],
        list: List::new(Vec::new()),
        text: "".to_string(),
        scroll_amount: 0,
        in_search_mode: false,
        new_note_mode: false,
    };

    ui.run().unwrap();
    handle.join().unwrap();
}