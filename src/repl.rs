use super::do_the_thing;

use ast;
use env::Environment;
use parser;

use lalrpop_util::ParseError;

use rustyline;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use rustyline::completion::{extract_word, Completer};

use std::collections::BTreeSet;

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
                let res = parser::parse_Expressions(line.as_str());

                match res {
                    Ok(exprs) => {
                        rl.add_history_entry(&line);
                        do_the_thing(exprs, &mut bindings);
                    }
                    Err(ParseError::UnrecognizedToken { token: None, expected: _ }) => {
                        let exprs = multiline(&mut rl, line.clone());
                        do_the_thing(exprs, &mut bindings);
                        // Restore the default completer.
                        rl.set_completer(Some(ParseCompleter::default()));
                    }
                    Err(thing) => {
                        panic!("Parse error: {:?}", thing);
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

fn multiline(mut rl: &mut Editor<ParseCompleter>, mut partial_input: String) -> ast::Exprs {
    partial_input += "\n";
    rl.set_completer(Some(ParseCompleter::from_context(partial_input.clone())));
    match rl.readline("  ...> ") {
        Ok(line) => {
            partial_input += line.as_str();
            match parser::parse_Expressions(partial_input.clone().as_str()) {
                Ok(exprs) => exprs,
                Err(ParseError::UnrecognizedToken { token: None, expected: _ }) => {
                    multiline(&mut rl, partial_input)
                }
                Err(something) => panic!("Error, error! {:?}", something),
            }
        }

        Err(ReadlineError::Interrupted) => multiline(&mut rl, partial_input),
        Err(ReadlineError::Eof) => panic!("I don't know what to do"),
        Err(err) => {
            println!("Error: {:?}", err);
            panic!("I don't know what to do");
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
            context: String::from(""),
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
            Err(ParseError::UnrecognizedToken { token: None, expected: candidates }) => {
                let candidates = candidates.into_iter()
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
