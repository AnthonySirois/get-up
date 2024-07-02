mod notification;

use std::{ops::Range, thread, time::Duration};
use clap::{Args, Parser, Subcommand};
use rand::{distributions::{Distribution, Uniform}, thread_rng};

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[arg(short, long)]
    verbose : bool,

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
    let cli = Cli::parse();
    
    let schedule = parse_command(cli.command);

    start_timer(schedule, cli.verbose);
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

fn start_timer(schedule : Box<dyn Schedule>, verbose : bool) { 
    let mut sitting: bool = true;

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

        thread::sleep(sleep_duration);

        if sitting {
            notification::send_stand_notification();
        } else {
            notification::send_sit_notification();
        }

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