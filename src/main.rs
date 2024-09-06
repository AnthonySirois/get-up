mod notification;

use std::{ops::Range, thread, time::Duration, io};
use clap::{Args, Parser, Subcommand};
use crossterm::{event::KeyEvent, terminal};
use rand::{distributions::{Distribution, Uniform}, thread_rng};
use ratatui::{
    crossterm::event::{self, KeyCode, KeyEventKind},
    style::Stylize,
    widgets::Paragraph,
    DefaultTerminal, Terminal
};

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[arg(short, long)]
    verbose : bool,

    #[arg(short, long, default_value_t = false, help = "Start the loop standing instead of sitting")]
    standing : bool,

    #[command(subcommand)]
    command : Commands
}

#[derive(Subcommand)]
enum Commands {
    Fixed(FixedArgs),
    Random(RandomArgs)
}

#[derive(Args)]
struct FixedArgs {
    seconds_sitting : u32,
    seconds_standing : u32,
}

#[derive(Args)]
struct RandomArgs {
    min_seconds_sitting : u32,
    max_seconds_sitting : u32,
    min_seconds_standing : u32,
    max_seconds_standing : u32,
}

fn main() {
    tui_main();
}

fn tui_main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let _ = terminal.clear();
    let app_result = run(terminal);
    ratatui::restore();
    app_result
}

fn run(mut terminal : DefaultTerminal) -> io::Result<()> {
    loop {
        terminal.draw(|frame| {
            let greeting = Paragraph::new("Hello ratatui! (press q to quit)")
                .white()
                .on_blue();

           frame.render_widget(greeting, frame.area());  
        })?;

        if let event::Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                return Ok(());
            }
        }
    }
}

fn cli_main() {
    let cli = Cli::parse();
    
    let schedule = parse_command(cli.command);

    start_timer(schedule, cli.verbose, !cli.standing);
}

fn parse_command(command : Commands) -> Box<dyn Schedule> {
    match command {
        Commands::Fixed(args) => {
            let schedule = FixedSchedule{
                sitting_duration : Duration::from_secs(args.seconds_sitting.into()),
                standing_duration : Duration::from_secs(args.seconds_standing.into()),
            };

            Box::new(schedule)
        },
        Commands::Random(args) => {
            let sitting_duration_range = Duration::from_secs(args.min_seconds_sitting.into())..Duration::from_secs(args.max_seconds_sitting.into());
            let standing_duration_range = Duration::from_secs(args.min_seconds_sitting.into())..Duration::from_secs(args.max_seconds_sitting.into());

            let schedule: RandomSchedule = RandomSchedule{
                sitting_duration_range,
                standing_duration_range
            };

            Box::new(schedule)
        },
    }
}

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

struct RandomSchedule {
    sitting_duration_range : Range<Duration>,
    standing_duration_range : Range<Duration>,
}
 
impl Schedule for RandomSchedule {
    fn get_sitting_duration(&self) -> Duration {
        let mut rng = thread_rng();
        let between = Uniform::from(self.sitting_duration_range.clone());
        between.sample(&mut rng)
    }

    fn get_standing_duration(&self) -> Duration {
        let mut rng = thread_rng();
        let between = Uniform::from(self.standing_duration_range.clone());
        between.sample(&mut rng)
    }
}