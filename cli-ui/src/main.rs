extern crate apple_notes_manager;
extern crate itertools;
extern crate log;
extern crate diesel;

use std::{io};
use tui::Terminal;
use tui::backend::CrosstermBackend;
use tui::layout::{Layout, Constraint, Direction};
use tui::widgets::{Borders, Block, ListItem, List, ListState, Paragraph, Wrap};
use tui::style::{Modifier, Style, Color};
use std::sync::{mpsc, Mutex, Arc};
use std::time::{Instant, Duration};
use itertools::*;
use std::{thread};

use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use apple_notes_manager::db::{DatabaseService, SqliteDBConnection};
use tui::layout::Alignment;
use apple_notes_manager::notes::localnote::LocalNote;
use std::thread::sleep;
use crate::Outcome::{Success, Failure, End, Busy};
use apple_notes_manager::AppleNotes;
use apple_notes_manager::notes::traits::identifyable_note::IdentifyableNote;
use std::sync::mpsc::{
    Sender,
    Receiver
};
use crossterm::event::KeyEvent;

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

struct UiState {
    action_sender: Sender<Task>,
    event_receiver: Receiver<Event<KeyEvent>>,
    event_sender: Sender<Event<KeyEvent>>
}

struct App {
    apple_notes: Arc<Mutex<AppleNotes>>,
    app_stuff: Arc<Mutex<AppStuff>>
}

struct AppStuff {
    action_receiver: Receiver<Task>,
    event_sender: Sender<Event<KeyEvent>>
}

struct Ui<'u> {
    note_list_state: ListState,
    end: bool,
    color: Color,
    status: String,
    app: Arc<Mutex<AppleNotes>>,
    ui_state: Arc<Mutex<UiState>>,
    entries: Vec<LocalNote>,
    keyword: Option<String>,
    items: Vec<ListItem<'u>>,
    list: List<'u>,
    text: String,
    scroll_amount: u16,
    in_search_mode: bool,
}

impl<'u> Ui<'u> {

    fn gen_list(&self) -> List<'u> {

        let title = match self.keyword.clone() {
            None => {
                format!("List")
            }
            Some(word) => {
                format!("List Filter:[{}]", word)

            }
        };

        List::new(self.items.clone())
            .block(Block::default().title(title).borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol(">>")
    }

    fn generate_list_items(&mut self) -> Vec<ListItem<'u>> {
        self.entries.iter()
            .filter(|entry| {
                if self.keyword.is_some() {
                    entry.body[0].text.as_ref().unwrap().to_lowercase().contains(&self.keyword.as_ref().unwrap().to_lowercase())
                } else {
                    return true
                }
            })
            .map(|e| {
                if e.needs_merge() {
                    ListItem::new(format!("[M] {} {}", e.metadata.folder(), e.first_subject()).to_string()).style(Style::default().fg(Color::LightBlue))
                } else if e.content_changed_locally() {
                    ListItem::new(format!("{} {}", e.metadata.folder(), e.first_subject()).to_string()).style(Style::default().fg(Color::LightYellow))
                } else if e.metadata.locally_deleted {
                    ListItem::new(format!("{} {}", e.metadata.folder(), e.first_subject()).to_string()).style(Style::default().fg(Color::LightRed))
                } else if e.metadata.new {
                    ListItem::new(format!("{} {}", e.metadata.folder(), e.first_subject()).to_string()).style(Style::default().fg(Color::LightGreen))
                } else {
                    ListItem::new(format!("{} {}", e.metadata.folder(), e.first_subject()).to_string())
                }
            }).collect()
    }

    fn refresh(&mut self) {
        self.entries = refetch_notes(&self.app.lock().unwrap(), &self.keyword);
        self.items = self.generate_list_items( );
        self.list = self.gen_list();
    }

    fn reload_text(&mut self) {
        // self.note_list_state.select(Some(0));

        match self.note_list_state.selected() {
            Some(index) if matches!(self.entries.get(index), Some(_)) => {
                let entry = self.entries.get(index).unwrap();
                self.text = entry.body[0].text.as_ref().unwrap().clone();
            }
            _ => {
                self.text = "".to_string();
            }
        }
    }

    fn set_status<'a>(&self, text: &'a str, color: Color) -> Paragraph<'a> {
        Paragraph::new(text)
            .block(Block::default().title("Status").borders(Borders::ALL))
            .style(Style::default().fg(color))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {

        enable_raw_mode().expect("can run in raw mode");

        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        terminal.clear().unwrap();

        //let ui_state = self.ui_state.lock().unwrap();

        self.status = "Syncing".to_string();
        self.color = Color::Yellow;

        //ui_state.action_sender.send(Task::Sync);

        // Insert Thread for input detection


        self.note_list_state = ListState::default();
        self.note_list_state.select(Some(0));
        self.end = false;

        self.refresh();

        self.reload_text();
        self.scroll_amount = 0;

        let a = Arc::clone(&self.app);
        let b =  Arc::clone(&self.ui_state);

        let app = a.lock().unwrap();
        let ui_state = b.lock().unwrap();

        loop {

            terminal.draw(|f| {

                let value = &self.status;
                let t2 = self.set_status(value, self.color);

                let lay = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints(
                        [
                            Constraint::Percentage(95),
                            Constraint::Percentage(5),
                        ].as_ref()
                    );

                let chunks = lay.split(f.size());

                let noteslayout = Layout::default()
                    .direction(Direction::Horizontal)
                    .margin(0)
                    .constraints(
                        [
                            Constraint::Percentage(20),
                            Constraint::Percentage(80),
                        ].as_ref()
                    ).split(chunks[0]);

                f.render_stateful_widget(
                    self.list.clone(),
                    noteslayout[0],
                    &mut self.note_list_state.clone()
                );

                let t  = Paragraph::new(self.text.clone())
                    .block(Block::default().title("Content").borders(Borders::ALL))
                    .style(Style::default().fg(Color::White))
                    .alignment(Alignment::Left)
                    .scroll((self.scroll_amount,self.scroll_amount))
                    .wrap(Wrap { trim: false });


                f.render_widget(t, noteslayout[1]);
                f.render_widget(t2.clone(), chunks[1]);
            }).unwrap();

            let received_keystroke = self.ui_state.lock().unwrap().event_receiver.recv()?;

            if self.in_search_mode {
                match received_keystroke {
                    Event::Input(event) => match event.code {
                        KeyCode::Esc => {
                            self.status = "".to_string();
                            self.color = Color::White;
                            self.in_search_mode = false;
                            self.refresh();
                            self.reload_text()
                        }
                        KeyCode::Backspace => {
                            if self.keyword.is_some() {
                                let len = self.keyword.as_ref().unwrap().len();
                                if len > 0 {
                                    let mut d = self.keyword.as_ref().unwrap().clone();
                                    d.pop();
                                    self.keyword = Some(d);
                                    self.status = self.keyword.as_ref().unwrap().clone();
                                }

                                self.refresh();
                                self.note_list_state.select(Some(0));
                            }
                        }
                        KeyCode::Char(c) => {
                            let ed = c;
                            self.keyword = Some(format!("{}{}", self.keyword.as_ref().unwrap(), ed));
                            self.status = self.keyword.as_ref().unwrap().clone();
                            self.refresh();
                        }
                        _ => {}
                    }
                    _ => {}
                }
            } else {
                match received_keystroke {
                    Event::Input(event) => match event.code {
                        KeyCode::Char('j') => {
                            let selected = self.note_list_state.selected();
                            if self.entries.len() > 0 && selected.unwrap_or(0) < self.entries.len() -1 {
                                self.note_list_state.select(Some(selected.unwrap_or(0) + 1));
                                self.reload_text();
                                self.scroll_amount = 0;
                            }
                        },
                        KeyCode::Char('k') => {
                            let selected = self.note_list_state.selected();
                            if selected.unwrap_or(0) > 0 {
                                self.note_list_state.select(Some(selected.unwrap_or(0) - 1));
                                self.reload_text();
                                self.scroll_amount = 0;
                            }
                        },
                        KeyCode::Char('J') => {
                            self.scroll_amount += 4;
                        },
                        KeyCode::Char('K') => {
                            if self.scroll_amount >= 4 {
                                self.scroll_amount -= 4;
                            } else {
                                self.scroll_amount = 0;
                            }
                        },
                        KeyCode::Char('m') => {
                            let note = self.entries.get(self.note_list_state.selected().unwrap()).unwrap();
                            match app.merge(&note.metadata.uuid) {
                                Ok(_) => {
                                    let old_uuid = self.entries.get(self.note_list_state.selected().unwrap()).unwrap().metadata.uuid.clone();
                                    self.refresh();

                                    let old_note_idx = self.entries.iter().enumerate().filter(|(_idx,note)| {
                                        note.metadata.uuid == old_uuid
                                    }).last().unwrap().0;

                                    self.note_list_state.select(Some(old_note_idx));
                                    self.reload_text();
                                }
                                Err(e) => {
                                    self.color = Color::Red;
                                    self.status = e.to_string();
                                }
                            }
                        }
                        KeyCode::Char('e') => {
                            let note = self.entries.get(self.note_list_state.selected().unwrap()).unwrap();
                            let result: Result<LocalNote,Box<dyn std::error::Error>> =
                                app.edit_note(&note, false)
                                    .map_err(|e| e.into())
                                    .and_then(|note| app.update_note(&note).map(|_n| note).map_err(|e| e.into()));

                            match result {
                                Ok(_note) => {
                                    let old_uuid = self.entries.get(self.note_list_state.selected().unwrap()).unwrap().metadata.uuid.clone();

                                    self.refresh();

                                    let old_note_idx = self.entries.iter().enumerate().filter(|(_idx,note)| {
                                        note.metadata.uuid == old_uuid
                                    }).last().unwrap().0;

                                    self.note_list_state.select(Some(old_note_idx));
                                    self.reload_text();

                                }
                                Err(e) => {
                                    self.color = Color::Red;
                                    self.status = e.to_string();
                                }
                            }

                        },
                        KeyCode::Char('d') => {
                            let mut note = self.entries.get(self.note_list_state.selected().unwrap()).unwrap().clone();
                            note.metadata.locally_deleted = !note.metadata.locally_deleted ;

                            let db_connection = apple_notes_manager::db::SqliteDBConnection::new();
                            db_connection.update(&note).unwrap();

                            self.refresh();

                        },
                        KeyCode::Char('s') => {
                            self.status = "Syncing".to_string();
                            self.color = Color::Yellow;

                            ui_state.action_sender.send(Task::Sync).unwrap();

                        },
                        KeyCode::Char('x') => {
                            self.status = "Syncing".to_string();
                            self.color = Color::Yellow;

                            ui_state.action_sender.send(Task::Test).unwrap();
                        },
                        KeyCode::Char('q') => {
                            self.end = true;

                            ui_state.action_sender.send(Task::End).unwrap();
                        },
                        KeyCode::Char('/') => {
                            self.keyword = Some("".to_string());
                            self.status = format!("Search mode: {}", self.keyword.as_ref().unwrap());
                            self.color = Color::Cyan;
                            self.in_search_mode = true;
                        },
                        KeyCode::Char('c') => {
                            self.status = format!("Filter Cleared");
                            self.color = Color::White;

                            self.keyword = None;

                            let mut old_uuid = None;

                            if let Some(old_selected_entry) = self.entries.get(self.note_list_state.selected().unwrap_or(0)) {
                                old_uuid = Some(old_selected_entry.metadata.uuid.clone());
                            }

                            self.refresh();

                            if let Some(uuid) = old_uuid {
                                let old_note_idx = self.entries.iter().enumerate().filter(|(_idx,note)| {
                                    note.metadata.uuid == uuid
                                }).last().unwrap().0;

                                self.note_list_state.select(Some(old_note_idx));
                            }

                        },
                        KeyCode::Esc => {
                            self.status = "".to_string();
                            self.in_search_mode = false;
                        }
                        _ => {}
                    }
                    Event::Tick => {}
                    Event::OutCome(outcome) => match outcome {
                        Outcome::Busy() => {
                            self.color = Color::Red;
                            self.status = "Currently Busy".to_string();
                        }
                        Outcome::Success(s) => {
                            let mut old_uuid = None;

                            if let Some(old_selected_entry) = self.entries.get(self.note_list_state.selected().unwrap_or(0)) {
                                old_uuid = Some(old_selected_entry.metadata.uuid.clone());
                            }

                            self.color = Color::Green;
                            self.status = s;

                            self.refresh();

                            let mut index = self.note_list_state.selected().unwrap_or(0);

                            //TODO old_uuid if present selection
                            if index > self.items.len() - 1 {
                                index = self.items.len() - 1;
                                self.note_list_state.select(Some(index));
                            }

                            if let Some(uuid) = old_uuid {
                                let old_note_idx = self.entries.iter().enumerate().filter(|(_idx,note)| {
                                    note.metadata.uuid == uuid
                                }).last().unwrap().0;

                                self.note_list_state.select(Some(old_note_idx));
                            }

                            self.text = self.entries.get(index).unwrap().body[0].text.as_ref().unwrap().clone();
                        }
                        Outcome::Failure(s) => {
                            self.color = Color::Red;
                            self.status = s;
                            self.refresh();
                        }
                        Outcome::End() => {
                            break;
                        }
                    }
                }
            }


        }

        terminal.clear().unwrap();
        disable_raw_mode().unwrap();


        Ok(())
    }

}

impl App {

    pub fn new(action_receiver: Receiver<Task>) -> App {

        let (tx, rx) = mpsc::channel();

        let profile = apple_notes_manager::get_user_profile();
        let db_connection = SqliteDBConnection::new();
        let connection = Box::new(db_connection);
        let app = apple_notes_manager::AppleNotes::new(profile.unwrap(), connection);

        let app = App {
            apple_notes: Arc::new(Mutex::new(app)),
            app_stuff: Arc::new(Mutex::new(AppStuff {
                action_receiver,
                event_sender: tx
            }))
        };

        app

    }

    //TODO entries nil check
    pub fn run(&'static self) -> Result<(), Box<dyn std::error::Error + '_>> {

        let worker_tread = thread::spawn( move || {

            let active = Arc::new(Mutex::new(false));

            let a = Arc::clone(&self.app_stuff);
            let b =  Arc::clone(&self.apple_notes);

            let app_stuff = a.lock().unwrap();
            let app_lock = b.lock().unwrap();

            let action_rx = &app_stuff.action_receiver;
            let event_tx = &app_stuff.event_sender;

            loop {

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
                                    match app_lock.lock().unwrap().sync_notes() {
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

        });


        worker_tread.join().unwrap();

        Ok(())
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

    let app = App::new(action_rx);

    let ui_state = UiState {
        action_sender: action_tx,
        event_receiver: rx,
        event_sender: tx
    };

    let mut ui = Ui {
        note_list_state: Default::default(),
        end: false,
        color: Color::Reset,
        status: "Started".to_string(),
        app: app.apple_notes,
        ui_state: Arc::new(Mutex::new(ui_state)),
        entries: vec![],
        keyword: None,
        items: vec![],
        list: List::new(Vec::new()),
        text: "".to_string(),
        scroll_amount: 0,
        in_search_mode: false
    };

    ui.run().unwrap();
}