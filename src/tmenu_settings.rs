use std::cell::Cell;
use std::rc::Rc;

use crate::menu_item::MenuItem;
use crate::tmenu::ExitState;
use std::io::BufRead;
use std::process::exit;

#[derive(Debug, Clone)]
pub struct TMenuSettings {
    pub auto_accept: bool,
    pub case_insensitive: bool,
    pub allow_undefined: bool,
    pub filter_by_prefix: bool,
    pub fuzzy: bool,
    pub verbose: bool,
    pub print_help: PrintHelp,
    pub available_options: Vec<MenuItem>,
    pub exit_state: Rc<Cell<ExitState>>,
    pub width: Option<u32>
}

#[derive(Debug, Clone)]
pub enum PrintHelp {
    No,
    Short,
    Long,
}

impl Default for TMenuSettings {
    fn default() -> Self {
        Self {
            auto_accept: false,
            case_insensitive: false,
            allow_undefined: false,
            filter_by_prefix: false,
            fuzzy: false,
            verbose: false,
            print_help: PrintHelp::No,
            available_options: vec![],
            exit_state: Rc::new(Cell::new(ExitState::Continue)),
            width: None
        }
    }
}

impl TMenuSettings {
    pub fn from_args(args: Vec<String>) -> Self {
        let stdin = std::io::stdin();
        Self::from_args_and_input(args, || stdin.lock())
    }

    pub fn from_args_and_input<LINES>(args: Vec<String>, input: impl FnOnce() -> LINES) -> Self
    where
        LINES: BufRead,
    {
        let mut settings = Self::default();
        let read_stdin = parse_args(args, &mut settings);

        if read_stdin {
            read_options_stdin(&mut settings, input);
        }

        if let PrintHelp::No = settings.print_help {
            if settings.available_options.is_empty() {
                eprintln!("No options available");
                exit(2);
            }
        }

        settings
    }

    pub(crate) fn maybe_print_help(&self) -> bool {
        match self.print_help {
            PrintHelp::No => false,
            _ => {
                print_help(self);
                true
            }
        }
    }
}

fn parse_args(args: Vec<String>, state: &mut TMenuSettings) -> bool {
    let args_ref: Vec<&str> = args.iter().map(String::as_str).collect();
    let mut remaining = &args_ref[1..];

    let mut read_stdin = true;

    loop {
        match remaining {
            ["-a" | "--auto-accept", r @ ..] => {
                state.auto_accept = true;
                remaining = r;
            }
            ["-i" | "--case-insensitive", r @ ..] => {
                state.case_insensitive = true;
                remaining = r;
            }
            ["-p" | "--match_prefix", r @ ..] => {
                state.filter_by_prefix = true;
                remaining = r;
            }
            ["-f" | "--fuzzy", r @ ..] => {
                state.fuzzy = true;
                remaining = r;
            }
            ["-u" | "--allow-undefined", r @ ..] => {
                state.allow_undefined = true;
                remaining = r;
            }
            ["-w" | "--width", w, r @ ..] => {
                let Ok(width) = w.parse() else {
                    eprintln!("Cannot parse width");
                    exit(-1);
                };
                state.width = Some(width);
                remaining = r;
            }
            ["--", options @ ..] => {
                read_stdin = false;
                options.iter().enumerate().for_each(|(index, s)| {
                    state.available_options.push(MenuItem::create(s, index))
                });
                break;
            }
            ["--verbose", r @ ..] => {
                state.verbose = true;
                remaining = r;
            }
            ["-h", r @ ..] => {
                state.print_help = PrintHelp::Short;
                read_stdin = false;
                remaining = r;
            }
            ["--help", r @ ..] => {
                state.print_help = PrintHelp::Long;
                read_stdin = false;
                remaining = r;
            }
            [option, ..] => {
                eprintln!("Unknown option: {}", option);
                exit(-1);
            }
            [] => {
                break;
            }
        }
    }

    read_stdin
}

pub(crate) fn print_help(_settings: &TMenuSettings) {
    let author = env!("CARGO_PKG_AUTHORS");
    let version = env!("CARGO_PKG_VERSION");
    let description = r#"
Displays a basic menu controlled by keyboard.
When an item is selected, it is printed on stdout and the program
returns exit code 0, when stopped with the ESC key, nothing is printed
and it exits with exit code 1.
Menu items are read from stdin or passed as arguments.

The main difference to the suckless tool dmenu, tmenu is designed
to also work on wayland (also Mac and Windows). Accepts mouse input
to capture keyboard control, but is not designed to allow mouse input.
"#;
    let msg = if let PrintHelp::Short = _settings.print_help {
        r#"
Usage:
    tmenu [OPTIONS] -- ITEM [ITEM ...]
    command | tmenu [OPTIONS]

Item:
    [(MNEMONIC)] [KEY=]VALUE
    Mnemonic: Optional mnemonic (shortcut) for the item
    Key:      Optional item key, used as output, when selected
    Value:    Displayed in the menu, used as output when key is missing

Options:
    -a, --auto-accept       Auto accept option when single option matches
    -i, --case-insensitive  Match options case insensitive
    -p, --match-prefix      Match options using starts-with matcher
    -u, --allow-undefined   Allow users to type custom options
    --verbose               More verbose lot output on stderr
    -h, --help              print help message. --help for more details
    "#
    } else {
        r#"
Usage:
    tmenu [OPTIONS] -- OPTION [OPTION ...]
    command | tmenu [OPTIONS]

Options:
    -a, --auto-accept
        Auto accept option when single option matches.
        Combined with mnemonics, this allows selecting options with a
        single key press.

    -i, --case-insensitive
        Match options case insensitive.
        Mnemonics are still case sensitive though.

    -p, --match-prefix
        Keyboard input matches only the start of the options

    -u, --allow-undefined
        Allow users to type custom options.
        This excludes the --auto-accept.

    --verbose
        More verbose output on stderr.
        For debugging only.

    -h, --help
        print help message. --help for more details
    "#
    };

    println!("tmenu {}\n{}\n{}{}", version, author, description, msg)
}

fn read_options_stdin<LINES>(state: &mut TMenuSettings, input: impl FnOnce() -> LINES)
where
    LINES: BufRead,
{
    let std_in = input();
    let mut lines = std_in.lines();
    let mut index = 0;
    while let Some(line) = lines.next() {
        match line {
            Ok(line) => state.available_options.push(MenuItem::create(&line, index)),
            Err(e) => {
                eprintln!("Failed reading stdin: {:?}", e);
                exit(-1);
            }
        }
        index += 1;
    }
}
