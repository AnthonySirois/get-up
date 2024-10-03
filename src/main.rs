mod notification;
mod pausable_timer;

use pausable_timer::Timer;
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
use std::{io, time::Duration};

const INCREASE_STEP_DURATION: Duration = Duration::from_secs(300);
const POLL_DURATION: Duration = Duration::from_millis(1000);
const MAX_DURATION: Duration = Duration::from_secs(14400);
const MIN_DURATION: Duration = Duration::from_secs(300); 
const DEFAULT_SITTING_DURATION: Duration = Duration::from_secs(3600);
const DEFAULT_STANDING_DURATION: Duration = Duration::from_secs(1800);

const TITLE_STYLE: Style = Style::new().fg(Color::LightCyan);
const SELECTED_STYLE: Style = Style::new().fg(Color::Rgb(202, 166, 247));
const UNSELECTED_STYLE: Style = Style::new().fg(Color::DarkGray);
const IN_PROGRESS_GAUGE_STYLE: Style = Style::new().fg(Color::Green);
const PAUSED_GAUGE_STYLE: Style = Style::new().fg(Color::Yellow);
const SETTINGS_GAUGE_STYLE: Style = Style::new().fg(Color::Blue);

#[derive(Debug, Default)]
struct Model {
    state: State,
    timer_state: TimerState,
    sitting_duration: Duration,
    standing_duration: Duration,

    running_state: RunningState,
    selected_widget_block: WidgetBlock,
    timer: Timer,
}

#[derive(Debug, Default, PartialEq, Eq)]
enum RunningState {
    #[default]
    Running,
    Done,
}

#[derive(Debug, Default, PartialEq, Eq)]
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
    SittingSettings,
    StandingSettings,
}

enum Message {
    Increase,
    Decrease,
    Reset,
    Next,
    Quit,
    Pause,
    Resume,
    NavigateForward,
    NavigateBackward,
    TimerFinished,
}

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let _ = terminal.clear()?;

    let mut model = Model::default();
    model.sitting_duration = DEFAULT_SITTING_DURATION;
    model.standing_duration = DEFAULT_STANDING_DURATION;

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

    let settings_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .spacing(1)
        .split(chunks[1]);

    let timer_duration = match model.state {
        State::Sitting => model.sitting_duration,
        State::Standing => model.standing_duration,
    };
    
    let ratio = (model.timer.elapsed().as_secs_f64() / timer_duration.as_secs_f64())
        .clamp(0.0, 1.0);
    let time_left = timer_duration.saturating_sub(model.timer.elapsed());

    let progress_title = Title::from(
        format!(
            " GET UP : {} until {} ",
            if model.state == State::Sitting {
                "Sitting"
            } else {
                "Standing"
            },
            format_time_after_duration(time_left)
        )
        .bold(),
    );
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
        .border_style(if model.selected_widget_block == WidgetBlock::Timer {
            SELECTED_STYLE
        } else {
            UNSELECTED_STYLE
        })
        .title_style(TITLE_STYLE)
        .border_set(border::THICK);

    frame.render_widget(
        LineGauge::default()
            .block(progress_block)
            .filled_style(if model.timer_state == TimerState::InProgress {
                IN_PROGRESS_GAUGE_STYLE
            } else {
                PAUSED_GAUGE_STYLE
            })
            .line_set(symbols::line::DOUBLE)
            .label(format!(
                "{} {}",
                if model.timer_state == TimerState::Paused {
                    "[PAUSED]"
                } else {
                    "        "
                },
                format_duration_hours_minutes_seconds(time_left)
            ))
            .ratio(ratio),
        chunks[0],
    );

    let settings_instructions = Title::from(Line::from(vec![
        " Decrease ".into(),
        "<H>".blue().bold(),
        " Increase ".into(),
        "<L> ".blue().bold(),
    ]))
    .alignment(Alignment::Center)
    .position(Position::Bottom);

    let sitting_settings_title = Title::from(" Sitting duration ".bold());
    let sitting_settings_block = Block::bordered()
        .title(sitting_settings_title.alignment(Alignment::Center))
        .title(settings_instructions.clone())
        .padding(Padding::uniform(1))
        .border_style(
            if model.selected_widget_block == WidgetBlock::SittingSettings {
                SELECTED_STYLE
            } else {
                UNSELECTED_STYLE
            },
        )
        .title_style(TITLE_STYLE)
        .border_set(border::THICK);

    frame.render_widget(
        LineGauge::default()
            .block(sitting_settings_block)
            .filled_style(SETTINGS_GAUGE_STYLE)
            .line_set(symbols::line::NORMAL)
            .label(format_duration_hours_minutes(model.sitting_duration))
            .ratio(ratio_duration(
                model.sitting_duration,
                MIN_DURATION,
                MAX_DURATION,
            )),
        settings_chunks[0],
    );

    let standing_settings_title = Title::from(" Standing duration ".bold());
    let standing_settings_block = Block::bordered()
        .title(standing_settings_title.alignment(Alignment::Center))
        .title(settings_instructions.clone())
        .padding(Padding::uniform(1))
        .border_style(
            if model.selected_widget_block == WidgetBlock::StandingSettings {
                SELECTED_STYLE
            } else {
                UNSELECTED_STYLE
            },
        )
        .title_style(TITLE_STYLE)
        .border_set(border::THICK);

    frame.render_widget(
        LineGauge::default()
            .block(standing_settings_block)
            .filled_style(SETTINGS_GAUGE_STYLE)
            .line_set(symbols::line::NORMAL)
            .label(format_duration_hours_minutes(model.standing_duration))
            .ratio(ratio_duration(
                model.standing_duration,
                MIN_DURATION,
                MAX_DURATION,
            )),
        settings_chunks[1],
    );
}

fn handle_events(model: &Model) -> io::Result<Option<Message>> {
    if let Some(message) = handle_async(model) {
        return Ok(Some(message));
    }

    if event::poll(POLL_DURATION)? {
        if let event::Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                if let Some(message) = handle_key(model, key) {
                    return Ok(Some(message));
                }
            }
        }
    }
    Ok(None)
}

fn handle_async(model: &Model) -> Option<Message> {
    let timer_duration = if model.state == State::Sitting {
        model.sitting_duration
    } else {
        model.standing_duration
    };

    if model.timer.elapsed() > timer_duration {
        return Some(Message::TimerFinished);
    }

    None
}

fn handle_key(model: &Model, key: crossterm::event::KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => Some(Message::Quit),
        KeyCode::Char(' ') => {
            if model.timer_state == TimerState::Paused {
                Some(Message::Resume)
            } else {
                Some(Message::Pause)
            }
        }
        KeyCode::Tab => Some(Message::NavigateForward),
        KeyCode::BackTab => Some(Message::NavigateBackward),
        KeyCode::Char('h') | KeyCode::Char('H') => {
            if model.selected_widget_block == WidgetBlock::Timer {
                Some(Message::Reset)
            } else {
                Some(Message::Decrease)
            }
        }
        KeyCode::Char('l') | KeyCode::Char('L') => {
            if model.selected_widget_block == WidgetBlock::Timer {
                Some(Message::Next)
            } else {
                Some(Message::Increase)
            }
        }
        _ => None,
    }
}

fn update(model: &mut Model, message: Message) -> Option<Message> {
    match message {
        Message::Quit => model.running_state = RunningState::Done,
        Message::Increase => match model.selected_widget_block {
            WidgetBlock::SittingSettings => {
                model.sitting_duration = model
                    .sitting_duration
                    .saturating_add(INCREASE_STEP_DURATION)
                    .clamp(MIN_DURATION, MAX_DURATION);
            }
            WidgetBlock::StandingSettings => {
                model.standing_duration = model
                    .standing_duration
                    .saturating_add(INCREASE_STEP_DURATION)
                    .clamp(MIN_DURATION, MAX_DURATION);
            }
            _ => {}
        },
        Message::Decrease => match model.selected_widget_block {
            WidgetBlock::SittingSettings => {
                model.sitting_duration = model
                    .sitting_duration
                    .saturating_sub(INCREASE_STEP_DURATION)
                    .clamp(MIN_DURATION, MAX_DURATION);
            }
            WidgetBlock::StandingSettings => {
                model.standing_duration = model
                    .standing_duration
                    .saturating_sub(INCREASE_STEP_DURATION)
                    .clamp(MIN_DURATION, MAX_DURATION);
            }
            _ => {}
        },
        Message::Pause => {
            model.timer_state = TimerState::Paused;
            model.timer.pause();
        }
        Message::Resume => {
            model.timer_state = TimerState::InProgress;
            model.timer.resume();
        }
        Message::NavigateForward => {
            model.selected_widget_block = match model.selected_widget_block {
                WidgetBlock::Timer => WidgetBlock::SittingSettings,
                WidgetBlock::SittingSettings => WidgetBlock::StandingSettings,
                WidgetBlock::StandingSettings => WidgetBlock::Timer,
            }
        }
        Message::NavigateBackward => {
            model.selected_widget_block = match model.selected_widget_block {
                WidgetBlock::Timer => WidgetBlock::StandingSettings,
                WidgetBlock::SittingSettings => WidgetBlock::Timer,
                WidgetBlock::StandingSettings => WidgetBlock::SittingSettings,
            }
        }
        Message::Next => {
            model.timer.reset();
            model.state = match model.state {
                State::Sitting => State::Standing,
                State::Standing => State::Sitting,
            };
        }
        Message::Reset => {
            model.timer.reset();
        }
        Message::TimerFinished => {
            model.timer.reset();
            model.state = match model.state {
                State::Sitting => State::Standing,
                State::Standing => State::Sitting,
            };

            match model.state {
                State::Sitting => notification::send_sit_notification(model.sitting_duration),
                State::Standing => notification::send_stand_notification(model.standing_duration),
            };
        }
    }

    None
}

fn ratio_duration(duration: Duration, min: Duration, max: Duration) -> f64 {
    (duration.as_secs_f64() - min.as_secs_f64()) / (max.as_secs_f64() - min.as_secs_f64())
}

fn format_duration_hours_minutes(duration: Duration) -> String {
    let hours = duration.as_secs() / 3600;
    let minutes = (duration.as_secs() / 60) % 60;

    format!("{}h{}m", hours, minutes)
}

fn format_duration_hours_minutes_seconds(duration: Duration) -> String {
    let hours = duration.as_secs() / 3600;
    let minutes = (duration.as_secs() / 60) % 60;
    let seconds = duration.as_secs() % 60;

    format!("{}h{}m{}s", hours, minutes, seconds)
}

const LONG_TIME_FORMAT: &str = "%H:%M:%S";

fn format_time_after_duration(duration: Duration) -> String {
    let sleep_time = duration.as_secs();

    let wait_time_delta: chrono::TimeDelta =
        chrono::TimeDelta::try_seconds(sleep_time.try_into().unwrap_or_default())
            .unwrap_or_default();
    let sleep_end_time = chrono::Local::now()
        .checked_add_signed(wait_time_delta)
        .unwrap_or_default();

    sleep_end_time.format(LONG_TIME_FORMAT).to_string()
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
        model.selected_widget_block = WidgetBlock::SittingSettings;

        update(&mut model, Message::Increase);

        assert_eq!(model.sitting_duration, Duration::from_secs(2100));
    }

    #[test]
    fn test_update_increase_standing() {
        let mut model = Model::default();
        model.standing_duration = Duration::from_secs(1800);
        model.selected_widget_block = WidgetBlock::StandingSettings;

        update(&mut model, Message::Increase);

        assert_eq!(model.standing_duration, Duration::from_secs(2100));
    }

    #[test]
    fn test_update_decrease_sitting() {
        let mut model = Model::default();
        model.sitting_duration = Duration::from_secs(1800);
        model.selected_widget_block = WidgetBlock::SittingSettings;

        update(&mut model, Message::Decrease);

        assert_eq!(model.sitting_duration, Duration::from_secs(1500));
    }

    #[test]
    fn test_update_decrease_standing() {
        let mut model = Model::default();
        model.standing_duration = Duration::from_secs(1800);
        model.selected_widget_block = WidgetBlock::StandingSettings;

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
    fn test_update_navigate_forward_timer_block() {
        let mut model = Model::default();
        model.selected_widget_block = WidgetBlock::Timer;

        update(&mut model, Message::NavigateForward);

        assert_eq!(model.selected_widget_block, WidgetBlock::SittingSettings);
    }

    #[test]
    fn test_update_navigate_forward_sitting_settings_block() {
        let mut model = Model::default();
        model.selected_widget_block = WidgetBlock::SittingSettings;

        update(&mut model, Message::NavigateForward);

        assert_eq!(model.selected_widget_block, WidgetBlock::StandingSettings);
    }

    #[test]
    fn test_update_navigate_forward_standing_settings_block() {
        let mut model = Model::default();
        model.selected_widget_block = WidgetBlock::StandingSettings;

        update(&mut model, Message::NavigateForward);

        assert_eq!(model.selected_widget_block, WidgetBlock::Timer);
    }

    #[test]
    fn test_update_navigate_backward_timer_block() {
        let mut model = Model::default();
        model.selected_widget_block = WidgetBlock::Timer;

        update(&mut model, Message::NavigateBackward);

        assert_eq!(model.selected_widget_block, WidgetBlock::StandingSettings);
    }

    #[test]
    fn test_update_navigate_backward_sitting_settings_block() {
        let mut model = Model::default();
        model.selected_widget_block = WidgetBlock::SittingSettings;

        update(&mut model, Message::NavigateBackward);

        assert_eq!(model.selected_widget_block, WidgetBlock::Timer);
    }

    #[test]
    fn test_update_navigate_backward_standing_settings_block() {
        let mut model = Model::default();
        model.selected_widget_block = WidgetBlock::StandingSettings;

        update(&mut model, Message::NavigateBackward);

        assert_eq!(model.selected_widget_block, WidgetBlock::SittingSettings);
    }
}
