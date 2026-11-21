//! Reliquary: the registry for agents, skills and more: build once, equip every agent. For now this is the `reliquary` binary saying what it is; Reliquary itself
//! comes with CP-T9.1 (docs/design/README.md, STATUS.md).

/// What `reliquary` prints for its arguments, where, and the exit status.
#[derive(Debug, PartialEq, Eq)]
pub struct Reply {
    pub text: String,
    pub to_stderr: bool,
    pub status: u8,
}

pub const USAGE: &str = "usage: reliquary [--version | --help | bench <command> ...]\n";

const HELP: &str = "reliquary: Reliquary, the registry for agents, skills and more: build once, equip every agent.\n";

/// The status for arguments the binary refuses: the status a shell's builtins give for incorrect
/// usage (docs/design/constants.md).
const MISUSE: u8 = 2;

/// The commit this build was made from: `RELIQUARY_COMMIT` at build time, which kiln CI and
/// scripts/bench-build.sh set, else "unknown commit".
#[must_use]
pub fn commit() -> &'static str {
    option_env!("RELIQUARY_COMMIT").unwrap_or("unknown commit")
}

/// The name, the version and the commit it was built from.
#[must_use]
pub fn version_line() -> String {
    format!("reliquary {} ({})\n", env!("CARGO_PKG_VERSION"), commit())
}

/// `reliquary bench` (plan RQ0.7): `args` are the words after `bench`, run with these streams; returns
/// the exit status: 0 done, 1 a failure the command found (a gate, an alert, a refusal), 2 misuse.
pub fn bench(
    args: &[Option<String>],
    input: &mut impl std::io::BufRead,
    out: &mut impl std::io::Write,
    err: &mut impl std::io::Write,
) -> u8 {
    let Some(words) = args.iter().cloned().collect::<Option<Vec<String>>>() else {
        let _ = write!(err, "reliquary bench: an argument isn't Unicode\n{USAGE}");
        return MISUSE;
    };
    match reliquary_bench::cli::main(&words, commit(), input, out, err) {
        reliquary_bench::cli::Ending::Done => 0,
        reliquary_bench::cli::Ending::Failed => 1,
        reliquary_bench::cli::Ending::Misuse => MISUSE,
    }
}

/// The reply to the binary's arguments; `None` stands for an argument that isn't Unicode.
#[must_use]
pub fn reply(args: &[Option<String>]) -> Reply {
    let refuse = |why: String| Reply {
        text: format!("reliquary: {why}\n{USAGE}"),
        to_stderr: true,
        status: MISUSE,
    };
    match args {
        [Some(a)] if a == "--version" || a == "-V" => Reply {
            text: version_line(),
            to_stderr: false,
            status: 0,
        },
        [Some(a)] if a == "--help" || a == "-h" => Reply {
            text: format!("{HELP}{USAGE}"),
            to_stderr: false,
            status: 0,
        },
        [] => Reply {
            text: USAGE.to_owned(),
            to_stderr: true,
            status: MISUSE,
        },
        _ if args.iter().any(Option::is_none) => refuse("an argument isn't Unicode".to_owned()),
        _ => refuse(format!(
            "unknown arguments: {}",
            args.iter().flatten().cloned().collect::<Vec<_>>().join(" ")
        )),
    }
}
