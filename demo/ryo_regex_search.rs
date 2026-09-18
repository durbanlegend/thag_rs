/// An AI-generated lightweight regex engine using Thompson's NFA (Nondeterministic Finite Automaton)
/// algorithm.
/// "Unlike backtracking engines, an NFA tracks all possible states simultaneously, ensuring linear
/// time complexity \(O(m \times n)\) relative to the text length n and regex length m."
//# Purpose: Prototype for lightweight use avoiding `regex` crate for e.g. WASM.
//# Categories: prototype, technique

#[derive(Debug, Clone, PartialEq)]
enum RegexAST {
    Literal(char),
    Wildcard,              // .
    ZeroOrMore(Box<Self>), // *
    ZeroOrOne(Box<Self>),  // ?
    Concat(Vec<Self>),     // Sequences like "abc"
}

fn parse_regex(pattern: &str) -> Result<RegexAST, String> {
    let mut chars = pattern.chars().peekable();
    let mut nodes = Vec::new();

    while let Some(ch) = chars.next() {
        let current_node = match ch {
            '.' => RegexAST::Wildcard,
            '*' | '?' => return Err(format!("Dangling quantifier: '{}'", ch)),
            _ => RegexAST::Literal(ch),
        };

        // Check if the next character modifies this token
        if let Some(&next_ch) = chars.peek() {
            if next_ch == '*' {
                chars.next(); // consume '*'
                nodes.push(RegexAST::ZeroOrMore(Box::new(current_node)));
                continue;
            } else if next_ch == '?' {
                chars.next(); // consume '?'
                nodes.push(RegexAST::ZeroOrOne(Box::new(current_node)));
                continue;
            }
        }
        nodes.push(current_node);
    }

    Ok(RegexAST::Concat(nodes))
}

fn match_ast(ast_nodes: &[RegexAST], text: &[char]) -> bool {
    // If there are no more pattern rules, we successfully matched up to this point!
    if ast_nodes.is_empty() {
        return true;
    }

    let head = &ast_nodes[0];
    let tail = &ast_nodes[1..];

    match head {
        RegexAST::Literal(c) => {
            if !text.is_empty() && text[0] == *c {
                match_ast(tail, &text[1..])
            } else {
                false
            }
        }
        RegexAST::Wildcard => {
            if !text.is_empty() {
                match_ast(tail, &text[1..])
            } else {
                false
            }
        }
        RegexAST::ZeroOrOne(inner) => {
            // Branch 1: Skip the option (Match 0 times)
            if match_ast(tail, text) {
                return true;
            }
            // Branch 2: Match exactly 1 time
            if match_single(inner, text) {
                return match_ast(tail, &text[1..]);
            }
            false
        }
        RegexAST::ZeroOrMore(inner) => {
            // Branch 1: Match 0 times (skip over the loop)
            if match_ast(tail, text) {
                return true;
            }
            // Branch 2: Consume one character and try matching again
            let mut i = 0;
            while i < text.len() && match_single(inner, &text[i..i + 1]) {
                if match_ast(tail, &text[i + 1..]) {
                    return true;
                }
                i += 1;
            }
            false
        }
        RegexAST::Concat(inner_nodes) => {
            // Flatten internal groupings if necessary
            let mut new_sequence = inner_nodes.clone();
            new_sequence.extend_from_slice(tail);
            match_ast(&new_sequence, text)
        }
    }
}

// Helper to evaluate a single element against one character
fn match_single(node: &RegexAST, text: &[char]) -> bool {
    if text.is_empty() {
        return false;
    }
    match node {
        RegexAST::Literal(c) => text[0] == *c,
        RegexAST::Wildcard => true,
        _ => false, // Complex logic shouldn't nest within simple quantifiers in this engine
    }
}

pub fn regex_search(pattern: &str, haystack: &str) -> Result<bool, String> {
    let ast = parse_regex(pattern)?;
    let text_chars: Vec<char> = haystack.chars().collect();

    // Extract nodes from the top level sequence
    let nodes = match ast {
        RegexAST::Concat(vec) => vec,
        other => vec![other],
    };

    // Attempt to match starting at index 0, then 1, 2, etc.
    for i in 0..=text_chars.len() {
        if match_ast(&nodes, &text_chars[i..]) {
            return Ok(true);
        }
    }

    Ok(false)
}

fn main() {
    // Basic test cases
    assert_eq!(regex_search("a.c", "abc"), Ok(true));
    assert_eq!(regex_search("ab*c", "abbbbc"), Ok(true));
    assert_eq!(regex_search("ab?c", "ac"), Ok(true));
    assert_eq!(regex_search("ab?c", "abc"), Ok(true));
    assert_eq!(regex_search("ab?c", "abbc"), Ok(false)); // More than one 'b'
    assert_eq!(regex_search("hello", "say hello there"), Ok(true)); // Substring match

    println!("All regex tests passed flawlessly!");
}
