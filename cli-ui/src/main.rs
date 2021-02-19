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

enum Event<I> {
    Input(I),
    Tick,
}

enum Tasks {
    Sync
}

struct App {
     status: Arc<Mutex<String>>,
     color: Arc<Mutex<Color>>
}

impl App {

    pub fn new() -> App {
        App {
            status: Arc::new(Mutex::new("Started".to_string())),
            color: Arc::new(Mutex::new(Color::White))
        }
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        enable_raw_mode().expect("can run in raw mode");

        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        terminal.clear().unwrap();

        let (tx, rx) = mpsc::channel();

        let tick_rate = Duration::from_millis(1000);

        let color_2 = Arc::new(Mutex::new(Color::White));
        let color_3 = color_2.clone();


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

        thread::spawn( move || {

            let ten_millis = time::Duration::from_millis(3000);
            thread::sleep(ten_millis);

            *color_2.lock().unwrap() = Color::Red;

            //apple_notes_rs_lib::sync::sync_notes().unwrap();

        });



        let db =  apple_notes_rs_lib::db::SqliteDBConnection::new();

        let mut entries = self.refetch_notes(&db);
        let mut items: Vec<ListItem> = self.generate_list_items(&entries);

        let mut list = self.gen_list(&mut items);

        let mut text: String = "".to_string();

        let mut note_list_state = ListState::default();
        note_list_state.select(Some(0));

        let mut scroll_amount = 0;

        loop {

            terminal.draw(|f| {

                let value = self.status.lock().unwrap();
                let t2 = self.set_status(value.as_ref(), *color_3.lock().unwrap());

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

                f.render_stateful_widget(list.clone(), noteslayout[0], &mut note_list_state);

                let t  = Paragraph::new(text.clone())
                    .block(Block::default().title("Content").borders(Borders::ALL))
                    .style(Style::default().fg(Color::White))
                    .alignment(Alignment::Left)
                    .scroll((scroll_amount,scroll_amount))
                    .wrap(Wrap { trim: true });


                f.render_widget(t, noteslayout[1]);
                f.render_widget(t2.clone(), chunks[1]);
            }).unwrap();

            match rx.recv()? {
                Event::Input(event) => match event.code {
                    KeyCode::Char('j') => {
                        if note_list_state.selected().unwrap_or(0) < entries.len() -1 {
                            note_list_state.select(Some(note_list_state.selected().unwrap_or(0) + 1));
                            text = entries.get(note_list_state.selected().unwrap()).unwrap().body[0].text.as_ref().unwrap().clone();
                        }
                    },
                    KeyCode::Char('k') => {
                        if note_list_state.selected().unwrap_or(0) > 0 {
                            note_list_state.select(Some(note_list_state.selected().unwrap_or(0) - 1));
                            text = entries.get(note_list_state.selected().unwrap()).unwrap().body[0].text.as_ref().unwrap().clone();
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
                        let note = entries.get(note_list_state.selected().unwrap()).unwrap();
                        let result: Result<LocalNote,Box<dyn std::error::Error>> = apple_notes_rs_lib::edit::edit_note(&note, false).map_err(|e| e.into());
                        result.and_then(|note| db.update(&note).map_err(|e| e.into()))
                            .unwrap();
                        let db_connection = apple_notes_rs_lib::db::SqliteDBConnection::new();
                        entries = self.refetch_notes(&db_connection);
                        items = self.generate_list_items(&entries);
                        list = self.gen_list(&mut items);
                    },
                    KeyCode::Char('d') => {
                        let mut note = entries.get(note_list_state.selected().unwrap()).unwrap().clone();
                        note.metadata.locally_deleted = !note.metadata.locally_deleted ;
                        let db_connection = apple_notes_rs_lib::db::SqliteDBConnection::new();
                        db_connection.update(&note).unwrap();
                        entries = self.refetch_notes(&db_connection);
                        items = self.generate_list_items(&entries);
                        list = self.gen_list(&mut items);

                    },
                    KeyCode::Char('s') => {
                        //TODO block multiple invocations
                        *self.status.lock().unwrap() = "Syncing".to_string();
                        *self.color.lock().unwrap() = Color::Yellow;



                        let db_connection = apple_notes_rs_lib::db::SqliteDBConnection::new();
                        entries = self.refetch_notes(&db_connection);
                        items = self.generate_list_items(&entries);
                        list = self.gen_list(&mut items);
                    },
                    KeyCode::Char('q') => {
                        terminal.clear().unwrap();
                        break;
                    }
                    _ => {}
                }
                Event::Tick => {}
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

    fn refetch_notes(&self, db: &SqliteDBConnection) -> Vec<LocalNote> {
        db.fetch_all_notes().unwrap()
            .into_iter()
            .sorted_by_key(|note| format!("{}_{}", &note.metadata.subfolder, &note.body[0].subject()))
            .collect()
    }

    fn generate_list_items<'a>(&self, entries: &Vec<LocalNote>) -> Vec<ListItem<'a>> {
        entries.iter().map(|e| {
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

fn main() {
    let app = App::new();
    app.run();
}