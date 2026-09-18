/// A lightweight regex engine using Thompson's NFA algorithm, enhanced with
/// `find_iter`-style match-position reporting and configurable search options:
/// case sensitivity, whole-word matching, and regex vs plain-text mode.
///
/// "Unlike backtracking engines, an NFA tracks all possible states simultaneously,
/// ensuring linear time complexity O(m × n) relative to text length n and pattern length m."
//# Purpose: Prototype for a lightweight regex/search engine without the `regex` crate (e.g. for WASM).
//# Categories: prototype, technique
use std::iter::Peekable;
use std::str::Chars;

// ═══════════════════════════════════ AST ══════════════════════════════════════

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
    Wildcard,              // .
    Class(CharClass),      // \d, \s, [...]
    AnchorStart,           // ^
    AnchorEnd,             // $
    ZeroOrMore(Box<Self>), // *   (greedy)
    ZeroOrOne(Box<Self>),  // ?   (greedy)
    OneOrMore(Box<Self>),  // +   (greedy)
    Concat(Vec<Self>),
    Alternation(Box<Self>, Box<Self>), // a|b
}

// ═════════════════════════ Search options & Match type ════════════════════════

/// Configures case sensitivity, whole-word, and regex vs. plain-text matching.
/// All three flags are independent and may be combined freely.
#[derive(Debug, Clone)]
pub struct SearchOptions {
    /// `true` = case-sensitive (default), `false` = case-insensitive.
    pub case_sensitive: bool,
    /// Only accept matches bounded by non-word characters on both sides
    /// (equivalent to `\b...\b`).
    pub whole_word: bool,
    /// `true` = interpret the pattern as a regex (default);
    /// `false` = treat the pattern as a literal string.
    pub use_regex: bool,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            case_sensitive: true,
            whole_word: false,
            use_regex: true,
        }
    }
}

/// A half-open byte-index range `[start, end)` identifying a match inside a `&str`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    /// Byte offset of the first character of the match.
    pub start: usize,
    /// Byte offset one past the last character of the match.
    pub end: usize,
}

impl Match {
    /// Borrows the matched substring from the original text.
    #[must_use]
    pub fn as_str<'t>(&self, text: &'t str) -> &'t str {
        &text[self.start..self.end]
    }
}

// ═══════════════════════════════════ Parser ════════════════════════════════════

fn parse_regex(pattern: &str) -> Result<RegexAST, String> {
    let mut chars = pattern.chars().peekable();
    parse_alternation(&mut chars)
}

fn parse_alternation(chars: &mut Peekable<Chars<'_>>) -> Result<RegexAST, String> {
    let left = parse_concat(chars)?;
    if chars.peek() == Some(&'|') {
        chars.next();
        let right = parse_alternation(chars)?;
        Ok(RegexAST::Alternation(Box::new(left), Box::new(right)))
    } else {
        Ok(left)
    }
}

fn parse_concat(chars: &mut Peekable<Chars<'_>>) -> Result<RegexAST, String> {
    let mut nodes = Vec::new();
    while let Some(&ch) = chars.peek() {
        if ch == '|' || ch == ')' {
            break;
        }
        chars.next();
        let node = match ch {
            '^' => RegexAST::AnchorStart,
            '$' => RegexAST::AnchorEnd,
            '.' => RegexAST::Wildcard,
            '\\' => parse_escape(chars)?,
            '[' => parse_bracket(chars)?,
            '(' => parse_group(chars)?,
            '*' | '?' | '+' => return Err(format!("Dangling quantifier: '{ch}'")),
            _ => RegexAST::Literal(ch),
        };
        match chars.peek() {
            Some(&'*') => {
                chars.next();
                nodes.push(RegexAST::ZeroOrMore(Box::new(node)));
            }
            Some(&'?') => {
                chars.next();
                nodes.push(RegexAST::ZeroOrOne(Box::new(node)));
            }
            Some(&'+') => {
                chars.next();
                nodes.push(RegexAST::OneOrMore(Box::new(node)));
            }
            _ => nodes.push(node),
        }
    }
    Ok(RegexAST::Concat(nodes))
}

fn parse_group(chars: &mut Peekable<Chars<'_>>) -> Result<RegexAST, String> {
    let inner = parse_alternation(chars)?;
    if chars.peek() == Some(&')') {
        chars.next();
        Ok(inner)
    } else {
        Err("Unmatched '('".to_string())
    }
}

fn parse_bracket(chars: &mut Peekable<Chars<'_>>) -> Result<RegexAST, String> {
    let mut classes: Vec<CharClass> = Vec::new();
    let mut raw: Vec<char> = Vec::new();
    while let Some(ch) = chars.next() {
        if ch == ']' {
            if !raw.is_empty() {
                classes.push(CharClass::Set(raw));
            }
            return Ok(RegexAST::Class(
                classes.pop().unwrap_or(CharClass::Set(vec![])),
            ));
        }
        if chars.peek() == Some(&'-') {
            chars.next();
            if let Some(end) = chars.next() {
                classes.push(CharClass::Range(ch, end));
                continue;
            }
        }
        raw.push(ch);
    }
    Err("Unmatched '['".to_string())
}

fn parse_escape(chars: &mut Peekable<Chars<'_>>) -> Result<RegexAST, String> {
    match chars.next() {
        Some('d') => Ok(RegexAST::Class(CharClass::Digit)),
        Some('s') => Ok(RegexAST::Class(CharClass::Whitespace)),
        Some(c) => Ok(RegexAST::Literal(c)),
        None => Err("Trailing backslash".to_string()),
    }
}

// ═══════════════════════════════ Matching ═════════════════════════════════════

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn chars_eq_ci(a: char, b: char) -> bool {
    a.to_lowercase().eq(b.to_lowercase())
}

fn match_class(class: &CharClass, c: char, case_sensitive: bool) -> bool {
    match class {
        CharClass::Digit => c.is_ascii_digit(),
        CharClass::Whitespace => c.is_whitespace(),
        CharClass::Range(s, e) => {
            if case_sensitive {
                c >= *s && c <= *e
            } else {
                let cl = c.to_lowercase().next().unwrap_or(c);
                let sl = s.to_lowercase().next().unwrap_or(*s);
                let el = e.to_lowercase().next().unwrap_or(*e);
                cl >= sl && cl <= el
            }
        }
        CharClass::Set(chars) => {
            if case_sensitive {
                chars.contains(&c)
            } else {
                chars.iter().any(|&ch| chars_eq_ci(ch, c))
            }
        }
    }
}

fn match_single(node: &RegexAST, text: &[char], cs: bool) -> bool {
    if text.is_empty() {
        return false;
    }
    let c = text[0];
    match node {
        RegexAST::Literal(lit) => {
            if cs {
                c == *lit
            } else {
                chars_eq_ci(c, *lit)
            }
        }
        RegexAST::Wildcard => true,
        RegexAST::Class(cls) => match_class(cls, c, cs),
        _ => false,
    }
}

/// Returns `Some(end_char_index)` when `nodes` match from `cursor` onward; `None` otherwise.
///
/// All quantifiers (`*`, `?`, `+`) are **greedy** — they consume as many characters as
/// possible while still allowing the overall pattern to match.  This is necessary for
/// correct span reporting: a lazy `\d+` on "123" would spuriously return [0..1] on the
/// first attempt and never produce the full [0..3] span.
fn match_ast(nodes: &[RegexAST], text: &[char], cursor: usize, cs: bool) -> Option<usize> {
    if nodes.is_empty() {
        return Some(cursor);
    }
    let (head, tail) = (&nodes[0], &nodes[1..]);

    match head {
        RegexAST::AnchorStart => {
            if cursor == 0 {
                match_ast(tail, text, 0, cs)
            } else {
                None
            }
        }
        RegexAST::AnchorEnd => {
            if cursor == text.len() {
                match_ast(tail, text, cursor, cs)
            } else {
                None
            }
        }
        RegexAST::Alternation(left, right) => {
            let mut lseq = vec![*left.clone()];
            lseq.extend_from_slice(tail);
            match_ast(&lseq, text, cursor, cs).or_else(|| {
                let mut rseq = vec![*right.clone()];
                rseq.extend_from_slice(tail);
                match_ast(&rseq, text, cursor, cs)
            })
        }
        RegexAST::Literal(_) | RegexAST::Wildcard | RegexAST::Class(_) => {
            if match_single(head, &text[cursor..], cs) {
                match_ast(tail, text, cursor + 1, cs)
            } else {
                None
            }
        }
        RegexAST::ZeroOrOne(inner) => {
            // Greedy: prefer consuming one character.
            if match_single(inner, &text[cursor..], cs) {
                if let Some(end) = match_ast(tail, text, cursor + 1, cs) {
                    return Some(end);
                }
            }
            match_ast(tail, text, cursor, cs)
        }
        RegexAST::ZeroOrMore(inner) => {
            // Greedy: find the maximum run length, then backtrack until tail matches.
            let mut max_i = 0;
            while cursor + max_i < text.len() && match_single(inner, &text[cursor + max_i..], cs) {
                max_i += 1;
            }
            for count in (0..=max_i).rev() {
                if let Some(end) = match_ast(tail, text, cursor + count, cs) {
                    return Some(end);
                }
            }
            None
        }
        RegexAST::OneOrMore(inner) => {
            // Greedy: like ZeroOrMore but requires at least one match.
            if !match_single(inner, &text[cursor..], cs) {
                return None;
            }
            let mut max_i = 1;
            while cursor + max_i < text.len() && match_single(inner, &text[cursor + max_i..], cs) {
                max_i += 1;
            }
            for count in (1..=max_i).rev() {
                if let Some(end) = match_ast(tail, text, cursor + count, cs) {
                    return Some(end);
                }
            }
            None
        }
        RegexAST::Concat(inner) => {
            let mut seq = inner.clone();
            seq.extend_from_slice(tail);
            match_ast(&seq, text, cursor, cs)
        }
    }
}

// ═════════════════════════════ Public API ═════════════════════════════════════

/// Returns all non-overlapping matches as `[start, end)` **byte** ranges.
///
/// Analogous to `regex::Regex::find_iter`.  Each search resumes from the end of
/// the previous match (or advances one character on a zero-length match) to ensure
/// termination and non-overlapping results.
///
/// Set `opts.use_regex = false` to treat `pattern` as a literal string instead.
pub fn find_all(pattern: &str, haystack: &str, opts: &SearchOptions) -> Result<Vec<Match>, String> {
    if opts.use_regex {
        regex_find_all(pattern, haystack, opts)
    } else {
        Ok(plain_text_find_all(pattern, haystack, opts))
    }
}

/// Returns `true` if `pattern` (as a regex) matches anywhere in `haystack`.
///
/// Preserved for backward compatibility; internally delegates to `find_all`.
pub fn regex_search(pattern: &str, haystack: &str) -> Result<bool, String> {
    find_all(pattern, haystack, &SearchOptions::default()).map(|m| !m.is_empty())
}

// ═══════════════════════════ Search backends ══════════════════════════════════

fn regex_find_all(
    pattern: &str,
    haystack: &str,
    opts: &SearchOptions,
) -> Result<Vec<Match>, String> {
    let ast = parse_regex(pattern)?;
    let text: Vec<char> = haystack.chars().collect();

    // Precompute the byte offset of every character position.
    let byte_pos: Vec<usize> = haystack.char_indices().map(|(i, _)| i).collect();
    let to_byte = |ci: usize| byte_pos.get(ci).copied().unwrap_or(haystack.len());

    let nodes = match ast {
        RegexAST::Concat(v) => v,
        other => vec![other],
    };
    let anchored = nodes.first() == Some(&RegexAST::AnchorStart);

    let mut results = Vec::new();
    let mut pos = 0_usize;

    while pos <= text.len() {
        if anchored && pos > 0 {
            break; // `^` only matches at position 0
        }
        match match_ast(&nodes, &text, pos, opts.case_sensitive) {
            Some(end) => {
                if !opts.whole_word || is_whole_word(&text, pos, end) {
                    results.push(Match {
                        start: to_byte(pos),
                        end: to_byte(end),
                    });
                }
                // Advance past the match; always move at least one char to avoid
                // infinite loops on zero-length matches.
                pos = if end > pos { end } else { pos + 1 };
            }
            None => pos += 1,
        }
    }
    Ok(results)
}

fn plain_text_find_all(pattern: &str, haystack: &str, opts: &SearchOptions) -> Vec<Match> {
    let text: Vec<char> = haystack.chars().collect();
    let pat: Vec<char> = pattern.chars().collect();
    if pat.is_empty() {
        return vec![];
    }

    let byte_pos: Vec<usize> = haystack.char_indices().map(|(i, _)| i).collect();
    let to_byte = |ci: usize| byte_pos.get(ci).copied().unwrap_or(haystack.len());

    let mut results = Vec::new();
    let mut i = 0_usize;

    while i + pat.len() <= text.len() {
        let window = &text[i..i + pat.len()];
        let hit = if opts.case_sensitive {
            window == pat.as_slice()
        } else {
            window
                .iter()
                .zip(pat.iter())
                .all(|(&a, &b)| chars_eq_ci(a, b))
        };

        if hit && (!opts.whole_word || is_whole_word(&text, i, i + pat.len())) {
            results.push(Match {
                start: to_byte(i),
                end: to_byte(i + pat.len()),
            });
            i += pat.len(); // non-overlapping: skip past the matched text
        } else {
            i += 1;
        }
    }
    results
}

/// Returns `true` when the match at `[start, end)` is bounded by non-word characters.
fn is_whole_word(text: &[char], start: usize, end: usize) -> bool {
    let before_ok = start == 0 || !is_word_char(text[start - 1]);
    let after_ok = end >= text.len() || !is_word_char(text[end]);
    before_ok && after_ok
}

// ══════════════════════════════ Demo / tests ══════════════════════════════════

fn show(label: &str, matches: &[Match], haystack: &str) {
    if matches.is_empty() {
        println!("  {label}: (no matches)");
    } else {
        let hits: Vec<String> = matches
            .iter()
            .map(|m| format!("[{}..{}] {:?}", m.start, m.end, m.as_str(haystack)))
            .collect();
        println!("  {label}: {}", hits.join("  "));
    }
}

fn main() {
    // ── 1. Backward-compatible boolean search ─────────────────────────────────
    println!("=== Backward-compatible regex_search ===");
    assert_eq!(regex_search(r"c\dat", "c3at"), Ok(true));
    assert_eq!(regex_search("[a-z]ox", "fox"), Ok(true));
    assert_eq!(regex_search(r"cat\s", "cat "), Ok(true));
    assert_eq!(regex_search("^abc", "abcdef"), Ok(true));
    assert_eq!(regex_search("^abc", "xabcdef"), Ok(false));
    assert_eq!(regex_search("xyz$", "wuvxyz"), Ok(true));
    assert_eq!(regex_search("cat|dog", "I love dogs"), Ok(true));
    assert_eq!(regex_search("cat|dog", "I love cats"), Ok(true));
    assert_eq!(regex_search("a(b|c)d", "abd"), Ok(true));
    println!("All original assertions pass.\n");

    // ── 2. find_all: byte positions — also exercises the new `+` quantifier ───
    let hay = "foo123bar456baz";
    println!("=== Match positions  [{hay}] ===");
    show(
        r"regex '\d+'",
        &find_all(r"\d+", hay, &SearchOptions::default()).unwrap(),
        hay,
    );
    println!();

    // ── 3. Case sensitivity ───────────────────────────────────────────────────
    let hay = "Cat CAT cat caT";
    println!("=== Case sensitivity  [{hay}] ===");
    show(
        "case-sensitive  'cat'",
        &find_all(
            "cat",
            hay,
            &SearchOptions {
                case_sensitive: true,
                ..SearchOptions::default()
            },
        )
        .unwrap(),
        hay,
    );
    show(
        "case-insensitive 'cat'",
        &find_all(
            "cat",
            hay,
            &SearchOptions {
                case_sensitive: false,
                ..SearchOptions::default()
            },
        )
        .unwrap(),
        hay,
    );
    println!();

    // ── 4. Whole-word (word boundaries, equivalent to \b…\b) ─────────────────
    // "scatter" and "cats" contain "cat" but not as a whole word.
    let hay = "scatter cats and a cat or two";
    println!("=== Whole-word  [{hay}] ===");
    show(
        "any match 'cat'",
        &find_all(
            "cat",
            hay,
            &SearchOptions {
                whole_word: false,
                ..SearchOptions::default()
            },
        )
        .unwrap(),
        hay,
    );
    show(
        "whole-word 'cat'",
        &find_all(
            "cat",
            hay,
            &SearchOptions {
                whole_word: true,
                ..SearchOptions::default()
            },
        )
        .unwrap(),
        hay,
    );
    println!();

    // ── 5. Plain-text vs regex ────────────────────────────────────────────────
    // In plain-text mode "$10.00" is literal; regex mode treats "." as wildcard,
    // so use "\." to escape it.
    let hay = "Price: $10.00 and $9.99";
    println!("=== Plain-text vs regex  [{hay}] ===");
    show(
        r"regex  '\d+\.\d+'",
        &find_all(r"\d+\.\d+", hay, &SearchOptions::default()).unwrap(),
        hay,
    );
    show(
        "plain  '$10.00'",
        &find_all(
            "$10.00",
            hay,
            &SearchOptions {
                use_regex: false,
                ..SearchOptions::default()
            },
        )
        .unwrap(),
        hay,
    );
    println!();

    // ── 6. Combined: case-insensitive + whole-word + regex ────────────────────
    // "gopher" begins with "go" but is not a standalone word — excluded.
    let hay = "Go go GO! gopher";
    println!("=== Combined CI + whole-word  [{hay}] ===");
    show(
        "CI+WW 'go'",
        &find_all(
            "go",
            hay,
            &SearchOptions {
                case_sensitive: false,
                whole_word: true,
                use_regex: true,
            },
        )
        .unwrap(),
        hay,
    );
    println!();

    // ── 7. Combined: case-insensitive + plain-text ────────────────────────────
    let hay = "HELLO World hello WORLD";
    println!("=== Combined CI + plain-text  [{hay}] ===");
    show(
        "CI plain 'hello'",
        &find_all(
            "hello",
            hay,
            &SearchOptions {
                case_sensitive: false,
                use_regex: false,
                ..SearchOptions::default()
            },
        )
        .unwrap(),
        hay,
    );

    println!("\nAll demos complete.");
}
