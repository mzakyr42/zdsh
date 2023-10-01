use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::env;
use std::ffi::OsString;
use std::path::Path;
use std::process::{self, Child, Command};

pub fn run() {
    let mut multiline_input = String::new();
    let mut rl = DefaultEditor::new().expect("cannot create the default editor");
    if rl
        .load_history(&format!("{}/.zdsh_history", env::var("HOME").unwrap()))
        .is_err()
    {
        println!("no previous history");
        std::fs::File::create(format!("{}/.zdsh_history", env::var("HOME").unwrap()))
            .expect("Couldn't create history file");
    }

    loop {
        let prompt = if multiline_input.is_empty() {
            "> "
        } else {
            "  "
        };
        // let _ = io::stdout().flush();
        //
        // let mut input = String::new();
        // io::stdin().read_line(&mut input).unwrap();
        // let input = input.trim();
        let input = rl.readline(prompt);

        match input {
            Ok(input) => {
                let _ = rl.add_history_entry(input.clone());

                if input.is_empty() {
                    continue;
                }

                if multiline_input.is_empty() {
                    multiline_input = input.to_string();
                } else {
                    multiline_input.push_str("\n");
                    multiline_input.push_str(&input);
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
                                let new_dir =
                                    args.iter().peekable().peek().map_or("/", |x| {
                                        x.as_os_str().to_str().unwrap_or_default()
                                    });
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
                            "clear" => rl.clear_screen().expect("cannot clear screen"),
                            "exit" => {
                                rl.save_history(&format!(
                                    "{}/.rush_history",
                                    env::var("HOME").unwrap()
                                ))
                                .expect("Couldn't save history");
                                std::process::exit(0);
                            }
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
            Err(ReadlineError::Interrupted) => {
                println!("Ctrl-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(e) => println!("{}", e),
        }
    }
}
