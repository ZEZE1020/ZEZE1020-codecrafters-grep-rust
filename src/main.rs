use std::env;
use std::io;
use std::process;

#[derive(Debug, Clone)]
enum PatternToken {
    Digit,          // \d
    Word,           // \w
    Char(char),     // literal character
    CharGroup(Vec<char>, bool),  // [...] or [^...]
    Plus(Box<PatternToken>),     // token+
    Question(Box<PatternToken>), // token?
}

fn tokenize_pattern(pattern: &str) -> Vec<PatternToken> {
    let mut tokens = Vec::new();
    let mut chars = pattern.chars().peekable();
    
    while let Some(c) = chars.next() {
        let token = match c {
            '\\' => {
                if let Some(special) = chars.next() {
                    match special {
                        'd' => PatternToken::Digit,
                        'w' => PatternToken::Word,
                        _ => PatternToken::Char(special),
                    }
                } else {
                    continue;
                }
            },
            '[' => {
                let is_negative = chars.peek() == Some(&'^');
                if is_negative {
                    chars.next();  // consume '^'
                }
                let mut group_chars = Vec::new();
                while let Some(gc) = chars.next() {
                    if gc == ']' {
                        break;
                    }
                    group_chars.push(gc);
                }
                PatternToken::CharGroup(group_chars, is_negative)
            },
            _ => PatternToken::Char(c),
        };
        
        // Check for quantifier after the token
        if chars.peek() == Some(&'+') {
            chars.next(); // consume '+'
            tokens.push(PatternToken::Plus(Box::new(token)));
        } else if chars.peek() == Some(&'?') {
            chars.next(); // consume '?'
            tokens.push(PatternToken::Question(Box::new(token)));
        } else {
            tokens.push(token);
        }
    }
    tokens
}

fn matches_token(c: char, token: &PatternToken) -> bool {
    match token {
        PatternToken::Digit => c.is_ascii_digit(),
        PatternToken::Word => c.is_ascii_alphanumeric() || c == '_',
        PatternToken::Char(pattern_char) => &c == pattern_char,
        PatternToken::CharGroup(chars, is_negative) => {
            let contains = chars.contains(&c);
            if *is_negative { !contains } else { contains }
        }
        PatternToken::Plus(_) => false, // This should not be called directly
        PatternToken::Question(_) => false, // This should not be called directly
    }
}

fn match_tokens_at_position(input_chars: &[char], tokens: &[PatternToken], start_pos: usize) -> Option<usize> {
    fn backtrack_match(input_chars: &[char], tokens: &[PatternToken], pos: usize, token_idx: usize) -> Option<usize> {
        if token_idx >= tokens.len() {
            return Some(pos);
        }
        
        match &tokens[token_idx] {
            PatternToken::Plus(inner_token) => {
                // Must match at least once
                if pos >= input_chars.len() || !matches_token(input_chars[pos], inner_token) {
                    return None;
                }
                
                // Try different numbers of matches (greedy approach with backtracking)
                let mut max_matches = 1;
                while pos + max_matches < input_chars.len() && 
                      matches_token(input_chars[pos + max_matches], inner_token) {
                    max_matches += 1;
                }
                
                // Try from maximum matches down to minimum (1)
                for num_matches in (1..=max_matches).rev() {
                    if let Some(final_pos) = backtrack_match(input_chars, tokens, pos + num_matches, token_idx + 1) {
                        return Some(final_pos);
                    }
                }
                None
            },
            PatternToken::Question(inner_token) => {
                // Can match zero or one time
                let max_matches = if pos < input_chars.len() && matches_token(input_chars[pos], inner_token) {
                    1
                } else {
                    0
                };
                
                // Try 1 match first (greedy), then 0 matches
                for num_matches in (0..=max_matches).rev() {
                    if let Some(final_pos) = backtrack_match(input_chars, tokens, pos + num_matches, token_idx + 1) {
                        return Some(final_pos);
                    }
                }
                None
            },
            _ => {
                if pos >= input_chars.len() || !matches_token(input_chars[pos], &tokens[token_idx]) {
                    return None;
                }
                backtrack_match(input_chars, tokens, pos + 1, token_idx + 1)
            }
        }
    }
    
    backtrack_match(input_chars, tokens, start_pos, 0)
}

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    let input_chars: Vec<char> = input_line.trim_end().chars().collect();

    // Anchor detection
    let (start_anchored, body) = if let Some(rest) = pattern.strip_prefix('^') {
        (true, rest)
    } else { (false, pattern) };

    let (end_anchored, body) = if let Some(rest) = body.strip_suffix('$') {
        (true, rest)
    } else { (false, body) };

    let tokens = tokenize_pattern(body);

    // Edge cases for anchor-only patterns
    if start_anchored && end_anchored && tokens.is_empty() {
        return input_chars.is_empty();
    }
    if start_anchored && tokens.is_empty() {
        return input_chars.is_empty();
    }
    if end_anchored && tokens.is_empty() {
        return input_chars.is_empty();
    }

    // Try matching at different starting positions
    let start_positions: Vec<usize> = if start_anchored {
        vec![0]
    } else {
        (0..=input_chars.len()).collect()
    };

    for start_pos in start_positions {
        if let Some(end_pos) = match_tokens_at_position(&input_chars, &tokens, start_pos) {
            // If end-anchored, ensure we matched exactly to the end
            if end_anchored {
                if end_pos == input_chars.len() {
                    return true;
                }
            } else {
                return true;
            }
        }
    }
    
    false
}

fn run() -> Result<bool, &'static str> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 3 {
        return Err("Usage: program -E <pattern>");
    }

    if args[1] != "-E" {
        return Err("Expected first argument to be '-E'");
    }

    let pattern = &args[2];
    let mut input_line = String::new();

    io::stdin()
        .read_line(&mut input_line)
        .map_err(|_| "Failed to read input")?;

    Ok(match_pattern(&input_line, pattern))
}

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() {
    process::exit(match run() {
        Ok(true) => 0,
        Ok(false) => 1,
        Err(e) => {
            eprintln!("Error: {}", e);
            1
        }
    });
}
