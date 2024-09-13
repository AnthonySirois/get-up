mod notification;

use crossterm::{event::KeyEvent, terminal};
use ratatui::{
    crossterm::event::{self, KeyCode, KeyEventKind},
    layout::Alignment,
    prelude::{symbols, Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{
        block::{Position, Title},
        Block, LineGauge, Padding,
    },
    Frame,
};
use std::{default, io, thread, time::Duration};

const INCREASE_STEP_DURATION: Duration = Duration::from_secs(300);

#[derive(Debug, Default)]
struct Model {
    state: State,
    timer_state: TimerState,
    sitting_duration: Duration,
    standing_duration: Duration,

    running_state: RunningState,
    selected_widget_block: WidgetBlock,
}

#[derive(Debug, Default, PartialEq, Eq)]
enum RunningState {
    #[default]
    Running,
    Done,
}

#[derive(Debug, Default)]
enum State {
    #[default]
    Sitting,
    Standing,
}

#[derive(Debug, Default, PartialEq, Eq)]
enum TimerState {
    #[default]
    InProgress,
    Paused,
}

#[derive(Debug, Default, PartialEq, Eq)]
enum WidgetBlock {
    #[default]
    Timer,
    SittingOption,
    StandingOption,
}

enum Message {
    Increase,
    Decrease,
    Reset,
    Next,
    Quit,
    Pause,
    Resume,
    Navigate,
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

fn view(model: &Model, frame: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .spacing(1)
        .split(frame.area());

    let option_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .spacing(1)
        .split(chunks[1]);

    let progress_title = Title::from(" GET UP : Standing until X ".bold());
    let progress_instructions = Title::from(Line::from(vec![
        " Quit ".into(),
        "<Q>".blue().bold(),
        " Pause/Resume ".into(),
        "<Space>".blue().bold(),
        " Restart ".into(),
        "<H>".blue().bold(),
        " Next ".into(),
        "<L> ".blue().bold(),
    ]));
    let progress_block = Block::bordered()
        .title(progress_title.alignment(Alignment::Center))
        .title(
            progress_instructions
                .alignment(Alignment::Center)
                .position(Position::Bottom),
        )
        .padding(Padding::uniform(1))
        .border_set(border::THICK);

    frame.render_widget(
        LineGauge::default()
            .block(progress_block)
            .filled_style(Style::default().fg(Color::Green))
            .line_set(symbols::line::DOUBLE)
            .label("[PAUSED] 15m13")
            .ratio(0.4),
        chunks[0],
    );

    let option_instructions = Title::from(Line::from(vec![
        " Decrease ".into(),
        "<H>".blue().bold(),
        " Increase ".into(),
        "<L> ".blue().bold(),
    ]))
    .alignment(Alignment::Center)
    .position(Position::Bottom);

    let sitting_option_title = Title::from(" Sitting duration ".bold());
    let sitting_option_block = Block::bordered()
        .title(sitting_option_title.alignment(Alignment::Center))
        .title(option_instructions.clone())
        .padding(Padding::uniform(1))
        .border_set(border::THICK);

    frame.render_widget(
        LineGauge::default()
            .block(sitting_option_block)
            .filled_style(Style::default().fg(Color::Blue))
            .line_set(symbols::line::NORMAL)
            .label("1h00m")
            .ratio(0.25),
        option_chunks[0],
    );

    let standing_option_title = Title::from(" Standing duration ".bold());
    let standing_option_block = Block::bordered()
        .title(standing_option_title.alignment(Alignment::Center))
        .title(option_instructions.clone())
        .padding(Padding::uniform(1))
        .border_set(border::THICK);

    frame.render_widget(
        LineGauge::default()
            .block(standing_option_block)
            .filled_style(Style::default().fg(Color::Blue))
            .line_set(symbols::line::NORMAL)
            .label("0h30m")
            .ratio(0.25),
        option_chunks[1],
    );
}

fn handle_events(model: &Model) -> io::Result<Option<Message>> {
    if event::poll(Duration::from_millis(250))? {
        if let event::Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(Some(Message::Quit)),
                    KeyCode::Char(' ') => {
                        return Ok(Some(if model.timer_state == TimerState::Paused {
                            Message::Resume
                        } else {
                            Message::Pause
                        }))
                    }
                    KeyCode::Tab => return Ok(Some(Message::Navigate)),
                    KeyCode::Char('h') | KeyCode::Char('H') => {
                        return Ok(Some(if model.selected_widget_block == WidgetBlock::Timer {
                            Message::Reset
                        } else {
                            Message::Decrease
                        }))
                    }
                    KeyCode::Char('l') | KeyCode::Char('L') => {
                        return Ok(Some(if model.selected_widget_block == WidgetBlock::Timer {
                            Message::Next
                        } else {
                            Message::Increase
                        }))
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(None)
}

fn update(model: &mut Model, message: Message) -> Option<Message> {
    match message {
        Message::Quit => model.running_state = RunningState::Done,
        Message::Increase => match model.selected_widget_block {
            WidgetBlock::SittingOption => {
                model.sitting_duration = model
                    .sitting_duration
                    .saturating_add(INCREASE_STEP_DURATION)
            }
            WidgetBlock::StandingOption => {
                model.standing_duration = model
                    .standing_duration
                    .saturating_add(INCREASE_STEP_DURATION)
            }
            _ => {}
        },
        Message::Decrease => match model.selected_widget_block {
            WidgetBlock::SittingOption => {
                model.sitting_duration = model
                    .sitting_duration
                    .saturating_sub(INCREASE_STEP_DURATION)
            }
            WidgetBlock::StandingOption => {
                model.standing_duration = model
                    .standing_duration
                    .saturating_sub(INCREASE_STEP_DURATION)
            }
            _ => {}
        },
        Message::Pause => model.timer_state = TimerState::Paused,
        Message::Resume => model.timer_state = TimerState::InProgress,
        Message::Navigate => {
            model.selected_widget_block = match model.selected_widget_block {
                WidgetBlock::Timer => WidgetBlock::SittingOption,
                WidgetBlock::SittingOption => WidgetBlock::StandingOption,
                WidgetBlock::StandingOption => WidgetBlock::Timer,
            }
        }
        _ => {}
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_quit() {
        let mut model = Model::default();

        update(&mut model, Message::Quit);

        assert_eq!(model.running_state, RunningState::Done);
    }

    #[test]
    fn test_update_increase_sitting() {
        let mut model = Model::default();
        model.sitting_duration = Duration::from_secs(1800);
        model.selected_widget_block = WidgetBlock::SittingOption;

        update(&mut model, Message::Increase);

        assert_eq!(model.sitting_duration, Duration::from_secs(2100));
    }

    #[test]
    fn test_update_increase_standing() {
        let mut model = Model::default();
        model.standing_duration = Duration::from_secs(1800);
        model.selected_widget_block = WidgetBlock::StandingOption;

        update(&mut model, Message::Increase);

        assert_eq!(model.standing_duration, Duration::from_secs(2100));
    }

    #[test]
    fn test_update_decrease_sitting() {
        let mut model = Model::default();
        model.sitting_duration = Duration::from_secs(1800);
        model.selected_widget_block = WidgetBlock::SittingOption;

        update(&mut model, Message::Decrease);

        assert_eq!(model.sitting_duration, Duration::from_secs(1500));
    }

    #[test]
    fn test_update_decrease_standing() {
        let mut model = Model::default();
        model.standing_duration = Duration::from_secs(1800);
        model.selected_widget_block = WidgetBlock::StandingOption;

        update(&mut model, Message::Decrease);

        assert_eq!(model.standing_duration, Duration::from_secs(1500));
    }

    #[test]
    fn test_update_pause() {
        let mut model = Model::default();
        model.timer_state = TimerState::InProgress;

        update(&mut model, Message::Pause);

        assert_eq!(model.timer_state, TimerState::Paused);
    }

    #[test]
    fn test_update_resume() {
        let mut model = Model::default();
        model.timer_state = TimerState::Paused;

        update(&mut model, Message::Resume);

        assert_eq!(model.timer_state, TimerState::InProgress);
    }

    #[test]
    fn test_update_navigate_timer_block() {
        let mut model = Model::default();
        model.selected_widget_block = WidgetBlock::Timer;

        update(&mut model, Message::Navigate);

        assert_eq!(model.selected_widget_block, WidgetBlock::SittingOption);
    }

    #[test]
    fn test_update_navigate_sitting_option_block() {
        let mut model = Model::default();
        model.selected_widget_block = WidgetBlock::SittingOption;

        update(&mut model, Message::Navigate);

        assert_eq!(model.selected_widget_block, WidgetBlock::StandingOption);
    }

    #[test]
    fn test_update_navigate_standing_option_block() {
        let mut model = Model::default();
        model.selected_widget_block = WidgetBlock::StandingOption;

        update(&mut model, Message::Navigate);

        assert_eq!(model.selected_widget_block, WidgetBlock::Timer);
    }
}

// ------------------------------------------------------------------------------

fn start_timer(schedule: Box<dyn Schedule>, verbose: bool, sitting: bool) {
    let mut sitting: bool = sitting;

    loop {
        let sleep_duration = if sitting {
            schedule.get_sitting_duration()
        } else {
            schedule.get_standing_duration()
        };

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

const LONG_TIME_FORMAT: &str = "%H:%M:%S";

fn format_time_after_duration(duration: Duration) -> String {
    let sleep_time = duration.as_secs();

    let wait_time_delta = chrono::TimeDelta::try_seconds(sleep_time.try_into().unwrap_or_default())
        .unwrap_or_default();
    let sleep_end_time = chrono::Local::now()
        .checked_add_signed(wait_time_delta)
        .unwrap_or_default();

    sleep_end_time.format(LONG_TIME_FORMAT).to_string()
}

trait Schedule {
    fn get_sitting_duration(&self) -> Duration;
    fn get_standing_duration(&self) -> Duration;
}

struct FixedSchedule {
    sitting_duration: Duration,
    standing_duration: Duration,
}

impl Schedule for FixedSchedule {
    fn get_sitting_duration(&self) -> Duration {
        self.sitting_duration
    }

    fn get_standing_duration(&self) -> Duration {
        self.standing_duration
    }
}
