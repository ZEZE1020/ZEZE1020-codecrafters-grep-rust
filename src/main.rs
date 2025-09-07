use std::env;
use std::io;
use std::process;

#[derive(Debug)]
enum PatternToken {
    Digit,          // \d
    Word,           // \w
    Char(char),     // literal character
    CharGroup(Vec<char>, bool),  // [...] or [^...]
}

fn tokenize_pattern(pattern: &str) -> Vec<PatternToken> {
    let mut tokens = Vec::new();
    let mut chars = pattern.chars().peekable();
    
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if let Some(special) = chars.next() {
                    match special {
                        'd' => tokens.push(PatternToken::Digit),
                        'w' => tokens.push(PatternToken::Word),
                        _ => tokens.push(PatternToken::Char(special)),
                    }
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
                tokens.push(PatternToken::CharGroup(group_chars, is_negative));
            },
            _ => tokens.push(PatternToken::Char(c)),
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
    }
}

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    let tokens = tokenize_pattern(pattern);
    let input_chars: Vec<char> = input_line.trim_end().chars().collect();
    
    if input_chars.len() < tokens.len() {
        return false;
    }
    
    // Try matching at each position in the input
    'outer: for start in 0..=input_chars.len() - tokens.len() {
        let mut pos = start;
        
        // Try to match all tokens from this position
        for token in &tokens {
            if pos >= input_chars.len() {
                continue 'outer;
            }
            
            if !matches_token(input_chars[pos], token) {
                continue 'outer;
            }
            
            pos += 1;
        }
        
        // If we got here, all tokens matched
        return true;
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
