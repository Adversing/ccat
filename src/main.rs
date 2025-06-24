use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
use std::path::Path;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

#[derive(Parser)]
#[command(name = "ccat")]
#[command(about = "A colorized cat command for displaying source code files with syntax highlighting.")]
#[command(version = "0.1.0")]
#[command(author = "Adversing")]
struct Args {
    /// The file to display
    file: String,
    
    /// Theme to use for highlighting
    #[arg(short, long, default_value = "base16-ocean.dark")]
    theme: String,
    
    /// Force a specific syntax (overrides file extension detection)
    #[arg(short, long)]
    syntax: Option<String>,
    
    /// Show line numbers
    #[arg(short, long)]
    line_numbers: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    if !Path::new(&args.file).exists() {
        anyhow::bail!("File '{}' not found", args.file);
    }
    
    let content = fs::read_to_string(&args.file)
        .with_context(|| format!("Failed to read file '{}'", args.file))?;
    
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    
    let theme = ts.themes.get(&args.theme)
        .with_context(|| format!("Theme '{}' not found", args.theme))?;
    
    let syntax = {
        if let Some(syntax_name) = args.syntax {
            ps.find_syntax_by_name(&syntax_name)
                .with_context(|| format!("Syntax '{}' not found", syntax_name))?
        } else {
            let extension = Path::new(&args.file)
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("");

            ps.find_syntax_by_extension(extension)
                .or_else(|| ps.find_syntax_by_first_line(&content))
                .unwrap_or_else(|| ps.find_syntax_plain_text())
        }
    };

    
    let mut h = HighlightLines::new(syntax, theme);
    
    for (line_num, line) in LinesWithEndings::from(&content).enumerate() {
        let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps)
            .with_context(|| "Failed to highlight line")?;
        
        if args.line_numbers {
            print!("{:4} | ", line_num + 1);
        }
        
        let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
        print!("{}", escaped);
    }
    
    Ok(())
}