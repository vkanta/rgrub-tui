use crate::model::entry::{GrubCommand, GrubEntry, GrubMenu};
use crate::model::settings::GrubSettings;
use anyhow::{Context, Result};
use regex::Regex;
use std::fs;
use std::path::Path;

pub struct GrubParser;

impl GrubParser {
    pub fn parse_settings<P: AsRef<Path>>(path: P) -> Result<GrubSettings> {
        let content = fs::read_to_string(path).context("Failed to read settings file")?;
        let mut settings = GrubSettings::new();
        let re = Regex::new(r#"(?m)^([A-Z0-9_]+)\s*=\s*["']?(.*?)["']?\s*(?:#.*)?$"#)?;

        for cap in re.captures_iter(&content) {
            settings.set(cap[1].to_string(), cap[2].to_string());
        }

        Ok(settings)
    }

    pub fn parse_config<P: AsRef<Path>>(path: P) -> Result<Vec<GrubMenu>> {
        let content = fs::read_to_string(path).context("Failed to read config file")?;
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        Self::parse_recursive(&lines)
    }

    fn parse_recursive(lines: &[String]) -> Result<Vec<GrubMenu>> {
        let mut menus = Vec::new();
        let mut i = 0;
        let re_name = Regex::new(r#"(?:menuentry|submenu)\s+['"]([^'"]+)['"]"#)?;

        while i < lines.len() {
            let line = lines[i].trim();
            if line.is_empty() || line.starts_with('#') {
                i += 1;
                continue;
            }

            if line.contains("menuentry") || line.contains("submenu") {
                let header = line.clone();
                
                // Find the corresponding closing brace for this block
                let mut brace_count = 0;
                let mut end_idx = None;
                for idx in i..lines.len() {
                    if lines[idx].contains('{') {
                        brace_count += 1;
                    } else if lines[idx].contains('}') {
                        brace_count -= 1;
                        if brace_count == 0 {
                            end_idx = Some(idx);
                            break;
                        }
                    }
                }

                if let Some(final_idx) = end_idx {
                    let name = re_name.captures(&header)
                        .map(|cap| cap[1].to_string())
                        .unwrap_or_else(|| "Unknown".to_string());

                    if header.contains("submenu") {
                        // Recursively parse the lines inside this block
                        let submenus = Self::parse_recursive(&lines[(i + 1)..final_idx])?;
                        menus.push(GrubMenu::Submenu {
                            name,
                            id: None,
                            entries: submenus,
                        });
                    } else {
                        // Parse commands inside the entry block
                        let mut commands = Vec::new();
                        for cmd_idx in (i + 1)..final_idx {
                            let cmd_line = lines[cmd_idx].trim();
                            if cmd_line.is_empty() || cmd_line.starts_with('#') || cmd_line.contains('{') || cmd_line.contains('}') {
                                continue;
                            }

                            let parts: Vec<&str> = cmd_line.split_whitespace().collect();
                            if parts.len() >= 2 {
                                commands.push(GrubCommand {
                                    command: parts[0].to_string(),
                                    args: parts[1..].iter().map(|&s| s.to_string()).collect(),
                                });
                            }
                        }
                        menus.push(GrubMenu::Entry(GrubEntry {
                            name,
                            classes: Vec::new(),
                            id: None,
                            commands,
                        }));
                    }
                    i = final_idx + 1;
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }

        Ok(menus)
    }
}
