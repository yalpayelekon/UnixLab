//! Script parsing for Reed's supported sed-inspired command subset.

use regex::Regex;

use crate::address::{AddressSpec, parse_address};

/// Flags accepted by the substitute command.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct SubstituteFlags {
    /// Replace every non-overlapping match on the line.
    pub global: bool,
    /// Print the pattern space when a replacement occurs.
    pub print: bool,
}

/// A compiled Reed command with its address.
#[derive(Debug)]
pub struct Command {
    /// Address that selects when the command runs.
    pub address: AddressSpec,
    /// Command body.
    pub kind: CommandKind,
}

/// Supported command bodies.
#[derive(Debug)]
pub enum CommandKind {
    /// Print the pattern space.
    Print,
    /// Delete the pattern space and end the cycle early.
    Delete,
    /// Substitute matches in the pattern space.
    Substitute {
        /// Compiled search pattern.
        pattern: Regex,
        /// Replacement text using Rust regex replacement syntax.
        replacement: String,
        /// Substitute flags.
        flags: SubstituteFlags,
    },
}

/// A compiled script ready for the engine.
#[derive(Debug)]
pub struct Script {
    commands: Vec<Command>,
    needs_last_line: bool,
}

impl Script {
    /// Compiles one or more script fragments joined as sequential commands.
    ///
    /// # Errors
    ///
    /// Returns a descriptive message when parsing or regex compilation fails.
    pub fn compile(fragments: &[String]) -> Result<Self, String> {
        let mut commands = Vec::new();
        for fragment in fragments {
            parse_script_fragment(fragment, &mut commands)?;
        }
        if commands.is_empty() {
            return Err("script is empty".to_owned());
        }
        let needs_last_line = commands
            .iter()
            .any(|command| command.address.needs_last_line());
        Ok(Self {
            commands,
            needs_last_line,
        })
    }

    /// Returns the compiled commands.
    #[must_use]
    pub fn commands(&self) -> &[Command] {
        &self.commands
    }

    /// Reports whether execution needs last-line knowledge (`$`).
    #[must_use]
    pub const fn needs_last_line(&self) -> bool {
        self.needs_last_line
    }
}

fn parse_script_fragment(fragment: &str, commands: &mut Vec<Command>) -> Result<(), String> {
    let mut rest = fragment;
    while !rest.is_empty() {
        rest = skip_separators(rest);
        if rest.is_empty() {
            break;
        }
        let (command, remaining) = parse_command(rest)?;
        commands.push(command);
        rest = skip_separators(remaining);
    }
    Ok(())
}

fn skip_separators(input: &str) -> &str {
    input.trim_start_matches(|ch: char| ch == ';' || ch.is_whitespace())
}

fn parse_command(input: &str) -> Result<(Command, &str), String> {
    let input = input.trim_start();
    let (address, after_address) = parse_address_spec(input)?;
    let after_address = after_address.trim_start();
    let Some(command_char) = after_address.chars().next() else {
        return Err("missing command".to_owned());
    };
    let remainder = &after_address[command_char.len_utf8()..];

    match command_char {
        'p' => Ok((
            Command {
                address,
                kind: CommandKind::Print,
            },
            remainder,
        )),
        'd' => Ok((
            Command {
                address,
                kind: CommandKind::Delete,
            },
            remainder,
        )),
        's' => {
            let (pattern, replacement, flags, rest) = parse_substitute(remainder)?;
            Ok((
                Command {
                    address,
                    kind: CommandKind::Substitute {
                        pattern,
                        replacement,
                        flags,
                    },
                },
                rest,
            ))
        }
        other => Err(format!("unsupported command '{other}'")),
    }
}

fn parse_address_spec(input: &str) -> Result<(AddressSpec, &str), String> {
    let trimmed = input.trim_start();
    let Some(first) = trimmed.chars().next() else {
        return Err("missing command".to_owned());
    };

    if !first.is_ascii_digit() && first != '$' && first != '/' {
        return Ok((AddressSpec::All, trimmed));
    }

    let (start, after_start) = parse_address(trimmed)?;
    let after_start = after_start.trim_start();
    if let Some(rest) = after_start.strip_prefix(',') {
        let (end, after_end) = parse_address(rest.trim_start())?;
        Ok((AddressSpec::Range { start, end }, after_end))
    } else {
        Ok((AddressSpec::Single(start), after_start))
    }
}

fn parse_substitute(input: &str) -> Result<(Regex, String, SubstituteFlags, &str), String> {
    let mut chars = input.chars();
    let delimiter = chars
        .next()
        .ok_or_else(|| "substitute is missing delimiter".to_owned())?;
    if delimiter == '\\' || delimiter.is_ascii_whitespace() {
        return Err("invalid substitute delimiter".to_owned());
    }

    let rest = chars.as_str();
    let (pattern_text, after_pattern) = take_delimited_field(rest, delimiter)?;
    let (replacement, after_replacement) = take_delimited_field(after_pattern, delimiter)?;
    let (flags, after_flags) = parse_substitute_flags(after_replacement)?;
    let pattern = Regex::new(&pattern_text).map_err(|error| error.to_string())?;
    Ok((pattern, replacement, flags, after_flags))
}

fn take_delimited_field(input: &str, delimiter: char) -> Result<(String, &str), String> {
    let mut value = String::new();
    let mut chars = input.char_indices();
    while let Some((index, ch)) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some((_, escaped)) => {
                    if escaped == delimiter {
                        value.push(delimiter);
                    } else {
                        value.push('\\');
                        value.push(escaped);
                    }
                }
                None => return Err("trailing backslash in substitute".to_owned()),
            }
            continue;
        }
        if ch == delimiter {
            return Ok((value, &input[index + delimiter.len_utf8()..]));
        }
        value.push(ch);
    }
    Err(format!("unclosed substitute delimiter '{delimiter}'"))
}

fn parse_substitute_flags(input: &str) -> Result<(SubstituteFlags, &str), String> {
    let mut flags = SubstituteFlags {
        global: false,
        print: false,
    };
    let mut end = 0;
    for (index, ch) in input.char_indices() {
        match ch {
            'g' => {
                flags.global = true;
                end = index + 1;
            }
            'p' => {
                flags.print = true;
                end = index + 1;
            }
            ';' | '\n' | '\r' => break,
            other if other.is_whitespace() => break,
            other if other.is_ascii_digit() => {
                return Err("numeric substitute occurrence counts are not supported".to_owned());
            }
            other => return Err(format!("unknown substitute flag '{other}'")),
        }
    }
    Ok((flags, &input[end..]))
}

#[cfg(test)]
mod tests {
    use super::{CommandKind, Script};

    #[test]
    fn compiles_substitute_print_and_delete() {
        let script = Script::compile(&[String::from("s/a/b/g; 2p; /x/d")]).expect("compile");
        assert_eq!(script.commands().len(), 3);
        assert!(matches!(
            script.commands()[0].kind,
            CommandKind::Substitute { .. }
        ));
        assert!(matches!(script.commands()[1].kind, CommandKind::Print));
        assert!(matches!(script.commands()[2].kind, CommandKind::Delete));
        assert!(!script.needs_last_line());
    }

    #[test]
    fn detects_last_line_address() {
        let script = Script::compile(&[String::from("$p")]).expect("compile");
        assert!(script.needs_last_line());
    }

    #[test]
    fn supports_custom_substitute_delimiter() {
        let script = Script::compile(&[String::from(r"s#a/b#c/d#")]).expect("compile");
        assert!(matches!(
            script.commands()[0].kind,
            CommandKind::Substitute { .. }
        ));
    }

    #[test]
    fn rejects_unknown_command() {
        let error = Script::compile(&[String::from("x")]).expect_err("unknown");
        assert!(error.contains("unsupported command"));
    }
}
