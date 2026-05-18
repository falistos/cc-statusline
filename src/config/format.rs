//! Parser for format strings.
//!
//! Grammar:
//!   - `$ident`              variable reference
//!   - `[ ... ]`              group
//!   - `[ ... ]($style)`     styled group; `$style` inside is substituted later
//!   - `\$ \[ \] \\`          escapes
//!   - other chars are literals
//!
//! The format AST is the same shape for global format and per-module format.
//! Resolution semantics differ depending on the caller (modules vs vars map).

use thiserror::Error;

#[derive(Debug, Clone)]
pub enum Node {
    Literal(String),
    Variable(String),
    Group {
        children: Vec<Node>,
        style: Option<String>,
    },
}

#[derive(Debug, Clone)]
pub struct Format(pub Vec<Node>);

#[derive(Debug, Error)]
pub enum FormatError {
    #[error("unmatched '[' in format string at position {0}")]
    UnmatchedBracket(usize),
    #[error("unterminated style '(' in format string at position {0}")]
    UnterminatedStyle(usize),
}

impl Format {
    pub fn parse(s: &str) -> Result<Self, FormatError> {
        let chars: Vec<char> = s.chars().collect();
        let mut p = Parser {
            chars: &chars,
            pos: 0,
        };
        let nodes = p.parse_nodes(None, 0)?;
        Ok(Format(nodes))
    }
}

struct Parser<'a> {
    chars: &'a [char],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn parse_nodes(
        &mut self,
        terminator: Option<char>,
        opened_at: usize,
    ) -> Result<Vec<Node>, FormatError> {
        let mut nodes = Vec::new();
        let mut literal = String::new();

        while self.pos < self.chars.len() {
            let c = self.chars[self.pos];

            if Some(c) == terminator {
                self.pos += 1;
                if !literal.is_empty() {
                    nodes.push(Node::Literal(std::mem::take(&mut literal)));
                }
                return Ok(nodes);
            }

            match c {
                '\\' => {
                    self.pos += 1;
                    if self.pos < self.chars.len() {
                        literal.push(self.chars[self.pos]);
                        self.pos += 1;
                    }
                }
                '$' => {
                    let start = self.pos + 1;
                    let mut end = start;
                    while end < self.chars.len() && is_ident_char(self.chars[end]) {
                        end += 1;
                    }
                    if end == start {
                        literal.push('$');
                        self.pos += 1;
                    } else {
                        if !literal.is_empty() {
                            nodes.push(Node::Literal(std::mem::take(&mut literal)));
                        }
                        let name: String = self.chars[start..end].iter().collect();
                        nodes.push(Node::Variable(name));
                        self.pos = end;
                    }
                }
                '[' => {
                    let bracket_pos = self.pos;
                    self.pos += 1;
                    if !literal.is_empty() {
                        nodes.push(Node::Literal(std::mem::take(&mut literal)));
                    }
                    let children = self.parse_nodes(Some(']'), bracket_pos)?;
                    let style = self.try_parse_style()?;
                    nodes.push(Node::Group { children, style });
                }
                _ => {
                    literal.push(c);
                    self.pos += 1;
                }
            }
        }

        if terminator.is_some() {
            return Err(FormatError::UnmatchedBracket(opened_at));
        }
        if !literal.is_empty() {
            nodes.push(Node::Literal(literal));
        }
        Ok(nodes)
    }

    fn try_parse_style(&mut self) -> Result<Option<String>, FormatError> {
        if self.pos >= self.chars.len() || self.chars[self.pos] != '(' {
            return Ok(None);
        }
        let style_start = self.pos;
        self.pos += 1; // consume '('
        let start = self.pos;
        while self.pos < self.chars.len() && self.chars[self.pos] != ')' {
            self.pos += 1;
        }
        if self.pos >= self.chars.len() {
            return Err(FormatError::UnterminatedStyle(style_start));
        }
        let style: String = self.chars[start..self.pos].iter().collect();
        self.pos += 1; // consume ')'
        Ok(Some(style))
    }
}

fn is_ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Walk the tree and report whether any descendant is a Variable node.
pub fn contains_variable(nodes: &[Node]) -> bool {
    nodes.iter().any(|n| match n {
        Node::Variable(_) => true,
        Node::Group { children, .. } => contains_variable(children),
        Node::Literal(_) => false,
    })
}
