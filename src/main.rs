use std::os::unix::process::CommandExt;
use std::process::ExitCode;

use clap::Parser;
use clap::builder::{ArgPredicate, styling};

mod identity;

const STYLES: styling::Styles = styling::Styles::styled()
    .header(styling::Style::new().bold())
    .usage(styling::Style::new().bold())
    .literal(styling::AnsiColor::BrightMagenta.on_default())
    .placeholder(styling::AnsiColor::BrightBlack.on_default())
    .context(styling::AnsiColor::BrightBlack.on_default())
    .context_value(styling::AnsiColor::BrightMagenta.on_default())
    .valid(styling::AnsiColor::BrightMagenta.on_default())
    .invalid(styling::AnsiColor::BrightRed.on_default());

#[derive(Debug, clap::Parser)]
#[command(
    version,
    styles = STYLES,
    disable_version_flag = true,
    disable_help_flag = true,
    disable_help_subcommand = true,
    arg_required_else_help = true,
)]
/// Simple utility for user and group substitution.
struct Cli {
    /// Program to execute.
    #[arg(hide = true, trailing_var_arg = true, required = true)]
    program: Vec<String>,

    /// Name or uid of user. Implicitly root user unless `-G` flag is provided.
    #[arg(long, short, hide_default_value = true)]
    #[arg(default_value = "0")]
    #[arg(default_value_if("groups", ArgPredicate::IsPresent, None))]
    user: Option<identity::User>,

    /// Names or guids of supplementary groups separated by commas.
    #[arg(long, short = 'G', value_delimiter = ',')]
    groups: Option<Vec<identity::Group>>,

    /// Inherit supplementary groups of calling user.
    #[arg(long, short)]
    inherit_groups: bool,

    /// Clear environment variables.
    #[arg(long)]
    no_keep_env: bool,

    /// Print version.
    #[arg(long, short, action = clap::ArgAction::Version)]
    version: (),

    /// Print this message.
    #[arg(global = true, long, short, action = clap::ArgAction::HelpLong)]
    help: (),
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let mut argv = cli.program.into_iter();

    let groups = identity::get_groups().expect("failed to get user groups");

    let wheel =
        identity::Group::from_name(String::from("wheel")).expect("failed to get wheel group");
    if !groups.contains(&wheel) {
        eprintln!("\x1b[1;91merror:\x1b[0m you must be in \x1b[95m:wheel\x1b[0m group!");
        return ExitCode::FAILURE;
    }

    identity::set_groups(cli.groups.unwrap_or_default(), cli.inherit_groups)
        .expect("failed to set supplementary groups");

    identity::set_user(cli.user.unwrap_or_else(identity::get_user)).expect("failed to set user");

    let mut cmd = std::process::Command::new(argv.next().unwrap());
    cmd.args(argv);
    if cli.no_keep_env {
        cmd.env_clear();
    }

    eprintln!("{}", cmd.exec());
    return ExitCode::FAILURE;
}
