use ariadne::{Color, Label, Report, ReportKind, Source};
use rollkit::{EvalError, Value, eval, parse};
use rustyline::{DefaultEditor, error::ReadlineError};
use yansi::Paint;

mod explain;

use explain::explain_expr;

// Color schema
// Red - Errors
// Green - commands, sequence numbers
// Blue - Auxiliary info
// Cyan - headings
// Magenta - expressions
// Yellow - results

/// Report parse errors using ariadne
fn report_parse_errors(seq: usize, input: &str, errors: Vec<chumsky::error::Rich<'_, char>>) {
    for error in errors {
        let span = error.span();
        let msg = error.to_string();

        Report::build(ReportKind::Error, ("<stdin>", span.into_range()))
            .with_message("Parse Error")
            .with_label(
                Label::new(("<stdin>", span.start..span.end))
                    .with_message(msg)
                    .with_color(Color::Red),
            )
            .finish()
            .print((
                "<stdin>",
                Source::from(input).with_display_line_offset(seq.saturating_sub(1)),
            ))
            .unwrap();
    }
}

/// Report evaluation errors using ariadne
fn report_eval_error(seq: usize, input: &str, error: EvalError) {
    let msg = error.to_string();

    Report::build(ReportKind::Error, ("<stdin>", 0..input.len()))
        .with_message("Evaluation Error")
        .with_label(
            Label::new(("<stdin>", 0..input.len()))
                .with_message(msg)
                .with_color(Color::Red),
        )
        .finish()
        .print((
            "<stdin>",
            Source::from(input).with_display_line_offset(seq.saturating_sub(1)),
        ))
        .unwrap();
}

/// Print the result value
fn format_expr_result(value: &Value) -> String {
    match value {
        Value::Integer(n) => format!("{}", n).yellow().to_string(),
        Value::List(lst) => {
            let sum: i64 = lst.iter().sum();
            format!(
                "{} (from list with {} elements: {{{}}})",
                sum.yellow(),
                lst.len().to_string().blue(),
                lst.iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
                    .blue()
            )
        }
    }
}

fn print_welcome() {
    println!(
        "{}",
        "RollKit REPL - Interactive Dice Expression Evaluator".cyan()
    );
    println!(
        "{}",
        "====================================================".cyan()
    );
    println!("Type {} for help\n", ":help".green());
}

/// Print help message
fn print_help() {
    println!(
        "\n{}\n  Enter {} to evaluate them or {} for other functions",
        "Usage:".yellow(),
        "dice expressions".magenta(),
        "commands".green()
    );
    println!("\n{}:", "Commands".yellow());
    println!(
        "  {} or {}                   - Show this help message",
        ":h[elp]".green(),
        ":?".green()
    );
    println!(
        "  {} {} or {} {}  - Explain the structure of an expression",
        ":ex[plain]".green(),
        "<expr>".magenta(),
        ":!".green(),
        "<expr>".magenta()
    );
    println!(
        "  {} or {}                - Exit the REPL",
        ":exit".green(),
        ":q[uit]".green()
    );
    println!("\n{}:", "Examples".yellow());
    println!(
        "  {}                  - Roll 3 six-sided dice",
        "3d6".magenta()
    );
    println!(
        "  {}               - Roll 4d6, keep highest 3",
        "4d6kh3".magenta()
    );
    println!(
        "  {}              - Roll 2d6 and add 5",
        "2d6 + 5".magenta()
    );
    println!(
        "  {}        - Roll dice from a list",
        "2d{1,2,3,5,8}".magenta()
    );
    println!(
        "  {}           - Use a range [start, end, step]",
        "[1, 10, 2]".magenta()
    );
    println!(
        "  {} {}  - Explain an expression structure",
        ":explain".green(),
        "4d6kh3 + 2".magenta()
    );
    println!();
}

fn print_err(err: &str) {
    println!("{}: {}", "Error".red(), err);
}

fn eval_expr(seq: usize, expr: &str, with_explain: bool) {
    match parse(expr) {
        Ok(parsed_expr) => {
            match eval(&parsed_expr) {
                Ok(value) => {
                    println!(
                        "[{}] {}",
                        seq.to_string().green(),
                        format_expr_result(&value)
                    );
                }
                Err(e) => {
                    report_eval_error(seq, expr, e);
                }
            }

            if with_explain {
                println!("Explanation:");
                println!("  Parsed: {}", parsed_expr.format_inline().magenta());
                println!("  Expression Structure:");
                explain_expr(&parsed_expr);
            }
        }
        Err(errors) => {
            report_parse_errors(seq, expr, errors);
        }
    }
}

/// Process a command, return true to exit REPL.
fn process_command(seq: usize, command: &str, args: &str) -> bool {
    match command {
        "help" | "h" | "?" => {
            print_help();
        }
        "exit" | "quit" | "q" => {
            println!("{}", "Goodbye!".yellow());
            return true;
        }
        "explain" | "ex" | "!" => {
            if args.is_empty() {
                print_err("No expression provided to explain");
            } else {
                eval_expr(seq, args, true);
            }
        }
        _ => {
            let cmd_with_colon = format!(":{}", command);
            print_err(&format!("Unknown command: {}", cmd_with_colon.red()));
            println!("Type {} for help", ":help".green());
        }
    }

    false
}

/// Process a single input line, return true to exit REPL.
fn process_line(seq: usize, line: &str) -> bool {
    // Handle commands
    if line.starts_with(':') {
        let first_space = line.find(char::is_whitespace).unwrap_or(line.len());
        let command = &line[1..first_space];
        let args = line[first_space..].trim();

        return process_command(seq, command, args);
    }

    eval_expr(seq, line, false);
    false
}

fn prompt(seq: usize) -> String {
    format!("{}:[{}]> ", "rollkit".cyan(), seq.to_string().green())
}

fn main() {
    print_welcome();

    // Create a rustyline editor with history support
    let mut rl = DefaultEditor::new().expect("Failed to create readline editor");
    // Sequence number for prompts
    let mut seq = 1usize;

    loop {
        // Read input with rustyline, to support history and inline editing
        let line = match rl.readline(&prompt(seq)) {
            Ok(line) => line,
            Err(ReadlineError::Interrupted) => {
                // Handle Ctrl+C
                println!("{}", "^C".red());
                continue;
            }
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                print_err(&format!("Error reading input: {:?}", err));
                break;
            }
        };

        // Trim whitespace from the input line
        let line = line.trim();

        // Handle empty input
        if line.is_empty() {
            continue;
        } else {
            let _ = rl.add_history_entry(line);
        }

        // Process the input line
        if process_line(seq, line) {
            break;
        }

        seq += 1;
    }

    println!("{} {} lines processed.", "Goodbye!".yellow(), seq);
}
