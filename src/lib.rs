extern crate signal_hook;

use signal_hook::consts::SIGINT;
use signal_hook::iterator::Signals;
use std::env;
use std::ffi::OsString;
use std::io::{self, Write};
use std::path::Path;
use std::process::{self, Child, Command};

pub fn run() {
    let mut signals = Signals::new(&[SIGINT]).expect("Error setting up the signal handler");
    let mut multiline_input = String::new();

    loop {
        if signals.pending().any(|signal| signal == SIGINT) {
            continue;
        }

        let prompt = if multiline_input.is_empty() {
            "> "
        } else {
            "  "
        };
        print!("{}", prompt);
        let _ = io::stdout().flush();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if multiline_input.is_empty() {
            multiline_input = input.to_string();
        } else {
            multiline_input.push_str("\n");
            multiline_input.push_str(input);
        }

        if !input.ends_with('\\') {
            if input.starts_with('#') {
                multiline_input.clear();
                continue;
            }

            let mut commands = multiline_input.split(" | ").peekable();
            let mut previous_command = None;
            let mut is_background = false;

            while let Some(command) = commands.next() {
                let mut part = command.trim().split_whitespace();
                let command = part.next().unwrap();
                let mut args = part
                    .map(|arg| {
                        let modified_arg = OsString::from(arg.replace("\\", ""));
                        modified_arg
                    })
                    .filter(|x| !x.is_empty())
                    .collect::<Vec<OsString>>();
                if Some(OsString::from("&")) == args.last().cloned() {
                    is_background = true;
                    args.pop();
                }

                match command {
                    "cd" => {
                        let new_dir = args
                            .iter()
                            .peekable()
                            .peek()
                            .map_or("/", |x| x.as_os_str().to_str().unwrap_or_default()); // x.to_str());
                        let root = Path::new(new_dir);
                        if let Err(e) = env::set_current_dir(&root) {
                            eprintln!("{}", e);
                        }
                        previous_command = None;
                    }
                    "pwd" => {
                        if let Ok(current_dir) = env::current_dir() {
                            println!("{}", current_dir.display());
                        } else {
                            eprintln!("Failed to get current working directory");
                        }
                        previous_command = None;
                    }
                    "clear" => print!("{}[2J{}[1;1H", 27 as char, 27 as char),
                    "exit" => return,
                    command => {
                        let stdin = previous_command
                            .map_or(process::Stdio::inherit(), |output: Child| {
                                process::Stdio::from(output.stdout.unwrap())
                            });
                        let stdout = if commands.peek().is_some() {
                            process::Stdio::piped()
                        } else {
                            process::Stdio::inherit()
                        };

                        let output = Command::new(command)
                            .args(args)
                            .stdin(stdin)
                            .stdout(stdout)
                            .spawn();

                        match output {
                            Ok(output) => {
                                previous_command = Some(output);
                            }
                            Err(e) => {
                                previous_command = None;
                                eprintln!("Error: {}", e);
                            }
                        }
                    }
                }
            }

            if let Some(mut final_command) = previous_command {
                if !is_background {
                    let _ = final_command.wait().unwrap();
                } else {
                    println!("{} started!", final_command.id());
                }
            }

            multiline_input.clear();
        }
    }
}
