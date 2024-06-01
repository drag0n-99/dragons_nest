use clap::{Arg, Command, ArgAction::Set};


fn main() {
    let matches = Command::new(clap::crate_name!())
        .version(clap::crate_version!())
        .about(clap::crate_description!())
        .subcommand(Command::new("list-agents").about("List all agents"))
        .subcommand(Command::new("list-jobs").about("List all jobs"))
        .subcommand(
            Command::new("exec")
                .about("Execute a command")
                .arg(
                    Arg::new("agent-uuid")
                        .short('a')
                        .long("agent-uuid")
                        .help("The agent id to execute the command on")
                        .action(Set)
                        .required(true),
                )
                .arg(
                    Arg::new("command")
                        .help("The command to execute, with its arguments.")
                        .required(true)
                        .action(Set)
                        .index(1),
                ),
        )
        .arg_required_else_help(true)
        .get_matches();


// pub fn subcommand(&self) -> Option<(&str, &ArgMatches)>
// Returns a tuple of two values the first is the Arg id, the second is a reference to the arguments and their values
match matches.subcommand() {
        Some(("list-agents", _)) => println!("run agents() function"),
        Some(("list-jobs", _)) => println!("run jobs() function"),
        Some(("exec", sub_m)) => {
            let agent_id: u128 = sub_m
                .get_one::<String>("agent-uuid")
                .expect("Agent ID is required")
                .parse()
                .expect("Error: Invalid agent ID");
            let command = sub_m.get_one::<String>("command").expect("Command is required");
            println!(
                "run exec() function, agent_id is {} and command is {}",
                agent_id, command
            );
        }
        _ => unreachable!("Exhaustive checking in subcommand match failed"),

    }

}