//! The REPL

use super::{do_the_thing, parse_expressions};

use ast;
use env::Environment;
use error::{print_error, REPLError, ParseError};
use parser;

use lalrpop_util::ParseError as PopParseError;

use rustyline;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use rustyline::completion::{extract_word, Completer};

use std::collections::BTreeSet;

/// Start the REPL
///
/// This will load and save the history in a file named `history.txt` in the current directory.
///
/// Note: Multiline support!
///
/// Note: Smart completion support! (not perfect)
pub fn start() {
    let mut rl = Editor::<ParseCompleter>::new();

    rl.set_completer(Some(ParseCompleter::default()));

    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    let mut bindings = Environment::new();

    loop {
        let readline = rl.readline("input> ");

        match readline {
            Ok(line) => {
                match parse_expressions(line.as_str()) {
                    Ok(exprs) => {
                        rl.add_history_entry(&line);
                        if let Err(err) = do_the_thing(exprs, &mut bindings) {
                            print_error("<command-line>", line.as_str(), &err);
                        }
                    }

                    Err(ParseError::UnrecognizedToken { token: None, .. }) => {
                        let mut partial_input = line.clone();
                        match multiline_loop(&mut rl, &mut partial_input) {
                            (input, Ok(exprs)) => {
                                if let Err(err) = do_the_thing(exprs, &mut bindings) {
                                    print_error("<command-line>", input.as_str(), &err);
                                }
                                // Restore the default completer.
                                rl.set_completer(Some(ParseCompleter::default()));
                            }
                            (_, Err(REPLError::Readline(ReadlineError::Eof))) => {}
                            (input, Err(err)) => print_error("<command-line>", input.as_str(), &err),
                        }
                    }

                    Err(err) => {
                        print_error("<command-line>", line.as_str(), &err);
                    }
                }

            }
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
        rl.save_history("history.txt").unwrap();
    }
}

/// The loop that keep asking for input until the expression is completed
///
/// This is done by looping while the parser returns an error of type "I needed something more"
fn multiline_loop<'a>(mut rl: &mut Editor<ParseCompleter>,
                      mut partial_input: &'a mut String)
                      -> (&'a String, Result<ast::Exprs, REPLError<'a>>) {
    loop {
        *partial_input += "\n";
        rl.set_completer(Some(ParseCompleter::from_context(partial_input.clone())));

        match rl.readline("  ...> ") {
            Ok(line) => {
                let previous_attempt_len = partial_input.len();
                *partial_input += line.as_str();

                match parse_expressions(partial_input) {
                    Ok(expr) => return (partial_input, Ok(expr)),
                    Err(ParseError::UnrecognizedToken { token: None, .. }) => continue,
                    Err(err) => {
                        // See: https://github.com/rust-lang/rust/issues/40307
                        //return Err(REPLError::Parse(err));

                        print_error("<command-line>", partial_input, &err);
                    }
                }

                // Restore previous attempt to prevent further errors.
                // Cannot do it inside the match above because partial_input
                // would be still borrowed.
                partial_input.truncate(previous_attempt_len);
            }

            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) => return (partial_input, Err(REPLError::Readline(ReadlineError::Eof))),
            Err(err) => return (partial_input, Err(REPLError::Readline(err))),
        }
    }
}

//================
//== Completion ==
//================

/// The break chars for the completion
const BREAKS: [char; 12] = [' ', '\t', '\n', '(', ')', ',', '+', '-', '*', '/', '<', '>'];

/// The structure that provides completion that will be given to the rustyline library
struct ParseCompleter {
    context: String,
    breaks: BTreeSet<char>,
}

impl ParseCompleter {
    /// Create a new completer
    pub fn new() -> Self {
        ParseCompleter {
            context: "".to_string(),
            breaks: BREAKS.iter().cloned().collect(),
        }
    }

    /// Create a new completer from the given context (partial user input from the previous
    /// multiline inputs)
    pub fn from_context(context: String) -> Self {
        ParseCompleter {
            context: context,
            breaks: BREAKS.iter().cloned().collect(),
        }
    }
}

impl Default for ParseCompleter {
    fn default() -> Self {
        ParseCompleter::new()
    }
}

impl Completer for ParseCompleter {
    fn complete(&self, line: &str, pos: usize) -> rustyline::Result<(usize, Vec<String>)> {

        fn begins_with(input: &str, prefix: &str) -> bool {
            let input_len = input.len();
            let prefix_len = prefix.len();

            if input_len < prefix_len {
                false
            } else {
                &input[0..prefix.len()] == prefix
            }
        }

        let (start, word) = extract_word(line, pos, &self.breaks);

        let partial_input = self.context.clone() + &line[0..start];

        match parser::parse_Expressions(partial_input.as_str()) {
            Err(PopParseError::UnrecognizedToken {
                    token: None,
                    expected: candidates,
                }) => {
                let candidates = candidates
                    .into_iter()
                    .filter_map(|mut candidate| {
                        if candidate.chars().nth(0) == Some('\"') {
                            // Remove quotes
                            candidate.remove(0);
                            candidate.pop();
                            if begins_with(&candidate, word) {
                                Some(candidate)
                            } else {
                                None
                            }
                        } else {
                            // Cannot extrapolate candidates from Regex
                            None
                        }
                    })
                    .collect();
                Ok((start, candidates))
            }
            _ => Ok((0, vec![])),
        }
    }
}
