use std::env;
use std::io::{self, Write};
use std::path::Path;
use std::process::{self, Child, Command};

fn main() {
    loop {
        print!("> ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let mut commands = input.trim().split(" | ").peekable();
        let mut previous_command = None;

        while let Some(command) = commands.next() {
            let mut part = command.trim().split_whitespace();
            let command = part.next().unwrap();
            let args = part;

            match command {
                "cd" => {
                    let new_dir = args.peekable().peek().map_or("/", |x| *x);
                    let root = Path::new(new_dir);
                    if let Err(e) = env::set_current_dir(&root) {
                        eprintln!("{}", e);
                    }
                    previous_command = None;
                }
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
                            eprintln!("{}", e);
                        }
                    }
                }
            }
        }

        if let Some(mut final_command) = previous_command {
            let _ = final_command.wait().unwrap();
        }
    }
}
