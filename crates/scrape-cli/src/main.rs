//! scrape - High-performance HTML extraction CLI.

mod args;
mod batch;
mod extract;
mod fetch;
mod output;
mod repl;

use std::{
    io::{self, Read, Write},
    process::ExitCode,
};

use args::{Args, ColorMode, OutputFormat};
use is_terminal::IsTerminal;
use output::{CsvOutput, HtmlOutput, JsonOutput, Output, TextOutput};

fn main() -> ExitCode {
    let args = match Args::parse_and_validate() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("Error: {e}");
            return ExitCode::from(4);
        }
    };

    match run(&args) {
        Ok(found) => {
            if found {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(e) => {
            if !args.quiet {
                eprintln!("Error: {e}");
            }
            ExitCode::from(2)
        }
    }
}

#[allow(clippy::too_many_lines)]
fn run(args: &Args) -> anyhow::Result<bool> {
    // Handle interactive mode
    if args.interactive {
        let mut repl = repl::Repl::new();
        repl.run()?;
        return Ok(true);
    }

    // Handle explain mode
    if args.explain {
        if let Some(ref selector) = args.selector {
            use scrape_core::query::explain;
            match explain(selector) {
                Ok(explanation) => {
                    println!("{}", explanation.format());
                    return Ok(true);
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("Invalid selector: {e}"));
                }
            }
        }
        return Err(anyhow::anyhow!("--explain requires a selector"));
    }

    let use_color = match args.color {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => std::io::stdout().is_terminal(),
    };

    let delimiter = if args.null { b'\0' } else { b'\n' };

    let output: Box<dyn Output> = match args.output {
        OutputFormat::Text => Box::new(TextOutput { delimiter, color: use_color }),
        OutputFormat::Json => Box::new(JsonOutput { pretty: args.pretty }),
        OutputFormat::Html => Box::new(HtmlOutput { delimiter }),
        OutputFormat::Csv => Box::new(CsvOutput),
    };

    let stdout = io::stdout();
    let mut writer = stdout.lock();

    let mut found_any = false;

    if args.files.is_empty() {
        // Read from stdin
        let mut html = String::new();
        io::stdin().read_to_string(&mut html)?;

        if let Some(ref selector) = args.selector {
            let results = extract::extract(
                &html,
                selector,
                args.attribute.as_deref(),
                args.first,
                args.output == OutputFormat::Json,
            )?;
            found_any = !results.is_empty();
            output.format_single(&mut writer, &results, None)?;
        } else {
            let selectors = args.parse_selects();
            let results =
                extract::extract_named(&html, &selectors, args.attribute.as_deref(), args.first)?;
            found_any = results.values().any(|v| !v.is_empty());
            output.format_named(&mut writer, &results, None)?;
        }
    } else if let Some(ref selector) = args.selector {
        // Single selector, multiple files
        let results = batch::process_files(
            &args.files,
            selector,
            args.attribute.as_deref(),
            args.first,
            args.parallel,
        );

        for file_result in results {
            match file_result.result {
                Ok(extractions) if !extractions.is_empty() => {
                    found_any = true;
                    let filename = if args.show_filename() {
                        Some(file_result.filename.as_str())
                    } else {
                        None
                    };
                    output.format_single(&mut writer, &extractions, filename)?;
                }
                Err(e) if !args.quiet => {
                    eprintln!("{}: {e}", file_result.filename);
                }
                Ok(_) | Err(_) => {}
            }
        }
    } else {
        // Named selectors, multiple files
        let selectors = args.parse_selects();
        let results = batch::process_files_named(
            &args.files,
            &selectors,
            args.attribute.as_deref(),
            args.first,
            args.parallel,
        );

        for file_result in results {
            match file_result.result {
                Ok(extractions) => {
                    if extractions.values().any(|v| !v.is_empty()) {
                        found_any = true;
                    }
                    let filename = if args.show_filename() {
                        Some(file_result.filename.as_str())
                    } else {
                        None
                    };
                    output.format_named(&mut writer, &extractions, filename)?;
                }
                Err(e) if !args.quiet => {
                    eprintln!("{}: {e}", file_result.filename);
                }
                Err(_) => {}
            }
        }
    }

    writer.flush()?;
    Ok(found_any)
}
