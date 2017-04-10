use super::{do_the_thing, parse_expressions};

use ast;
use env::Environment;
use error::{handle_error, REPLError, ParseError};
use parser;

use lalrpop_util::ParseError as PopParseError;

use rustyline;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use rustyline::completion::{extract_word, Completer};

use std::collections::BTreeSet;
use std::process;

pub fn start() {
    let mut rl = Editor::<ParseCompleter>::new();

    rl.set_completer(Some(ParseCompleter::default()));

    if let Err(_) = rl.load_history("history.txt") {
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
                            handle_error("<command-line>", Box::new(err));
                        }
                    }

                    Err(ParseError::UnrecognizedToken { token: None, expected: _ }) => {
                        let mut partial_input = line.clone();
                        match multiline_loop(&mut rl, &mut partial_input) {
                            Ok(exprs) => {
                                if let Err(err) = do_the_thing(exprs, &mut bindings) {
                                    handle_error("<command-line>", Box::new(err));
                                }
                                // Restore the default completer.
                                rl.set_completer(Some(ParseCompleter::default()));
                            }
                            Err(REPLError::Readline(ReadlineError::Eof)) => {}
                            Err(err) => handle_error("<command-line>", Box::new(err)),
                        }
                    }

                    Err(thing) => {
                        handle_error("<command-line>", Box::new(thing));
                        process::exit(1);
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

fn multiline_loop<'a>(mut rl: &mut Editor<ParseCompleter>,
                      mut partial_input: &'a mut String)
                      -> Result<ast::Exprs, REPLError<'a>> {
    loop {
        *partial_input += "\n";
        rl.set_completer(Some(ParseCompleter::from_context(partial_input.clone())));

        match rl.readline("  ...> ") {
            Ok(line) => {
                *partial_input += line.as_str();

                match parse_expressions(partial_input) {
                    Ok(expr) => return Ok(expr),
                    Err(ParseError::UnrecognizedToken { token: None, expected: _ }) => (),
                    Err(err) => {
                        handle_error("<command-line>", Box::new(err));
                        process::exit(1);
                        // See: https://github.com/rust-lang/rust/issues/40307
                        //return Err(REPLError::Parse(err));
                    }
                }

            }

            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) => return Err(REPLError::Readline(ReadlineError::Eof)),
            Err(err) => return Err(REPLError::Readline(err)),
        }
    }
}

//================
//== Completion ==
//================

const BREAKS: [char; 12] = [' ', '\t', '\n', '(', ')', ',', '+', '-', '*', '/', '<', '>'];

struct ParseCompleter {
    context: String,
    breaks: BTreeSet<char>,
}

impl ParseCompleter {
    pub fn new() -> Self {
        ParseCompleter {
            context: "".to_string(),
            breaks: BREAKS.iter().cloned().collect(),
        }
    }

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

        fn begins_with(input: &String, prefix: String) -> bool {
            let input_len = input.len();
            let prefix_len = prefix.len();

            if input_len < prefix_len {
                false
            } else {
                input[0..prefix.len()] == prefix
            }
        }

        let (start, word) = extract_word(line, pos, &self.breaks);

        let partial_input = self.context.clone() + &line[0..start];

        match parser::parse_Expressions(partial_input.as_str()) {
            Ok(_) => Ok((0, vec![])),
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
                            if begins_with(&candidate, String::from(word)) {
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
