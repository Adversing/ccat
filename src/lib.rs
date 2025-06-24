use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

pub struct HighlighterConfig {
    pub theme: String,
    pub show_line_numbers: bool,
    pub force_syntax: Option<String>,
}

impl Default for HighlighterConfig {
    fn default() -> Self {
        Self {
            theme: "base16-ocean.dark".to_string(),
            show_line_numbers: false,
            force_syntax: None,
        }
    }
}

pub struct SyntaxHighlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        }
    }
    
    pub fn available_themes(&self) -> Vec<&String> {
        self.theme_set.themes.keys().collect()
    }
    
    pub fn available_syntaxes(&self) -> Vec<&str> {
        self.syntax_set.syntaxes()
            .iter()
            .map(|s| s.name.as_str())
            .collect()
    }
    
    pub fn highlight_file(&self, file_path: &str, config: &HighlighterConfig) -> Result<String> {
        let content = std::fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file '{}'", file_path))?;
        
        self.highlight_content(&content, file_path, config)
    }
    
    pub fn highlight_content(&self, content: &str, file_path: &str, config: &HighlighterConfig) -> Result<String> {
        let theme = self.theme_set.themes.get(&config.theme)
            .with_context(|| format!("Theme '{}' not found", config.theme))?;
        
        let syntax = {
            if let Some(syntax_name) = &config.force_syntax {
                self.syntax_set.find_syntax_by_name(syntax_name)
                    .with_context(|| format!("Syntax '{}' not found", syntax_name))?
            } else {
                self.detect_syntax(content, file_path)
            }
        };

        let mut h = HighlightLines::new(syntax, theme);
        let mut result = String::new();
        
        for (line_num, line) in LinesWithEndings::from(content).enumerate() {
            let ranges: Vec<(Style, &str)> = h.highlight_line(line, &self.syntax_set)
                .with_context(|| "Failed to highlight line")?;
            
            if config.show_line_numbers {
                result.push_str(&format!("{:4} | ", line_num + 1));
            }
            
            let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
            result.push_str(&escaped);
        }
        
        Ok(result)
    }
    
    fn detect_syntax(&self, content: &str, file_path: &str) -> &syntect::parsing::SyntaxReference {
        let extension = Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        
        let custom_mappings = self.get_custom_mappings();
        
        if let Some(syntax_name) = custom_mappings.get(extension) {
            if let Some(syntax) = self.syntax_set.find_syntax_by_name(syntax_name) {
                return syntax;
            }
        }
        
        self.syntax_set.find_syntax_by_extension(extension)
            .or_else(|| self.syntax_set.find_syntax_by_first_line(content))
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text())
    }
    
    fn get_custom_mappings(&self) -> HashMap<&str, &str> {
        let mut mappings = HashMap::new();
        
        // this could be loaded from a config file 
        mappings.insert("c", "C");
        mappings.insert("h", "C");
        mappings.insert("cpp", "C++");
        mappings.insert("cxx", "C++");
        mappings.insert("cc", "C++");
        mappings.insert("hpp", "C++");
        mappings.insert("hxx", "C++");
        mappings.insert("java", "Java");
        mappings.insert("py", "Python");
        mappings.insert("js", "JavaScript");
        mappings.insert("ts", "TypeScript");
        mappings.insert("rs", "Rust");
        mappings.insert("go", "Go");
        mappings.insert("php", "PHP");
        mappings.insert("rb", "Ruby");
        mappings.insert("cs", "C#");
        mappings.insert("html", "HTML");
        mappings.insert("css", "CSS");
        mappings.insert("xml", "XML");
        mappings.insert("json", "JSON");
        mappings.insert("yaml", "YAML");
        mappings.insert("yml", "YAML");
        mappings.insert("md", "Markdown");
        mappings.insert("sh", "Bash");
        mappings.insert("bash", "Bash");
        mappings.insert("zsh", "Bash");
        mappings.insert("fish", "Fish");
        mappings.insert("ps1", "PowerShell");
        mappings.insert("sql", "SQL");
        mappings.insert("r", "R");
        mappings.insert("R", "R");
        mappings.insert("lua", "Lua");
        mappings.insert("vim", "VimL");
        mappings.insert("dockerfile", "Dockerfile");
        mappings.insert("toml", "TOML");
        mappings.insert("ini", "INI");
        mappings.insert("cfg", "INI");
        mappings.insert("conf", "INI");
        
        mappings
    }
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}
