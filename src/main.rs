mod notification;

use std::{thread, time::Duration};
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
    
    match cli.command {
        Commands::Fixed(args) => { 
            fixed_schedule(args, cli.verbose)
        },
        Commands::Random(args) => {
            random_schedule(args, cli.verbose)
        },
    }
}

const LONG_TIME_FORMAT : &str = "%H:%M:%S";

fn fixed_schedule(args : FixedArgs, verbose : bool) {
    let mut sitting = true;

    let seconds_sitting = args.seconds_sitting;
    let seconds_standing = args.seconds_standing;

    loop {
        let wait_time = if sitting { seconds_sitting } else { seconds_standing };
        let sleep_duration = Duration::from_secs(wait_time.into());

        if verbose {
            let wait_time_delta = chrono::TimeDelta::try_seconds(wait_time.into()).unwrap_or_default();
            let sleep_end_time = chrono::Local::now().checked_add_signed(wait_time_delta).unwrap_or_default();
            println!("Time to awaken the empire : {}", sleep_end_time.format(LONG_TIME_FORMAT).to_string());
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

fn random_schedule(args : RandomArgs, verbose : bool) {
    let mut sitting = true;
    let mut rng = thread_rng();
    let between_sitting = Uniform::from(args.min_seconds_sitting..args.max_seconds_sitting);
    let between_standing = Uniform::from(args.min_seconds_standing..args.max_seconds_standing);

    loop {
        let wait_time = if sitting { between_sitting.sample(&mut rng) } else { between_standing.sample(&mut rng) };
        
        let sleep_duration = Duration::from_secs(wait_time.into());

        if verbose {
            let wait_time_delta = chrono::TimeDelta::try_seconds(wait_time.into()).unwrap_or_default();
            let sleep_end_time = chrono::Local::now().checked_add_signed(wait_time_delta).unwrap_or_default();
            println!("Time to awaken the empire : {}", sleep_end_time.format(LONG_TIME_FORMAT).to_string());
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