use std::ffi::{CString, OsString};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, ExitStatus};

use clap::Parser;
use clap::builder::styling;
use sys_mount::{FilesystemType, Mount, MountFlags, PropagationType, UnmountFlags};

use crate::identity::group::{Group, Groups};
use crate::identity::user::User;

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
    arg_required_else_help = false,
)]
/// Command line utility for <insert thing name> substitution.
struct Cli {
    /// Program to execute. $SHELL by default or login shell if $SHELL is not set.
    #[arg(trailing_var_arg = true, required = false)]
    program: Option<Vec<String>>,

    /// Name or uid of user. Defaults to root.
    #[arg(long, short, hide_default_value = true, num_args = 0..=1)]
    #[arg(default_value = User::root())]
    user: User,

    /// Don't change the user.
    #[arg(long, short, conflicts_with = "user")]
    same_user: bool,

    /// Names or guids of additional supplementary groups separated by commas.
    #[arg(long, short = 'G', value_delimiter = ',')]
    groups: Vec<Group>,

    /// Inherit supplementary groups of the calling user.
    #[arg(long, short)]
    inherit_groups: bool,

    /// Clear supplementary groups. --groups remains untouched
    #[arg(long, short, conflicts_with = "inherit_groups")]
    clear_groups: bool,

    /// Clear environment variables. Also unsets $SHELL.
    #[arg(long, short)]
    no_keep_env: bool,

    /// Change $HOME to match PAM.
    #[arg(long, short = 'H', conflicts_with = "same_user")]
    home: bool,

    /// Run command as a login shell.
    #[arg(long, short, conflicts_with = "home")]
    login: bool,

    /// Change root before doing the user substitution. Binds /sys, /proc and /dev in the new root.
    /// Implies --login.
    #[arg(long, short, conflicts_with_all = ["same_user", "inherit_groups"])]
    root: Option<PathBuf>,

    /// Print version.
    #[arg(long, action = clap::ArgAction::Version)]
    version: (),

    /// Print this message.
    #[arg(global = true, long, short = 'h', action = clap::ArgAction::HelpLong)]
    help: (),
}

fn prepare_command(cli: &Cli, user: &User) -> anyhow::Result<Command> {
    let mut cmd = match cli.program {
        Some(ref argv) => {
            let mut argv = argv.into_iter();
            let mut cmd = Command::new(argv.next().unwrap());
            cmd.args(argv);
            cmd
        }

        None => {
            if !cli.no_keep_env
                && let Some(shell) = std::env::var_os("SHELL")
            {
                Command::new(shell)
            } else {
                Command::new(user.shell())
            }
        }
    };

    if cli.no_keep_env {
        cmd.env_clear();
    }

    if cli.home {
        cmd.env("HOME", user.home());
    }

    if cli.login {
        let mut prog = OsString::from("-");
        prog.push(cmd.get_program());
        cmd.arg0(prog);
    }

    Ok(cmd)
}

fn bind(root: impl AsRef<Path>, dir: impl AsRef<Path>) -> anyhow::Result<Mount> {
    let mut mount = Mount::builder()
        .fstype(FilesystemType::Manual(""))
        .flags(MountFlags::BIND | MountFlags::REC)
        .mount(
            &dir,
            Path::join(root.as_ref(), dir.as_ref().strip_prefix("/")?),
        )?;
    mount.set_propagation_type(PropagationType::SHARED)?;
    Ok(mount)
}

fn unbind(mount: Mount) -> anyhow::Result<()> {
    for entry in mountinfo2::MountInfo::new()?.mounting_points.iter().rev() {
        if entry
            .path
            .ancestors()
            .any(|path| path == mount.target_path())
        {
            sys_mount::unmount(&entry.path, UnmountFlags::empty())?;
        }
    }
    Ok(())
}

fn main() -> anyhow::Result<ExitCode> {
    let cli = Cli::parse();

    let user = User::current();
    let groups = Groups::current();
    let wheel = Group::from_name(String::from("wheel"));

    if !user.is_root() && !wheel.is_some_and(|g| groups.contains(&g)) {
        eprintln!("\x1b[1;91merror:\x1b[0m you must be in \x1b[95m:wheel\x1b[0m group!");
        return Ok(ExitCode::FAILURE);
    }

    let new_user = if cli.same_user {
        user
    } else {
        cli.user.clone()
    };

    let mut new_groups = if cli.inherit_groups {
        groups
    } else if cli.clear_groups {
        Groups::empty()
    } else {
        new_user.groups()
    };

    new_groups.extend(cli.groups.iter().map(Clone::clone));

    if let Some(root) = cli.root.clone() {
        let dev = bind(&root, "/dev")?;
        let sys = bind(&root, "/sys")?;
        let proc = bind(&root, "/proc")?;

        let status = (|| -> anyhow::Result<ExitStatus> {
            let mut cmd = prepare_command(&cli, &new_user)?;
            unsafe {
                cmd.pre_exec(move || {
                    let root = CString::new(root.to_string_lossy().as_ref())?;
                    if libc::chroot(root.as_ptr()) < 0 {
                        Err(std::io::Error::last_os_error())
                    } else {
                        new_user
                            .home()
                            .canonicalize()
                            .and_then(std::env::set_current_dir)
                            .or_else(|_| std::env::set_current_dir("/"))?;
                        new_groups.try_login()?;
                        new_user.try_login()
                    }
                });
            }

            let status = cmd.spawn()?.wait()?;

            Ok(status)
        })();

        unbind(dev).unwrap();
        unbind(sys).unwrap();
        unbind(proc).unwrap();

        std::process::exit(status?.code().unwrap_or_default())
    } else {
        let mut cmd = prepare_command(&cli, &new_user)?;
        new_groups.try_login()?;
        new_user.try_login()?;
        anyhow::bail!(cmd.exec())
    }
}
