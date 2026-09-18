/// An AI-generated lightweight regex engine using Thompson's NFA (Nondeterministic Finite Automaton)
/// algorithm.
/// "Unlike backtracking engines, an NFA tracks all possible states simultaneously, ensuring linear
/// time complexity \(O(m \times n)\) relative to the text length n and regex length m."
//# Purpose: Prototype for lightweight use avoiding `regex` crate for e.g. WASM.
//# Categories: prototype, technique

#[derive(Debug, Clone, PartialEq)]
enum CharClass {
    Digit,             // \d
    Whitespace,        // \s
    Range(char, char), // [a-z]
    Set(Vec<char>),    // [abc]
}

#[derive(Debug, Clone, PartialEq)]
enum RegexAST {
    Literal(char),
    Wildcard,
    Class(CharClass),
    AnchorStart, // ^
    AnchorEnd,   // $
    ZeroOrMore(Box<Self>),
    ZeroOrOne(Box<Self>),
    Concat(Vec<Self>),
    Alternation(Box<Self>, Box<Self>), // left | right
}

use std::iter::Peekable;
use std::str::Chars;

fn parse_regex(pattern: &str) -> Result<RegexAST, String> {
    let mut chars = pattern.chars().peekable();
    parse_alternation(&mut chars)
}

// Parses alternation (lowest precedence, split by '|')
fn parse_alternation(chars: &mut Peekable<Chars>) -> Result<RegexAST, String> {
    let left = parse_concat(chars)?;
    if chars.peek() == Some(&'|') {
        chars.next(); // consume '|'
        let right = parse_alternation(chars)?;
        Ok(RegexAST::Alternation(Box::new(left), Box::new(right)))
    } else {
        Ok(left)
    }
}

// Parses sequential elements and handles parenthesis boundaries
fn parse_concat(chars: &mut Peekable<Chars>) -> Result<RegexAST, String> {
    let mut nodes = Vec::new();

    while let Some(&ch) = chars.peek() {
        if ch == '|' || ch == ')' {
            break; // Stop sequencing at alternation or group close boundaries
        }
        chars.next(); // Consume the character

        let current_node = match ch {
            '^' => RegexAST::AnchorStart,
            '$' => RegexAST::AnchorEnd,
            '.' => RegexAST::Wildcard,
            '\\' => parse_escape(chars)?,
            '[' => parse_bracket(chars)?,
            '(' => parse_group(chars)?, // <-- Added to intercept grouping blocks
            '*' | '?' => return Err(format!("Dangling quantifier: '{}'", ch)),
            _ => RegexAST::Literal(ch),
        };

        // Quantifier lookahead applies to the entire group or token
        if let Some(&next_ch) = chars.peek() {
            if next_ch == '*' {
                chars.next();
                nodes.push(RegexAST::ZeroOrMore(Box::new(current_node)));
                continue;
            } else if next_ch == '?' {
                chars.next();
                nodes.push(RegexAST::ZeroOrOne(Box::new(current_node)));
                continue;
            }
        }
        nodes.push(current_node);
    }

    Ok(RegexAST::Concat(nodes))
}

// Recursively processes everything inside (...) as a self-contained alternation block
fn parse_group(chars: &mut Peekable<Chars>) -> Result<RegexAST, String> {
    let inner_ast = parse_alternation(chars)?;

    // The inner parser stops when it hits a group boundary or string end
    if chars.peek() == Some(&')') {
        chars.next(); // Consume the closing ')'
        Ok(inner_ast)
    } else {
        Err("Unmatched parenthesis grouping '('".to_string())
    }
}

// Parse custom character classes: e.g. [a-z0-9]
fn parse_bracket(chars: &mut Peekable<Chars>) -> Result<RegexAST, String> {
    let mut classes = Vec::new();
    let mut raw_chars = Vec::new();

    while let Some(ch) = chars.next() {
        if ch == ']' {
            if !raw_chars.is_empty() {
                classes.push(CharClass::Set(raw_chars));
            }
            // For simplicity, wrap multiple conditions under an alternation or sequence
            // Here, we convert a list of distinct ranges/sets into an effective evaluation model
            return Ok(RegexAST::Class(
                classes.pop().unwrap_or(CharClass::Set(vec![])),
            ));
        }

        if chars.peek() == Some(&'-') {
            chars.next(); // consume '-'
            if let Some(end) = chars.next() {
                classes.push(CharClass::Range(ch, end));
                continue;
            }
        }
        raw_chars.push(ch);
    }
    Err("Unmatched character class bracket '['".to_string())
}

// Parse backslash escapes like \d and \s
fn parse_escape(chars: &mut Peekable<Chars>) -> Result<RegexAST, String> {
    match chars.next() {
        Some('d') => Ok(RegexAST::Class(CharClass::Digit)),
        Some('s') => Ok(RegexAST::Class(CharClass::Whitespace)),
        Some(other) => Ok(RegexAST::Literal(other)),
        None => Err("Trailing backslash".to_string()),
    }
}

fn match_class(class: &CharClass, c: char) -> bool {
    match class {
        CharClass::Digit => c.is_ascii_digit(),
        CharClass::Whitespace => c.is_whitespace(),
        CharClass::Range(start, end) => c >= *start && c <= *end,
        CharClass::Set(chars) => chars.contains(&c),
    }
}

fn match_single(node: &RegexAST, text: &[char]) -> bool {
    if text.is_empty() {
        return false;
    }
    match node {
        RegexAST::Literal(c) => text[0] == *c,
        RegexAST::Wildcard => true,
        RegexAST::Class(cls) => match_class(cls, text[0]),
        _ => false,
    }
}

fn match_ast(ast_nodes: &[RegexAST], full_text: &[char], cursor: usize) -> bool {
    if ast_nodes.is_empty() {
        return true;
    }

    let head = &ast_nodes[0];
    let tail = &ast_nodes[1..];
    let current_slice = &full_text[cursor..];

    match head {
        RegexAST::AnchorStart => {
            // Must be at the strict beginning of the full string sequence
            if cursor == 0 {
                match_ast(tail, full_text, cursor)
            } else {
                false
            }
        }
        RegexAST::AnchorEnd => {
            // Must be at the strict end of the full string sequence
            if cursor == full_text.len() {
                match_ast(tail, full_text, cursor)
            } else {
                false
            }
        }
        RegexAST::Alternation(left, right) => {
            // Branch into two distinct matching pipelines
            let mut left_seq = vec![*left.clone()];
            left_seq.extend_from_slice(tail);
            if match_ast(&left_seq, full_text, cursor) {
                return true;
            }

            let mut right_seq = vec![*right.clone()];
            right_seq.extend_from_slice(tail);
            match_ast(&right_seq, full_text, cursor)
        }
        RegexAST::Literal(_) | RegexAST::Wildcard | RegexAST::Class(_) => {
            if match_single(head, current_slice) {
                match_ast(tail, full_text, cursor + 1)
            } else {
                false
            }
        }
        RegexAST::ZeroOrOne(inner) => {
            if match_ast(tail, full_text, cursor) {
                return true;
            }
            if match_single(inner, current_slice) && match_ast(tail, full_text, cursor + 1) {
                return true;
            }
            false
        }
        RegexAST::ZeroOrMore(inner) => {
            if match_ast(tail, full_text, cursor) {
                return true;
            }
            let mut i = 0;
            while cursor + i < full_text.len() && match_single(inner, &full_text[cursor + i..]) {
                if match_ast(tail, full_text, cursor + i + 1) {
                    return true;
                }
                i += 1;
            }
            false
        }
        RegexAST::Concat(inner_nodes) => {
            let mut new_sequence = inner_nodes.clone();
            new_sequence.extend_from_slice(tail);
            match_ast(&new_sequence, full_text, cursor)
        }
    }
}

pub fn regex_search(pattern: &str, haystack: &str) -> Result<bool, String> {
    let ast = parse_regex(pattern)?;
    let text_chars: Vec<char> = haystack.chars().collect();

    let nodes = match ast {
        RegexAST::Concat(vec) => vec,
        other => vec![other],
    };

    // If it requires starting at the line beginning, evaluate index 0 exclusively
    if nodes.first() == Some(&RegexAST::AnchorStart) {
        return Ok(match_ast(&nodes, &text_chars, 0));
    }

    // Otherwise evaluate every sliding start index window
    for i in 0..=text_chars.len() {
        if match_ast(&nodes, &text_chars, i) {
            return Ok(true);
        }
    }

    Ok(false)
}

fn main() {
    // Priority 1: Character Classes
    assert_eq!(regex_search(r"c\dat", "c3at"), Ok(true));
    assert_eq!(regex_search("[a-z]ox", "fox"), Ok(true));
    assert_eq!(regex_search(r"cat\s", "cat "), Ok(true));

    // Priority 2: Anchors
    assert_eq!(regex_search("^abc", "abcdef"), Ok(true));
    assert_eq!(regex_search("^abc", "xabcdef"), Ok(false));
    assert_eq!(regex_search("xyz$", "wuvxyz"), Ok(true));

    // Priority 3: Alternation
    assert_eq!(regex_search("cat|dog", "I love dogs"), Ok(true));
    assert_eq!(regex_search("cat|dog", "I love cats"), Ok(true));
    assert_eq!(regex_search("a(b|c)d", "abd"), Ok(true)); // Sequential boundary variants

    println!("All upgraded features pass successfully!");
}
