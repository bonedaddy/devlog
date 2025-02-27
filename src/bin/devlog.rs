extern crate clap;
extern crate devlog;

use clap::{Arg, ArgMatches, Command};
use devlog::{editor, hook, rollover, status, Config, Error, LogRepository, TaskStatus};
use std::fs::File;
use std::io::{copy, stdin, stdout, Write};
use std::process::exit;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const MAIN_INFO: &str =
    "Devlog files are created in the directory at $DEVLOG_REPO, which defaults to $HOME/devlogs if not set.";

const EDIT_INFO: &str =
    "Uses the editor program $DEVLOG_EDITOR, which defaults to nano if not set.";

fn main() -> Result<(), Error> {
    let yes_arg = Arg::new("yes")
        .short('y')
        .long("yes")
        .help("Automatically answer \"yes\" in response to all prompts.");

    let m = Command::new("devlog")
        .about("Track daily development work")
        .after_help(MAIN_INFO)
        .version(VERSION)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("init")
                .about("Initialize a new devlog repository if it does not already exist.")
                .arg(yes_arg.clone()),
        )
        .subcommand(
            Command::new("edit")
                .about("Edit the most recent devlog file")
                .after_help(EDIT_INFO)
                .arg(yes_arg.clone()),
        )
        .subcommand(
            Command::new("rollover")
                .about("Create new devlog file with incomplete and blocked tasks from the current devlog file")
                .arg(yes_arg.clone()),
        )
        .subcommand(
            Command::new("status")
                .about("Show recent tasks")
                .arg(
                    Arg::new("show")
                        .short('s')
                        .long("show")
                        .takes_value(true)
                        .value_name("SHOW")
                        .possible_values(&["all", "todo", "started", "blocked", "done"])
                        .default_value("all")
                        .help("Sections to show"),
                )
                .arg(
                    Arg::new("back")
                        .short('b')
                        .long("back")
                        .takes_value(true)
                        .value_name("BACK")
                        .default_value("0")
                        .help("Show tasks from a previous devlog"),
                ),
        )
        .subcommand(
            Command::new("tail")
                .about("Show recent devlogs")
                .arg(
                    Arg::new("limit")
                        .short('n')
                        .long("limit")
                        .takes_value(true)
                        .value_name("LIMIT")
                        .help("Maximum number of log files to display")
                        .default_value("2"),
                ),
        )
        .get_matches();

    let mut w = stdout();
    match m.subcommand() {
        Some(("init", m)) => init_cmd(&mut w, m),
        Some(("edit", m)) => edit_cmd(&mut w, m),
        Some(("rollover", m)) => rollover_cmd(&mut w, m),
        Some(("status", m)) => status_cmd(&mut w, m),
        Some(("tail", m)) => tail_cmd(&mut w, m),
        _ => panic!("No subcommand"),
    }
}

fn prompt_confirm<W: Write>(w: &mut W, msg: &str, m: &ArgMatches) -> Result<bool, Error> {
    if m.is_present("yes") {
        return Ok(true);
    }

    write!(w, "{} [y/n] ", msg)?;
    w.flush()?;

    let mut input = String::new();
    stdin()
        .read_line(&mut input)
        .map(|_| {
            let s = input.trim().to_lowercase();
            s == "yes" || s == "y"
        })
        .map_err(From::from)
}

fn abort_if_not_initialized<W: Write>(w: &mut W, repo: &LogRepository) -> Result<(), Error> {
    if !repo.initialized()? {
        write!(w, "Repository at {:?} has not been initialized.\nPlease run `devlog init` to initialize the repository.\n", repo.path())?;
        exit(1);
    }
    Ok(())
}

fn initialize_if_necessary<W: Write>(
    w: &mut W,
    repo: &LogRepository,
    m: &ArgMatches,
) -> Result<bool, Error> {
    if repo.initialized()? {
        Ok(false)
    } else {
        let msg = format!("Initialize devlog repository at {:?}?", repo.path());
        if prompt_confirm(w, &msg, m)? {
            repo.init()?;
            hook::init_hooks(repo.path())?;
        } else {
            exit(0);
        }
        Ok(true)
    }
}

fn init_cmd<W: Write>(w: &mut W, m: &ArgMatches) -> Result<(), Error> {
    let config = Config::load();
    let repo = LogRepository::new(config.repo_dir());
    initialize_if_necessary(w, &repo, m).and_then(|created| {
        if created {
            writeln!(
                w,
                "Success!  Now you can open your devlog using `devlog edit`",
            )
            .map_err(From::from)
        } else {
            writeln!(w, "Devlog repository already exists at {:?}", repo.path()).map_err(From::from)
        }
    })
}

fn edit_cmd<W: Write>(w: &mut W, m: &ArgMatches) -> Result<(), Error> {
    let config = Config::load();
    let repo = LogRepository::new(config.repo_dir());
    initialize_if_necessary(w, &repo, m).and_then(|_| match repo.latest()? {
        Some(logpath) => editor::open(w, &config, logpath.path()),
        None => {
            // The user already confirmed initialization of the repo,
            // so if we don't find it we initialize it again to ensure it exists.
            repo.init()
                .and_then(|logpath| editor::open(w, &config, logpath.path()))
        }
    })
}

fn rollover_cmd<W: Write>(w: &mut W, m: &ArgMatches) -> Result<(), Error> {
    let config = Config::load();
    let repo = LogRepository::new(config.repo_dir());
    abort_if_not_initialized(w, &repo).and_then(|()| {
        match repo.latest()? {
            Some(p) => {
                if prompt_confirm(w, "Rollover incomplete tasks?", m)? {
                    let (logpath, count) = rollover::rollover(w, &config, &p)?;
                    writeln!(w, "Imported {} tasks into {:?}", count, logpath.path())?;
                }
                Ok(())
            }
            None => {
                // This will only occur if something deleted the repo
                // right after we checked that it was initialized (unlikely)
                writeln!(w, "Could not find devlog file to rollover")?;
                exit(1)
            }
        }
    })
}

fn status_cmd<W: Write>(w: &mut W, m: &ArgMatches) -> Result<(), Error> {
    let num_back = m
        .value_of("back")
        .unwrap()
        .parse::<usize>()
        .map_err(|_| Error::InvalidArg("back must be an integer"))?;

    let display_mode = match m.value_of("show") {
        Some("all") => status::DisplayMode::ShowAll,
        Some("todo") => status::DisplayMode::ShowOnly(TaskStatus::ToDo),
        Some("started") => status::DisplayMode::ShowOnly(TaskStatus::Started),
        Some("blocked") => status::DisplayMode::ShowOnly(TaskStatus::Blocked),
        Some("done") => status::DisplayMode::ShowOnly(TaskStatus::Done),
        _ => panic!("Invalid value for show arg"),
    };

    let config = Config::load();
    let repo = LogRepository::new(config.repo_dir());
    abort_if_not_initialized(w, &repo).and_then(|_| status::print(w, &repo, num_back, display_mode))
}

fn parse_limit_arg(m: &ArgMatches) -> Result<usize, Error> {
    let limit = m
        .value_of("limit")
        .unwrap()
        .parse::<usize>()
        .map_err(|_| Error::InvalidArg("limit must be an integer"))?;
    if limit < 1 {
        Err(Error::InvalidArg("limit must be >= 1"))
    } else {
        Ok(limit)
    }
}

fn tail_cmd<W: Write>(w: &mut W, m: &ArgMatches) -> Result<(), Error> {
    let limit = parse_limit_arg(m)?;
    let config = Config::load();
    let repo = LogRepository::new(config.repo_dir());
    abort_if_not_initialized(w, &repo).and_then(|_| {
        let paths = repo.tail(limit)?;
        for (i, logpath) in paths.iter().enumerate() {
            if i > 0 {
                write!(w, "\n~~~~~~~~~~~~~~~~~~~~~~\n")?;
            }
            let mut f = File::open(logpath.path())?;
            copy(&mut f, w)?;
        }
        Ok(())
    })
}
