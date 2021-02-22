extern crate apple_notes_rs_lib;
extern crate itertools;
extern crate log;

use std::{io};
use tui::Terminal;
use tui::backend::CrosstermBackend;
use tui::layout::{Layout, Constraint, Direction};
use tui::widgets::{Borders, Block, ListItem, List, ListState, Paragraph, Wrap};
use tui::style::{Modifier, Style, Color};
use std::sync::{mpsc, Mutex, Arc};
use std::time::{Instant, Duration};
use itertools::*;
use std::{thread, time};

use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use apple_notes_rs_lib::db::{DatabaseService, SqliteDBConnection};
use apple_notes_rs_lib::notes::traits::identifyable_note::IdentifyableNote;
use tui::layout::Alignment;
use apple_notes_rs_lib::notes::localnote::LocalNote;
use std::thread::sleep;
use crate::Outcome::{Success, Failure};
use std::ops::Deref;

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
    Failure(String)
}

struct App {

}

impl App {

    pub fn new() -> App {
        App {

        }
    }

    //TODO entries nil check
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        enable_raw_mode().expect("can run in raw mode");

        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        terminal.clear().unwrap();

        let (tx, rx) = mpsc::channel();
        let tx_2 = tx.clone();

        let tick_rate = Duration::from_millis(1000);

        let color = Arc::new(Mutex::new(Color::White));
        let color_2 = color.clone();

        let status = Arc::new(Mutex::new("Started".to_string()));
        let status_2 = status.clone();

        let mut in_search_mode = false;
        let mut keyword = "".to_string();

        thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));

                if event::poll(timeout).expect("poll works") {
                    if let CEvent::Key(key) = event::read().expect("can read events") {
                        tx.send(Event::Input(key)).expect("can send events");
                    }
                }

                if last_tick.elapsed() >= tick_rate {
                    if let Ok(_) = tx.send(Event::Tick) {
                        last_tick = Instant::now();
                    }
                }
            }
        });

        let (action_tx, action_rx) = mpsc::channel::<Task>();

        let db_connection = apple_notes_rs_lib::db::SqliteDBConnection::new();

        let mut note_list_state = Arc::new(Mutex::new(ListState::default()));
        note_list_state.lock().unwrap().select(Some(0));
        let mut counter = Arc::new(Mutex::new(0));

        let note_list_state_2 = note_list_state.clone();
        let keyword_2 = keyword.clone();

        thread::spawn( move || {

            let active = Arc::new(Mutex::new(false));

            loop {
                let mut note_list_state_3 = note_list_state_2.clone();
                let next = action_rx.recv().unwrap();
                if *active.lock().unwrap() == false {
                    *active.lock().unwrap() = true;
                    let active_2 = active.clone();
                    let tx_3 = tx_2.clone();
                    let counter_2 = counter.clone();
                    let keyword_3 = keyword_2.clone();
                    thread::spawn( move || {
                        match next {
                            Task::Sync => {
                                let db_connection = apple_notes_rs_lib::db::SqliteDBConnection::new();
                                let entries = refetch_notes(&db_connection, &keyword_3);
                                let note = entries.get(note_list_state_3.lock().unwrap().selected().unwrap()).unwrap();
                                apple_notes_rs_lib::sync_notes().unwrap();
                                tx_3.send(Event::OutCome(Success("Synced!".to_string())));
                            }
                            Task::End => {

                            },
                            Task::Test => {
                                sleep(Duration::new(2,0));
                                *counter_2.lock().unwrap() += 1;
                                tx_3.send(Event::OutCome(Success(format!("Test! {}", *counter_2.lock().unwrap()))));
                            }
                        }

                        *active_2.lock().unwrap() = false;
                    });
                } else {
                    tx_2.send(Event::OutCome(Failure("Currently busy".to_string())));
                };

            }

            let ten_millis = time::Duration::from_millis(3000);
            thread::sleep(ten_millis);


            //apple_notes_rs_lib::sync::sync_notes().unwrap();

        });



        let db =  apple_notes_rs_lib::db::SqliteDBConnection::new();

        let mut entries: Vec<LocalNote> = refetch_notes(&db, &keyword);

        let mut items: Vec<ListItem> = self.generate_list_items(&entries, &keyword);

        let mut list = self.gen_list(&mut items);

        let mut text: String = "".to_string();

        let mut scroll_amount = 0;

        text = entries.get(note_list_state.lock().unwrap().selected().unwrap()).unwrap().body[0].text.as_ref().unwrap().clone();


        loop {

            terminal.draw(|f| {

                let value = status.lock().unwrap();
                let t2 = self.set_status(value.as_ref(), *color.lock().unwrap());

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

                f.render_stateful_widget(list.clone(), noteslayout[0], &mut note_list_state.lock().unwrap());

                let t  = Paragraph::new(text.clone())
                    .block(Block::default().title("Content").borders(Borders::ALL))
                    .style(Style::default().fg(Color::White))
                    .alignment(Alignment::Left)
                    .scroll((scroll_amount,scroll_amount))
                    .wrap(Wrap { trim: true });


                f.render_widget(t, noteslayout[1]);
                f.render_widget(t2.clone(), chunks[1]);
            }).unwrap();

            let received_keystroke = rx.recv()?;

            if in_search_mode {
                match received_keystroke {
                    Event::Input(event) => match event.code {
                        KeyCode::Esc => {
                            *status.lock().unwrap() = "".to_string();
                            *color.lock().unwrap() = Color::White;
                            in_search_mode = false;

                            entries = refetch_notes(&db_connection, &keyword);
                            items = self.generate_list_items(&entries, &keyword);
                            list = self.gen_list(&mut items);
                            note_list_state.lock().unwrap().select(Some(0));
                            text = entries.get(note_list_state.lock().unwrap().selected().unwrap()).unwrap().body[0].text.as_ref().unwrap().clone();

                        }
                        KeyCode::Backspace => {
                            let len = keyword.len();
                            if len > 0 {
                                keyword = keyword[..len-1].to_string();
                                *status.lock().unwrap() = keyword.clone();
                            }

                            entries = refetch_notes(&db_connection, &keyword);
                            items = self.generate_list_items(&entries, &keyword);
                            list = self.gen_list(&mut items);

                            note_list_state.lock().unwrap().select(Some(0));
                        }
                        KeyCode::Char(c) => {
                            let ed = c;
                            keyword = format!("{}{}", keyword, ed);
                             *status.lock().unwrap() = keyword.clone();

                            entries = refetch_notes(&db_connection, &keyword);
                            items = self.generate_list_items(&entries, &keyword);
                            list = self.gen_list(&mut items);
                        }
                        _ => {}
                    }
                    _ => {}
                }
            } else {
                match received_keystroke {
                    Event::Input(event) => match event.code {
                        KeyCode::Char('j') => {
                            let selected = note_list_state.lock().unwrap().selected();
                            if selected.unwrap_or(0) < entries.len() -1 {
                                note_list_state.lock().unwrap().select(Some(selected.unwrap_or(0) + 1));
                                text = entries.get(note_list_state.lock().unwrap().selected().unwrap()).unwrap().body[0].text.as_ref().unwrap().clone();
                                scroll_amount = 0;
                            }
                        },
                        KeyCode::Char('k') => {
                            let selected = note_list_state.lock().unwrap().selected();
                            if selected.unwrap_or(0) > 0 {
                                note_list_state.lock().unwrap().select(Some(selected.unwrap_or(0) - 1));
                                text = entries.get(note_list_state.lock().unwrap().selected().unwrap()).unwrap().body[0].text.as_ref().unwrap().clone();
                                scroll_amount = 0;
                            }
                        },
                        KeyCode::Char('J') => {
                            scroll_amount += 4;
                        },
                        KeyCode::Char('K') => {
                            if scroll_amount >= 4 {
                                scroll_amount -= 4;
                            } else {
                                scroll_amount = 0;
                            }
                        },
                        KeyCode::Char('e') => {
                            let note = entries.get(note_list_state.lock().unwrap().selected().unwrap()).unwrap();
                            let result: Result<LocalNote,Box<dyn std::error::Error>> = apple_notes_rs_lib::edit_note(&note, false).map_err(|e| e.into());
                            result.and_then(|note| db.update(&note).map_err(|e| e.into()))
                                .unwrap();
                            entries = refetch_notes(&db_connection, &keyword);
                            items = self.generate_list_items(&entries, &keyword);
                            list = self.gen_list(&mut items);
                            text = entries.get(note_list_state.lock().unwrap().selected().unwrap()).unwrap().body[0].text.as_ref().unwrap().clone();
                        },
                        KeyCode::Char('d') => {
                            let mut note = entries.get(note_list_state.lock().unwrap().selected().unwrap()).unwrap().clone();
                            note.metadata.locally_deleted = !note.metadata.locally_deleted ;
                            let db_connection = apple_notes_rs_lib::db::SqliteDBConnection::new();
                            db_connection.update(&note).unwrap();
                            entries = refetch_notes(&db_connection, &keyword);
                            items = self.generate_list_items(&entries, &keyword);
                            list = self.gen_list(&mut items);

                        },
                        KeyCode::Char('s') => {
                            *status.lock().unwrap() = "Syncing".to_string();
                            *color.lock().unwrap() = Color::Yellow;

                            action_tx.send(Task::Sync);

                        },
                        KeyCode::Char('x') => {
                            *status.lock().unwrap() = "Syncing".to_string();
                            *color.lock().unwrap() = Color::Yellow;

                            action_tx.send(Task::Test);
                        },
                        KeyCode::Char('q') => {
                            terminal.clear().unwrap();
                            break;
                        },
                        KeyCode::Char('/') => {
                            *status.lock().unwrap() = format!("Search mode: {}", keyword);
                            *color.lock().unwrap() = Color::Cyan;
                            in_search_mode = true;
                        },
                        KeyCode::Char('c') => {
                            *status.lock().unwrap() = format!("Filter Cleared");
                            *color.lock().unwrap() = Color::White;

                            keyword = "".to_string();

                            let old_uuid = entries.get(note_list_state.lock().unwrap().selected().unwrap()).unwrap().metadata.uuid.clone();

                            entries = refetch_notes(&db_connection, &keyword);
                            items = self.generate_list_items(&entries, &keyword);
                            list = self.gen_list(&mut items);

                            let old_note_idx = entries.iter().enumerate().filter(|(idx,note)| {
                                note.metadata.uuid == old_uuid
                            }).last().unwrap().0;

                            note_list_state.lock().unwrap().select(Some(old_note_idx));

                        },
                        KeyCode::Esc => {
                            *status.lock().unwrap() = "".to_string();
                            in_search_mode = false;
                        }
                        _ => {}
                    }
                    Event::Tick => {}
                    Event::OutCome(outcome) => match outcome {
                        Outcome::Success(s) => {
                            *color.lock().unwrap() = Color::Green;
                            *status.lock().unwrap() = s;
                            entries = refetch_notes(&db_connection, &keyword);
                            items = self.generate_list_items(&entries, &keyword);
                            list = self.gen_list(&mut items);
                            let mut index = note_list_state.lock().unwrap().selected().unwrap();

                            //TODO old_uuid if present selection
                            if index > items.len() - 1 {
                                index = items.len() - 1;
                                note_list_state.lock().unwrap().select(Some(index));
                            }

                            text = entries.get(index).unwrap().body[0].text.as_ref().unwrap().clone();
                        }
                        Outcome::Failure(s) => {
                            *color.lock().unwrap() = Color::Red;
                            *status.lock().unwrap() = s;
                            entries = refetch_notes(&db_connection, &keyword);
                            items = self.generate_list_items(&entries, &keyword);
                            list = self.gen_list(&mut items);
                        }
                    }
                }
            }


        }

        disable_raw_mode().unwrap();

        Ok(())
    }

    fn set_status<'a>(&self, text: &'a str, color: Color) -> Paragraph<'a> {
        Paragraph::new(text)
            .block(Block::default().title("Status").borders(Borders::ALL))
            .style(Style::default().fg(color))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true })
    }

    fn gen_list<'a>(&self, items: &mut Vec<ListItem<'a>>) -> List<'a> {
        List::new(items.clone())
            .block(Block::default().title("List").borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol(">>")
    }

    fn generate_list_items<'a>(&self, entries: &Vec<LocalNote>, filter_word: &String) -> Vec<ListItem<'a>> {
        entries.iter()
            .filter(|entry| {
                if filter_word.len() > 0 {
                    entry.body[0].text.as_ref().unwrap().to_lowercase().contains(&filter_word.to_lowercase())
                } else {
                    return true
                }
            })
            .map(|e| {
            if e.content_changed_locally() {
                ListItem::new(format!("{} {}",e.metadata.folder(),e.first_subject()).to_string()).style(Style::default().fg(Color::LightYellow))
            } else if e.metadata.locally_deleted {
                ListItem::new(format!("{} {}",e.metadata.folder(),e.first_subject()).to_string()).style(Style::default().fg(Color::LightRed))
            } else if e.metadata.new {
                ListItem::new(format!("{} {}",e.metadata.folder(),e.first_subject()).to_string()).style(Style::default().fg(Color::LightGreen))
            } else {
                ListItem::new(format!("{} {}",e.metadata.folder(),e.first_subject()).to_string())
            }
        }).collect()
    }
}

fn refetch_notes(db: &SqliteDBConnection, filter_word: &String) -> Vec<LocalNote> {
    db.fetch_all_notes().unwrap()
        .into_iter()
        .filter(|entry| {
            if filter_word.len() > 0 {
                entry.body[0].text.as_ref().unwrap().to_lowercase().contains(&filter_word.to_lowercase())
            } else {
                return true
            }
        })
        .sorted_by_key(|note| format!("{}_{}", &note.metadata.subfolder, &note.body[0].subject()))
        .collect()
}

fn main() {
    let app = App::new();
    app.run();
}