extern crate apple_notes_manager;
extern crate itertools;
extern crate log;
extern crate diesel;

mod ui;

use std::sync::{mpsc, Mutex, Arc};
use std::time::{Duration};
use itertools::*;
use std::{thread};

use apple_notes_manager::db::{SqliteDBConnection};

use apple_notes_manager::notes::localnote::LocalNote;
use std::thread::{sleep, JoinHandle};
use crate::Outcome::{Success, Failure, End, Busy};
use apple_notes_manager::AppleNotes;

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
    Test
}

enum Outcome {
    Success(String),
    Failure(String),
    Busy(),
    End()
}

struct App {
    apple_notes: Arc<Mutex<AppleNotes>>,
    app_stuff: Arc<Mutex<AppStuff>>
}

struct AppStuff {
    action_receiver: Receiver<Task>,
    event_sender: Sender<Event<KeyEvent>>
}

impl App {

    pub fn new(action_receiver: Receiver<Task>, event_sender: Sender<Event<KeyEvent>>) -> App {

        let profile = apple_notes_manager::get_user_profile();
        let db_connection = SqliteDBConnection::new();
        let connection = Box::new(db_connection);
        let app = apple_notes_manager::AppleNotes::new(profile.unwrap(), connection);

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
        let a = Arc::clone(&self.app_stuff);
        let b =  Arc::clone(&self.apple_notes);
         thread::spawn( move || {

            let active = Arc::new(Mutex::new(false));

            loop {

                let app_stuff = a.lock().unwrap();
               // let app_lock = b.lock().unwrap();

                let action_rx = &app_stuff.action_receiver;
                let event_tx = &app_stuff.event_sender;

                let next = action_rx.recv().unwrap();

                if *active.lock().unwrap() == false {
                    *active.lock().unwrap() = true;
                    let active_2 = active.clone();
                    let tx_2 = event_tx.clone();
                    let app_lock = Arc::clone(&b);

                    if matches!(next,Task::End) {
                        tx_2.send(Event::OutCome(End())).unwrap();
                    } else {

                        thread::spawn( move || {
                            match next {
                                Task::Sync => {
                                    let d = app_lock.lock().unwrap();
                                    match d.sync_notes() {
                                        Ok(result) => {
                                            if result.iter().find(|r| r.2.is_err()).is_some() {
                                                tx_2.send(Event::OutCome(Failure(format!("Sync error: Could not sync all note")))).unwrap();
                                            } else {
                                                tx_2.send(Event::OutCome(Success("Synced!".to_string()))).unwrap();
                                            }
                                        }
                                        Err(e) => {
                                            tx_2.send(Event::OutCome(Failure(format!("Sync error: {}",e)))).unwrap();
                                        }
                                    }
                                }
                                Task::End => {

                                },
                                Task::Test => {
                                    sleep(Duration::new(2,0));
                                    tx_2.send(Event::OutCome(Success(format!("Test!")))).unwrap();
                                }
                            }

                            *active_2.lock().unwrap() = false;
                        });
                    }
                } else {
                    event_tx.send(Event::OutCome(Busy())).unwrap();
                };

               /* if *active.lock().unwrap() == false && *end.lock().unwrap() {
                    event_tx.send(Event::OutCome(End())).unwrap();
                    break;
                }*/

            }

        })
    }

}

fn refetch_notes(app: &AppleNotes, filter_word: &Option<String>) -> Vec<LocalNote> {
    app.get_notes().unwrap()
        .into_iter()
        .filter(|entry| {
            if filter_word.is_some() {
                entry.body[0].text.as_ref().unwrap().to_lowercase().contains(&filter_word.as_ref().unwrap().to_lowercase())
            } else {
                return true
            }
        })
        .sorted_by_key(|note| note.metadata.timestamp())
        .rev()
        .collect()
}

fn main() {

    let (tx, rx) = mpsc::channel();
    let (action_tx, action_rx) = mpsc::channel::<Task>();

    let app = App::new(action_rx, tx.clone());

    let handle = app.start_action_event_loop();

    let ui_state = UiState {
        action_sender: action_tx,
        event_receiver: rx,
        event_sender: Arc::new(Mutex::new(tx))
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
        in_search_mode: false
    };

    ui.run().unwrap();
    handle.join().unwrap();
}