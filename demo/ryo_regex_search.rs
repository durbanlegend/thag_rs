/// A lightweight backtracking regex engine for environments where the `regex`
/// crate's size or compile-time overhead is unacceptable (e.g. WASM).
///
/// Supported syntax: literals, `.` wildcard, `^`/`$` anchors, `\b` word
/// boundary, character classes (`[a-z]`, `\d`, `\s` — including inside `[...]`),
/// quantifiers (`*`, `?`, `+`, `{n}`, `{n,}`, `{n,m}` — all greedy), grouping
/// (`(...)`, `(?:...)`), positive lookahead (`(?=...)`), alternation (`a|b`),
/// plus configurable case-insensitive, whole-word, and plain-text search modes.
///
/// Not supported: negative/lookbehind assertions, lazy quantifiers,
/// backreferences, or Unicode properties.
//# Purpose: Prototype for a lightweight regex/search engine without the `regex` crate (e.g. for WASM).
//# Categories: prototype, technique
use core::ops::Range;
use std::iter::Peekable;
use std::str::Chars;

// ═══════════════════════════════════ AST ══════════════════════════════════════

#[derive(Debug, Clone, PartialEq)]
enum CharClass {
    Digit,                 // \d
    Whitespace,            // \s
    Range(char, char),     // [a-z]
    Set(Vec<char>),        // [abc]
    Multi(Vec<CharClass>), // [a-zA-Z0-9] — union of multiple sub-classes
}

#[derive(Debug, Clone, PartialEq)]
enum RegexAST {
    Literal(char),
    Wildcard,                                // .
    Class(CharClass),                        // \d, \s, [...]
    AnchorStart,                             // ^
    AnchorEnd,                               // $
    WordBoundary,                            // \b
    ZeroOrMore(Box<Self>),                   // *         (greedy)
    ZeroOrOne(Box<Self>),                    // ?         (greedy)
    OneOrMore(Box<Self>),                    // +         (greedy)
    Repeat(Box<Self>, usize, Option<usize>), // {n,m}     (greedy, no catastrophic backtracking)
    LookAhead(Box<Self>),                    // (?=...)   (zero-width positive lookahead)
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

    /// Returns the match as a half-open byte range, usable directly as a slice index.
    ///
    /// ```
    /// let m = Match { start: 6, end: 11 };
    /// assert_eq!(m.range(), 6..11);
    /// assert_eq!(&"hello world"[m.range()], "world");
    /// ```
    #[must_use]
    pub fn range(&self) -> Range<usize> {
        self.start..self.end
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
            Some(&'{') => {
                // `{n}`, `{n,}`, or `{n,m}` repetition quantifier.
                chars.next(); // consume '{'
                let min = parse_number(chars)?;
                let max_opt = if chars.peek() == Some(&',') {
                    chars.next(); // consume ','
                    if chars.peek() == Some(&'}') {
                        None // {n,} — unbounded upper limit
                    } else {
                        Some(parse_number(chars)?)
                    }
                } else {
                    Some(min) // {n} — exact count
                };
                if chars.next() != Some('}') {
                    return Err("Expected '}' to close repetition quantifier".to_string());
                }
                if let Some(max) = max_opt {
                    if max < min {
                        return Err(format!("{{{min},{max}}} invalid: min > max"));
                    }
                }
                // Emit a single Repeat node.  Expanding into stacked ZeroOrOne nodes
                // causes O(C(n+m,n)) catastrophic backtracking when the character
                // class overlaps with a required literal later in the pattern.
                nodes.push(RegexAST::Repeat(Box::new(node), min, max_opt));
            }
            _ => nodes.push(node),
        }
    }
    Ok(RegexAST::Concat(nodes))
}

fn parse_group(chars: &mut Peekable<Chars<'_>>) -> Result<RegexAST, String> {
    // Handle non-capturing groups `(?:...)` by consuming the `?:` prefix.
    // Our engine treats them identically to capturing groups `(...)` since
    // it never exposes capture groups to the caller anyway.
    if chars.peek() == Some(&'?') {
        chars.next(); // consume '?'
        match chars.next() {
            Some(':') => {} // non-capturing group `(?:...)`: treat as plain `(...)`
            Some('=') => {
                // Positive lookahead `(?=...)`: zero-width assertion.
                let inner = parse_alternation(chars)?;
                if chars.next() != Some(')') {
                    return Err("Unmatched '(' in lookahead".to_string());
                }
                return Ok(RegexAST::LookAhead(Box::new(inner)));
            }
            Some(ch) => return Err(format!("Unsupported group modifier '(?{ch}'")),
            None => return Err("Unexpected end of pattern after '(?'".to_string()),
        }
    }
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
                classes.push(CharClass::Set(std::mem::take(&mut raw)));
            }
            return Ok(RegexAST::Class(match classes.len() {
                0 => CharClass::Set(vec![]),
                1 => classes.remove(0),
                _ => CharClass::Multi(classes),
            }));
        }
        // Backslash escapes inside a character class: `\d`, `\s`, or any literal.
        // This correctly handles patterns like `[A-Za-z\d]` and `[\+]`.
        if ch == '\\' {
            if !raw.is_empty() {
                classes.push(CharClass::Set(std::mem::take(&mut raw)));
            }
            match chars.next() {
                Some('d') => classes.push(CharClass::Digit),
                Some('s') => classes.push(CharClass::Whitespace),
                Some(c) => raw.push(c), // e.g. `\+` → literal `+`
                None => return Err("Trailing backslash in character class".to_string()),
            }
            continue;
        }
        if chars.peek() == Some(&'-') {
            // Peek one step further: if '-' is immediately before ']' (or end of input)
            // it is a literal hyphen, not a range operator.
            chars.next(); // tentatively consume '-'
            match chars.peek() {
                Some(&']') | None => {
                    // Literal '-' at the end of the class.
                    raw.push(ch);
                    raw.push('-');
                }
                Some(_) => {
                    // Genuine range: flush pending raw chars first so they form their
                    // own Set, then push the Range.
                    if !raw.is_empty() {
                        classes.push(CharClass::Set(std::mem::take(&mut raw)));
                    }
                    let end_ch = chars.next().unwrap(); // safe: we just peeked Some(_)
                    classes.push(CharClass::Range(ch, end_ch));
                }
            }
            continue;
        }
        raw.push(ch);
    }
    Err("Unmatched '['".to_string())
}

/// Parses a decimal number from the char stream (used by `{n,m}` quantifiers).
fn parse_number(chars: &mut Peekable<Chars<'_>>) -> Result<usize, String> {
    let mut digits = String::new();
    while let Some(&ch) = chars.peek() {
        if ch.is_ascii_digit() {
            chars.next();
            digits.push(ch);
        } else {
            break;
        }
    }
    if digits.is_empty() {
        Err("Expected a number in repetition quantifier".to_string())
    } else {
        digits.parse::<usize>().map_err(|e| e.to_string())
    }
}

fn parse_escape(chars: &mut Peekable<Chars<'_>>) -> Result<RegexAST, String> {
    match chars.next() {
        Some('d') => Ok(RegexAST::Class(CharClass::Digit)),
        Some('s') => Ok(RegexAST::Class(CharClass::Whitespace)),
        Some('b') => Ok(RegexAST::WordBoundary),
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
        CharClass::Multi(classes) => classes
            .iter()
            .any(|cls| match_class(cls, c, case_sensitive)),
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
        RegexAST::WordBoundary => {
            // Zero-width assertion: succeeds when the position is between a word
            // character (alphanumeric / '_') and a non-word character (or string edge).
            let before_word = cursor > 0 && is_word_char(text[cursor - 1]);
            let after_word = cursor < text.len() && is_word_char(text[cursor]);
            if before_word != after_word {
                match_ast(tail, text, cursor, cs)
            } else {
                None
            }
        }
        RegexAST::ZeroOrOne(inner) => {
            // Greedy: try one occurrence of inner (which may span multiple characters)
            // by prepending it to the remaining tail and delegating to match_ast.
            // This correctly handles multi-char inners such as groups `(?:...)`.
            let mut one_seq = vec![inner.as_ref().clone()];
            one_seq.extend_from_slice(tail);
            if let Some(end) = match_ast(&one_seq, text, cursor, cs) {
                return Some(end);
            }
            match_ast(tail, text, cursor, cs)
        }
        RegexAST::Repeat(inner, min, max_opt) => {
            // Greedy repetition that avoids catastrophic backtracking.
            //
            // Strategy: make one linear forward pass to collect every cursor position
            // reachable after 0, 1, 2, … consecutive matches of `inner`.  Then try
            // appending `tail` from the longest match down to `min`.  Each tail
            // attempt is independent — we never re-explore inner matches.
            //
            // This mirrors how `ZeroOrMore` handles single-char atoms (counting
            // forward, trying tail positions backward) but works for any inner
            // pattern, including multi-character groups.
            let inner_nodes: Vec<RegexAST> = match inner.as_ref() {
                RegexAST::Concat(v) => v.clone(),
                other => vec![other.clone()],
            };
            let max_count = max_opt.unwrap_or(text.len().saturating_add(1));

            // positions[k] = cursor after k successful matches of inner.
            let mut positions = vec![cursor];
            while positions.len() <= max_count {
                let last = *positions.last().unwrap();
                match match_ast(&inner_nodes, text, last, cs) {
                    Some(end) if end > last => positions.push(end),
                    _ => break,
                }
            }

            if positions.len() <= *min {
                return None; // fewer than `min` matches achieved
            }

            // Greedy: try from the longest match down to `min`.
            for i in (*min..positions.len()).rev() {
                if let Some(end) = match_ast(tail, text, positions[i], cs) {
                    return Some(end);
                }
            }
            None
        }
        RegexAST::LookAhead(inner) => {
            // Zero-width positive lookahead: test whether `inner` matches at the
            // current position WITHOUT consuming any characters.  If the sub-match
            // succeeds, continue matching `tail` from the same cursor.
            let inner_nodes: Vec<RegexAST> = match inner.as_ref() {
                RegexAST::Concat(v) => v.clone(),
                other => vec![other.clone()],
            };
            match match_ast(&inner_nodes, text, cursor, cs) {
                Some(_) => match_ast(tail, text, cursor, cs),
                None => None,
            }
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

/// Runs `find_all` and returns each match as a `core::ops::Range<usize>` byte range.
///
/// The ranges are half-open `[start, end)` byte offsets into `haystack`, identical
/// to those produced by the `regex` crate's `find_iter`.  They can be used directly
/// as slice indices: `&haystack[range]`.
pub fn find_ranges(
    pattern: &str,
    haystack: &str,
    opts: &SearchOptions,
) -> Result<Vec<Range<usize>>, String> {
    find_all(pattern, haystack, opts)
        .map(|matches| matches.into_iter().map(|m| m.range()).collect())
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
    assert_eq!(
        regex_search(
            r#"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"#,
            "user@mail.example.co.uk"
        ),
        Ok(true)
    );
    assert_eq!(regex_search("[.]", "8 k81.5 T71"), Ok(true));
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

// ══════════════════════════════ Unit tests ════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── Core: Match::range ─────────────────────────────────────────────────────

    #[test]
    fn test_ryo_regex_search_match_range_method() {
        let m = Match { start: 6, end: 11 };
        assert_eq!(m.range(), 6..11);
        // Verify the range works as a direct slice index.
        assert_eq!(&"hello world"[m.range()], "world");
    }

    // ── Literal patterns ──────────────────────────────────────────────────────

    #[test]
    fn test_ryo_regex_search_literal() {
        let ranges = find_ranges("cat", "the cat sat on a cat", &SearchOptions::default()).unwrap();
        assert_eq!(ranges, vec![4..7, 17..20]);
    }

    // ── Wildcard ──────────────────────────────────────────────────────────────

    #[test]
    fn test_ryo_regex_search_wildcard() {
        // '.' is a wildcard that matches any character, including space.
        let ranges = find_ranges("a.c", "abc axc a c", &SearchOptions::default()).unwrap();
        assert_eq!(ranges, vec![0..3, 4..7, 8..11]);
    }

    // ── Quantifiers ───────────────────────────────────────────────────────────

    #[test]
    fn test_ryo_regex_search_zero_or_more() {
        // 'ab*c' matches 'ac', 'abc', 'abbc'
        let ranges = find_ranges("ab*c", "ac abc abbc", &SearchOptions::default()).unwrap();
        assert_eq!(ranges, vec![0..2, 3..6, 7..11]);
    }

    #[test]
    fn test_ryo_regex_search_zero_or_one() {
        // 'colou?r' matches both 'color' and 'colour' (greedy: tries 'u' first)
        let ranges = find_ranges("colou?r", "color and colour", &SearchOptions::default()).unwrap();
        assert_eq!(ranges, vec![0..5, 10..16]);
    }

    #[test]
    fn test_ryo_regex_search_one_or_more() {
        // Greedy '+' consumes the longest possible run of digits.
        let ranges = find_ranges(r"\d+", "foo123bar456baz", &SearchOptions::default()).unwrap();
        assert_eq!(ranges, vec![3..6, 9..12]);
    }

    #[test]
    fn test_ryo_regex_search_exact_repetition() {
        // {4} matches exactly four consecutive digits.
        let ranges =
            find_ranges(r"\d{4}", "in 2024 and 2025 AD", &SearchOptions::default()).unwrap();
        assert_eq!(ranges, vec![3..7, 12..16]);
    }

    #[test]
    fn test_ryo_regex_search_min_repetition() {
        // {2,} matches runs of two or more digits; lone digits are skipped.
        let ranges = find_ranges(r"\d{2,}", "1 22 333 9 44", &SearchOptions::default()).unwrap();
        assert_eq!(ranges, vec![2..4, 5..8, 11..13]);
    }

    #[test]
    fn test_ryo_regex_search_bounded_repetition() {
        // {2,3} is greedy: "1234" yields "123" then "4" is too short to match again.
        let ranges = find_ranges(r"\d{2,3}", "1 22 333 1234 9", &SearchOptions::default()).unwrap();
        assert_eq!(ranges, vec![2..4, 5..8, 9..12]);
    }

    // ── Character classes ──────────────────────────────────────────────────────

    #[test]
    fn test_ryo_regex_search_bracket_multi_range() {
        // [a-zA-Z]+ must union both ranges — the historical bug returned only one.
        let ranges = find_ranges("[a-zA-Z]+", "abc123DEF", &SearchOptions::default()).unwrap();
        assert_eq!(ranges, vec![0..3, 6..9]);
    }

    #[test]
    fn test_ryo_regex_search_bracket_hyphen_literal() {
        // '-' at the end of a class is a literal hyphen, not a range operator.
        let ranges = find_ranges("[a-z-]+", "hello-world", &SearchOptions::default()).unwrap();
        assert_eq!(ranges, vec![0..11]);
    }

    #[test]
    fn test_ryo_regex_search_digit_class() {
        let ranges = find_ranges(r"\d+", "abc 42 xyz 7 99", &SearchOptions::default()).unwrap();
        assert_eq!(ranges, vec![4..6, 11..12, 13..15]);
    }

    // ── Anchors ────────────────────────────────────────────────────────────────

    #[test]
    fn test_ryo_regex_search_anchor_start() {
        assert_eq!(
            find_ranges("^hello", "hello world", &SearchOptions::default()).unwrap(),
            vec![0..5]
        );
        assert!(
            find_ranges("^hello", "say hello", &SearchOptions::default())
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn test_ryo_regex_search_anchor_end() {
        assert_eq!(
            find_ranges("world$", "hello world", &SearchOptions::default()).unwrap(),
            vec![6..11]
        );
        assert!(
            find_ranges("world$", "worlds apart", &SearchOptions::default())
                .unwrap()
                .is_empty()
        );
    }

    // ── Alternation ────────────────────────────────────────────────────────────

    #[test]
    fn test_ryo_regex_search_alternation() {
        let ranges = find_ranges(
            "cat|dog|fish",
            "I have a cat and a dog and a fish",
            &SearchOptions::default(),
        )
        .unwrap();
        assert_eq!(ranges, vec![9..12, 19..22, 29..33]);
    }

    // ── Case sensitivity ───────────────────────────────────────────────────────

    #[test]
    fn test_ryo_regex_search_case_sensitive() {
        let opts = SearchOptions {
            case_sensitive: true,
            ..SearchOptions::default()
        };
        assert_eq!(
            find_ranges("rust", "Rust rust RUST", &opts).unwrap(),
            vec![5..9]
        );
    }

    #[test]
    fn test_ryo_regex_search_case_insensitive() {
        let opts = SearchOptions {
            case_sensitive: false,
            ..SearchOptions::default()
        };
        assert_eq!(
            find_ranges("rust", "Rust rust RUST", &opts).unwrap(),
            vec![0..4, 5..9, 10..14]
        );
    }

    // ── Whole-word ─────────────────────────────────────────────────────────────

    #[test]
    fn test_ryo_regex_search_whole_word() {
        let opts = SearchOptions {
            whole_word: true,
            ..SearchOptions::default()
        };
        // "scatter" and "cats" embed "cat" but not as whole words.
        assert_eq!(
            find_ranges("cat", "scatter cats and a cat or two", &opts).unwrap(),
            vec![19..22]
        );
    }

    #[test]
    fn test_ryo_regex_search_whole_word_punctuation_boundary() {
        // '-' is not a word character, so "cat" in "cat-nap" IS a whole-word match.
        let opts = SearchOptions {
            whole_word: true,
            ..SearchOptions::default()
        };
        assert_eq!(
            find_ranges("cat", "cat-nap and cat", &opts).unwrap(),
            vec![0..3, 12..15]
        );
    }

    // ── Plain-text mode ────────────────────────────────────────────────────────

    #[test]
    fn test_ryo_regex_search_plain_text() {
        let opts = SearchOptions {
            use_regex: false,
            ..SearchOptions::default()
        };
        // '$' and '.' are regex metacharacters but are treated as literals here.
        assert_eq!(
            find_ranges("$10.00", "Price: $10.00 and $10.00", &opts).unwrap(),
            vec![7..13, 18..24]
        );
    }

    #[test]
    fn test_ryo_regex_search_plain_text_case_insensitive() {
        let opts = SearchOptions {
            use_regex: false,
            case_sensitive: false,
            whole_word: false,
        };
        assert_eq!(
            find_ranges("hello", "HELLO hello Hello", &opts).unwrap(),
            vec![0..5, 6..11, 12..17]
        );
    }

    // ── Real-world patterns ────────────────────────────────────────────────────

    #[test]
    fn test_ryo_regex_search_email_valid() {
        let pat = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$";
        let opts = SearchOptions::default();
        assert_eq!(
            find_ranges(pat, "user@example.com", &opts).unwrap(),
            vec![0..16]
        );
        assert_eq!(
            find_ranges(pat, "user@mail.example.co.uk", &opts).unwrap(),
            vec![0..23]
        );
        assert_eq!(
            find_ranges(pat, "first.last+tester@domain.com", &opts).unwrap(),
            vec![0..28]
        );
    }

    #[test]
    fn test_ryo_regex_search_email_invalid() {
        let pat = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$";
        let opts = SearchOptions::default();
        assert!(find_ranges(pat, "not-an-email", &opts).unwrap().is_empty());
        assert!(find_ranges(pat, "@nodomain.com", &opts).unwrap().is_empty());
        assert!(find_ranges(pat, "missing@dot", &opts).unwrap().is_empty());
        assert!(
            find_ranges(pat, "userexample.com", &opts)
                .unwrap()
                .is_empty()
        );
        assert!(find_ranges(pat, "user@.com", &opts).unwrap().is_empty());
        assert!(
            find_ranges(pat, "user@example.c", &opts)
                .unwrap()
                .is_empty()
        );
    }

    fn assert_range_match(pattern: &str, haystack: &str, opts: &SearchOptions) {
        assert_eq!(
            find_ranges(pattern, haystack, opts).unwrap(),
            vec![Range {
                start: 0,
                end: haystack.len()
            }]
        );
    }

    #[test]
    fn test_ryo_regex_search_url_valid() {
        let pat = r"^https?:\/\/(?:www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{2,6}\b(?:[-a-zA-Z0-9()@:%_\+.~#?& text=\/=]*)$";
        let opts = SearchOptions::default();
        assert_range_match(pat, "https://example.com", &opts);
        assert_range_match(pat, "https://example.org", &opts);
        assert_range_match(
            pat,
            "https://www.google.com/search?q=regex+test+cases&sourceid=chrome&source=chrome.ob&ie=UTF-8",
            &opts,
        );
    }

    #[test]
    fn test_ryo_regex_search_url_invalid() {
        let pat = r"^https?:\/\/(?:www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{2,6}\b(?:[-a-zA-Z0-9()@:%_\+.~#?& text=\/=]*)$";
        let opts = SearchOptions::default();
        assert!(find_ranges(pat, "example.com", &opts).unwrap().is_empty());
        assert!(
            find_ranges(pat, "https:/example.com", &opts)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn test_ryo_regex_search_password_valid() {
        let pat = r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&])[A-Za-z\d@$!%*?&]{8,}$";
        let opts = SearchOptions::default();
        assert_range_match(pat, "P@ssword123", &opts);
    }

    #[test]
    fn test_ryo_regex_search_password_invalid() {
        let pat = r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&])[A-Za-z\d@$!%*?&]{8,}$";
        let opts = SearchOptions::default();
        assert!(find_ranges(pat, "P@ss1", &opts).unwrap().is_empty());
        assert!(find_ranges(pat, "Password@", &opts).unwrap().is_empty());
        assert!(find_ranges(pat, "Password123", &opts).unwrap().is_empty());
        assert!(find_ranges(pat, "p@ssword123", &opts).unwrap().is_empty());
    }

    #[test]
    fn test_ryo_regex_search_decimal_numbers() {
        let ranges = find_ranges(
            r"\d+\.\d+",
            "pi is 3.14 and e is 2.718",
            &SearchOptions::default(),
        )
        .unwrap();
        assert_eq!(ranges, vec![6..10, 20..25]);
    }

    #[test]
    fn test_ryo_regex_search_iso_dates() {
        let ranges = find_ranges(
            r"\d{4}-\d{2}-\d{2}",
            "Born 1990-07-04, died 2024-12-31.",
            &SearchOptions::default(),
        )
        .unwrap();
        assert_eq!(ranges, vec![5..15, 22..32]);
    }

    #[test]
    fn test_ryo_regex_search_hex_colours() {
        let ranges = find_ranges(
            r"#[a-fA-F0-9]{6}",
            "bg: #ff0000; fg: #1A2B3C;",
            &SearchOptions::default(),
        )
        .unwrap();
        assert_eq!(ranges, vec![4..11, 17..24]);
    }

    #[test]
    fn test_ryo_regex_search_ip_address() {
        let ranges = find_ranges(
            r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}",
            "Server: 192.168.1.1 gateway 10.0.0.1",
            &SearchOptions::default(),
        )
        .unwrap();
        assert_eq!(ranges, vec![8..19, 28..36]);
    }
}
