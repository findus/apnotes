extern crate apple_notes_rs_lib;
extern crate itertools;

use std::{io, thread};
use tui::Terminal;
use tui::backend::CrosstermBackend;
use tui::layout::{Layout, Constraint, Direction};
use tui::widgets::{Borders, Block, ListItem, List, ListState, Paragraph, Wrap};
use tui::style::{Modifier, Style, Color};
use std::sync::mpsc;
use std::time::{Instant, Duration};
use itertools::*;

use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use apple_notes_rs_lib::db::{DatabaseService, SqliteDBConnection};
use apple_notes_rs_lib::notes::traits::identifyable_note::IdentifyableNote;
use crossterm::event::KeyEvent;
use tui::layout::Alignment;
use apple_notes_rs_lib::notes::localnote::LocalNote;

enum Event<I> {
    Input(I),
    Tick,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode().expect("can run in raw mode");
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.clear();

    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(1000);

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


    let db =  apple_notes_rs_lib::db::SqliteDBConnection::new();

    let mut entries = refetch_notes(&db);
    let mut items: Vec<ListItem> = generate_list_items(&entries);

    let mut list = gen_list(&mut items);

    let mut text: String = "".to_string();

    let mut note_list_state = ListState::default();
    note_list_state.select(Some(0));

    let mut scroll_amount = 0;

    loop {
        terminal.draw(|f| {

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


            let t2  = Paragraph::new("test")
                .block(Block::default().title("Status").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: true });

            f.render_widget(t, noteslayout[1]);
            f.render_widget(t2, chunks[1]);
        });

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
                },
                KeyCode::Char('d') => {
                    let mut note = entries.get(note_list_state.selected().unwrap()).unwrap().clone();
                    note.metadata.locally_deleted = !note.metadata.locally_deleted ;
                    let db_connection = apple_notes_rs_lib::db::SqliteDBConnection::new();
                    db_connection.update(&note).unwrap();
                    entries = refetch_notes(&db_connection);
                    items = generate_list_items(&entries);
                    list = gen_list(&mut items);

                },
                KeyCode::Char('q') => {
                    terminal.clear();
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

fn gen_list(mut items: &mut Vec<ListItem<'static>>) -> List<'static> {
    List::new(items.clone())
        .block(Block::default().title("List").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
        .highlight_symbol(">>")
}

fn refetch_notes(db: &SqliteDBConnection) -> Vec<LocalNote> {
    db.fetch_all_notes().unwrap()
        .into_iter()
        .sorted_by_key(|note| format!("{}_{}", &note.metadata.subfolder, &note.body[0].subject()))
        .collect()
}

fn generate_list_items(entries: &Vec<LocalNote>) -> Vec<ListItem<'static>> {
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