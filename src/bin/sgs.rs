use argh::FromArgs;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use set_game_solver::{Card, Table};
use std::{
    borrow::Cow,
    error::Error,
    io,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};

pub fn draw<B>(f: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    const CARD_HEIGHT: u16 = 5;
    let mut rows = Layout::default()
        .constraints(
            [
                Constraint::Length(CARD_HEIGHT),
                Constraint::Length(CARD_HEIGHT),
                Constraint::Length(CARD_HEIGHT),
                Constraint::Min(0),
            ]
            .as_ref(),
        )
        .split(f.size());
    rows.pop();
    let mut card_tiles = Vec::new();
    for row in rows.into_iter() {
        const CARD_WIDTH: u16 = 12;
        let mut columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(CARD_WIDTH),
                Constraint::Length(CARD_WIDTH),
                Constraint::Length(CARD_WIDTH),
                Constraint::Length(CARD_WIDTH),
                Constraint::Length(CARD_WIDTH),
                Constraint::Length(CARD_WIDTH),
                Constraint::Length(CARD_WIDTH),
                Constraint::Min(0),
            ])
            .split(row);
        columns.pop();
        for column in columns.into_iter() {
            card_tiles.push(column);
        }
    }
    draw_cards(f, app, card_tiles);
}

fn draw_cards<B>(f: &mut Frame<B>, app: &mut App, tiles: Vec<Rect>)
where
    B: Backend,
{
    let cards = app.cards.iter().map(Some).chain(std::iter::repeat(None));
    for (i, (tile, card)) in tiles.into_iter().zip(cards).enumerate() {
        draw_card(f, tile, card, format!("{}", i));
    }
}

fn draw_card<B>(f: &mut Frame<B>, area: Rect, card: Option<&Card>, title: String)
where
    B: Backend,
{
    let block = Block::default().borders(Borders::ALL).title(Span::styled(
        title,
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
    ));
    let text = vec![Spans::from(vec![
        Span::from("          "),
        Span::from("   "),
        Span::styled(
            card.map(|card| card.to_string()).unwrap_or_default(),
            Style::default().fg(Color::Red),
        ),
    ])];
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);
}

pub struct App<'a> {
    pub title: &'a str,
    pub seed: u64,
    pub cards: Vec<Card>,
    pub selected_card: u8,
    pub should_quit: bool,
    pub table: Table,
}

impl<'a> App<'a> {
    pub fn new(seed: u64) -> App<'a> {
        let mut table = Table::new_from_seed(seed);
        table.deal();
        let mut board = table.board_mut();
        match table.deal() {
            Some(card) => board.push(card),
            None => {}
        };
        let cards = table.board().clone();
        App {
            seed,
            cards,
            selected_card: 0,
            should_quit: false,
            title: "foo",
            table,
        }
    }

    pub fn on_up(&mut self) {
        unimplemented!()
    }

    pub fn on_down(&mut self) {
        unimplemented!()
    }

    pub fn on_right(&mut self) {
        unimplemented!()
    }

    pub fn on_left(&mut self) {
        unimplemented!()
    }

    pub fn on_key(&mut self, c: char) {
        match c {
            'q' => {
                self.should_quit = true;
            }
            _ => {}
        }
    }

    pub fn on_tick(&mut self) {}
}

pub fn run(seed: u64) -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout
        .execute(EnterAlternateScreen)?
        .execute(EnableMouseCapture)?;
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new(seed);
    let res = run_app(&mut terminal, app, Duration::from_millis(250));

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| draw(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char(c) => app.on_key(c),
                    KeyCode::Left => app.on_left(),
                    KeyCode::Up => app.on_up(),
                    KeyCode::Right => app.on_right(),
                    KeyCode::Down => app.on_down(),
                    _ => {}
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
        if app.should_quit {
            return Ok(());
        }
    }
}

/// Demo
#[derive(Debug, FromArgs)]
struct Cli {
    /// random seed for game
    #[argh(option)]
    seed: Option<u64>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli: Cli = argh::from_env();
    let tick_rate = Duration::from_millis(250);
    let seed = cli.seed.unwrap_or_else(|| {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        rng.gen()
    });
    run(seed)?;
    Ok(())
}
