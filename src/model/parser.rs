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
        // Pattern for KEY="VALUE" or KEY=VALUE (handling quotes and comments)
        let re = Regex::new(r#"(?m)^([A-Z0-9_]+)\s*=\s*["']?(.*?)["']?\s*(?:#.*)?$"#)?;

        for cap in re.captures_iter(&content) {
            let key = cap[1].to_string();
            let value = cap[2].to_string();
            settings.set(key, value);
        }

        Ok(settings)
    }

    pub fn parse_config<P: AsRef<Path>>(path: P) -> Result<Vec<GrubMenu>> {
        let content = fs::read_to_string(path).context("Failed to read config file")?;
        Self::parse_recursive(&content)
    }

    fn parse_recursive(content: &str) -> Result<Vec<GrubMenu>> {
        let mut menus = Vec::new();
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();
            if line.is_empty() || line.starts_with('#') || line.contains("### BEGIN") || line.contains("### END") {
                i += 1;
                continue;
            }

            if line.starts_with("menuentry") || line.starts_with("submenu") {
                match Self::parse_block(&lines, i) {
                    Ok((menu, next_idx)) => {
                        menus.push(menu);
                        i = next_idx;
                    }
                    Err(_) => {
                        i += 1;
                    }
                }
            } else {
                i += 1;
            }
        }

        Ok(menus)
    }

    fn parse_block(lines: &[String], start_idx: usize) -> Result<(GrubMenu, usize)> {
        let header = &lines[start_idx];
        let mut depth = 0;
        let mut found_first_brace = false;
        let mut end_idx = None;

        for (idx, line) in lines.iter().enumerate().skip(start_idx) {
            let trimmed = line.trim();
            let open_braces = trimmed.matches('{').count();
            let close_braces = trimmed.matches('}').count();

            if !found_first_brace && open_braces > 0 {
                found_first_brace = true;
                depth += open_braces;
            } else if found_first_brace {
                depth += open_braces;
                depth -= close_braces;
            }

            if found_first_brace && depth == 0 {
                end_idx = Some(idx);
                break;
            }
        }

        let end_idx = end_idx.ok_or_else(|| anyhow::anyhow!("No matching brace for block at line {}", start_idx + 1))?;

        // Robust regex to capture name inside single quotes: menuentry 'Name' ...
        let re_name = Regex::new(r#"(?:menuentry|submenu)\s+'([^']+)'"#)?;
        let name = re_name.captures(header)
            .map(|cap| cap[1].to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        let mut commands = Vec::new();
        for idx in (start_idx + 1)..end_idx {
            let line = lines[idx].trim();
            if line.is_empty() || line.starts_with('#') || line.contains('{') || line.contains('}') {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                commands.push(GrubCommand {
                    command: parts[0].to_string(),
                    args: parts[1..].iter().map(|&s| s.to_string()).collect(),
                });
            }
        }

        if header.starts_with("submenu") {
            let inner_lines: Vec<String> = lines[(start_idx + 1)..end_idx].iter().cloned().collect();
            let sub_entries = Self::parse_recursive_on_slice(&inner_lines)?;
            Ok((GrubMenu::Submenu {
                name,
                id: None,
                entries: sub_entries,
            }, end_idx + 1))
        } else {
            Ok((GrubMenu::Entry(GrubEntry {
                name,
                classes: vec![],
                id: None,
                commands,
            }), end_idx + 1))
        }
    }

    fn parse_recursive_on_slice(lines: &[String]) -> Result<Vec<GrubMenu>> {
        let mut menus = Vec::new();
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim();
            if line.is_empty() || line.starts_with('#') {
                i += 1;
                continue;
            }
            if line.starts_with("menuentry") || line.starts_with("submenu") {
                match Self::parse_block(lines, i) {
                    Ok((menu, next_idx)) => {
                        menus.push(menu);
                        i = next_idx;
                    }
                    Err(_) => i += 1,
                }
            } else {
                i += 1;
            }
        }
        Ok(menus)
    }
}
