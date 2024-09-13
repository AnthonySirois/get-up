mod notification;

use std::{default, io, thread, time::Duration};
use crossterm::{event::KeyEvent, terminal};
use ratatui::{
    crossterm::event::{self, KeyCode, KeyEventKind}, 
    style::{Color, Style, Stylize}, 
    layout::Alignment,
    widgets::{block::{Title, Position}, Block, LineGauge},
    text::Line, 
    Frame, 
    symbols::border,
    prelude::{symbols, Layout, Direction, Constraint},
};

#[derive(Debug, Default)]
struct Model {
    state : State,
    timer_state : TimerState,
    sitting_duration : Duration,
    standing_duration : Duration,

    running_state : RunningState
}

#[derive(Debug, Default, PartialEq, Eq)]
enum RunningState {
    #[default]
    Running,
    Done
}

#[derive(Debug, Default)]
enum State {
    #[default] 
    Sitting,
    Standing
}

#[derive(Debug, Default)]
enum TimerState {
    #[default]
    InProgress,
    Paused
}

enum Message {
    Increase,
    Decrease,
    Reset,
    Next,
    Quit,
    Pause,
    Resume,
    Navigate
}

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let _ = terminal.clear()?;

    let mut model = Model::default();

    while model.running_state != RunningState::Done {
        terminal.draw(|frame| view(&model, frame))?;

        let mut current_message = handle_events(&model)?;

        while current_message.is_some() {
            current_message = update(&mut model, current_message.unwrap());
        }
    }

    ratatui::restore();
    
    Ok(())
}

fn view(model : &Model, frame : &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .spacing(1)
        .split(frame.area());

    let option_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50)
            ])
            .spacing(1)
            .split(chunks[1]);

    let progress_title = Title::from(" GET UP : Standing until X ".bold());
    let progress_instructions = Title::from(Line::from(vec![
        " Quit ".into(),
        "<Q>".blue().bold(),
        " Pause/Resume ".into(),
        "<Space>".blue().bold(),
        " Restart ".into(),
        "<H>".blue().into(),
        " Next ".into(),
        "<L> ".blue().into()
    ]));
    let progress_block = Block::bordered()
                .title(progress_title.alignment(Alignment::Center))
                .title(
                    progress_instructions
                        .alignment(Alignment::Center)
                        .position(Position::Bottom
                        ))
                .border_set(border::THICK);

    frame.render_widget(
        LineGauge::default()
            .block(progress_block)
            .filled_style(Style::default().fg(Color::Green))
            .line_set(symbols::line::DOUBLE)
            .label("[PAUSED] 15m13")
            .ratio(0.4), 
        chunks[0]
    );
    

    let option_instructions = Title::from(Line::from(vec![
        " Decrease ".into(),
        "<H>".blue().into(),
        " Increase ".into(),
        "<L> ".blue().into(),
    ]))
        .alignment(Alignment::Center)
        .position(Position::Bottom);

    let sitting_option_title = Title::from(" Sitting duration ".bold());
    let sitting_option_block = Block::bordered()
        .title(sitting_option_title.alignment(Alignment::Center))
        .title(option_instructions.clone())
        .border_set(border::THICK);

    frame.render_widget(
        LineGauge::default()
            .block(sitting_option_block)
            .filled_style(Style::default().fg(Color::Blue))
            .line_set(symbols::line::NORMAL)
            .label("1h00m")
            .ratio(0.25), 
        option_chunks[0]
    );

    let standing_option_title = Title::from(" Standing duration ".bold());
    let standing_option_block = Block::bordered()
        .title(standing_option_title.alignment(Alignment::Center))
        .title(option_instructions.clone())
        .border_set(border::THICK);

    frame.render_widget(
        LineGauge::default()
            .block(standing_option_block)
            .filled_style(Style::default().fg(Color::Blue))
            .line_set(symbols::line::NORMAL)
            .label("0h30m")
            .ratio(0.25), 
        option_chunks[1]
    );
}

fn handle_events(model : &Model) -> io::Result<Option<Message>> {
    if event::poll(Duration::from_millis(250))? {
        if let event::Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(Some(Message::Quit)),
                    KeyCode::Char(' ') => return Ok(Some(Message::Resume)), // TODO Depend on state
                    _ => {}
                }
            }
        }
    }
    Ok(None)
}

fn update(model : &mut Model, message : Message) -> Option<Message> {
    match message {
        Message::Quit => model.running_state = RunningState::Done,
        _ => {}
    }

    None
}

// ------------------------------------------------------------------------------

fn start_timer(schedule : Box<dyn Schedule>, verbose : bool, sitting : bool) { 
    let mut sitting: bool = sitting;

    loop {
        let sleep_duration = if sitting { schedule.get_sitting_duration() } else { schedule.get_standing_duration() };

        if verbose {
            let wake_up_time = format_time_after_duration(sleep_duration);
            if sitting {
                println!("Sitting until {wake_up_time}");
            } else {
                println!("Standing until {wake_up_time}");
            }
        }
        
        if sitting {
            notification::send_sit_notification(sleep_duration);
        } else {
            notification::send_stand_notification(sleep_duration);
        }

        thread::sleep(sleep_duration);

        sitting = !sitting;
    }
}

const LONG_TIME_FORMAT : &str = "%H:%M:%S";

fn format_time_after_duration(duration : Duration) -> String {
    let sleep_time = duration.as_secs();

    let wait_time_delta = chrono::TimeDelta::try_seconds(sleep_time.try_into().unwrap_or_default()).unwrap_or_default();
    let sleep_end_time = chrono::Local::now().checked_add_signed(wait_time_delta).unwrap_or_default();
    
    sleep_end_time.format(LONG_TIME_FORMAT).to_string()
}

trait Schedule {
    fn get_sitting_duration(&self) -> Duration;
    fn get_standing_duration(&self) -> Duration;
}

struct FixedSchedule {
    sitting_duration : Duration,
    standing_duration : Duration,
}

impl Schedule for FixedSchedule {
    fn get_sitting_duration(&self) -> Duration {
        self.sitting_duration
    }

    fn get_standing_duration(&self) -> Duration {
        self.standing_duration
    }
}