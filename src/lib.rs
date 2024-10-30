use std::str::{Chars, FromStr};

pub fn match_pattern(input: &str, pattern: &str) -> bool {
    println!("[DEBUG] pattern is '{pattern}'");
    if pattern.chars().count() == 1 {
        return input.contains(pattern);
    } else {
        let regex: Regex = Regex::from_str(pattern).expect("Regex parsing failed");
        return regex.match_input(input, &regex.pattern_vec, true);
    }
}

#[derive(Debug)]
enum PatternKind {
    SingleChar(char),
    AnyChar,
    Digit,
    AlphaNumeric,
    CharSet {
        literal: String,
        is_complement: bool,
    },
    OneOrMore(char),
    ZeroOrOne(char),
    StartAnchor(usize),
    EndAnchor(usize),
}

struct Regex {
    pattern_vec: Vec<PatternKind>,
    pattern_len: usize,
}

impl FromStr for Regex {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iterator = s.chars().peekable();
        let mut pattern_buffer: Vec<PatternKind> = Vec::new();
        let mut index: usize = 0;
        while let Some(c) = iterator.next() {
            let pattern = match c {
                '\\' => match iterator.next() {
                    Some('d') => PatternKind::Digit,
                    Some('w') => PatternKind::AlphaNumeric,
                    Some('\\') => PatternKind::SingleChar('\\'),
                    None => return Err(format!("[FATAL] Bad escape char: {:?}", s)),
                    _ => {
                        todo!()
                    }
                },
                '^' => PatternKind::StartAnchor(index),
                '$' => PatternKind::EndAnchor(index),
                '.' => PatternKind::AnyChar,
                '[' => {
                    let mut literal = String::new();
                    let mut is_complement = false;
                    let mut is_at_end: bool = false;
                    for cc in iterator.by_ref() {
                        match cc {
                            '^' if literal.is_empty() => is_complement = true,
                            ']' => {
                                is_at_end = true;
                                break;
                            }
                            _ => literal.push(cc),
                        }
                    }
                    if !is_at_end {
                        return Err("[FATAL] No closing bracket for charset".into());
                    }
                    PatternKind::CharSet {
                        literal,
                        is_complement,
                    }
                }
                other => {
                    if iterator.next_if(|cc| *cc == '+').is_some() {
                        iterator.next();
                        PatternKind::OneOrMore(other)
                    } else if iterator.next_if(|cc| *cc == '?').is_some() {
                        PatternKind::ZeroOrOne(other)
                    } else {
                        PatternKind::SingleChar(other)
                    }
                }
            };
            pattern_buffer.push(pattern);
            index += 1;
        }
        Ok(Regex {
            pattern_vec: pattern_buffer,
            pattern_len: s.len(),
        })
    }
}

impl Regex {
    pub fn match_input(&self, input: &str, patterns: &[PatternKind], is_skip: bool) -> bool {
        println!("[DEBUG] parsed pattern:");
        for pattern in patterns {
            println!(">>> {:?}", pattern)
        }
        'input_loop: for i in 0..input.len() {
            let current = &input[i..];
            let mut iter = current.chars();
            for pattern in patterns.iter() {
                match pattern {
                    PatternKind::AlphaNumeric => {
                        if !match_alphanumeric(&mut iter) {
                            if is_skip {
                                continue 'input_loop;
                            } else {
                                return false;
                            }
                        }
                    }
                    PatternKind::Digit => {
                        if !match_digit(&mut iter) {
                            if is_skip {
                                continue 'input_loop;
                            } else {
                                return false;
                            }
                        }
                    }
                    PatternKind::CharSet {
                        literal,
                        is_complement,
                    } => {
                        if !match_charset(&mut iter, literal, *is_complement) {
                            if is_skip {
                                continue 'input_loop;
                            } else {
                                return false;
                            }
                        }
                    }
                    PatternKind::SingleChar(c) => {
                        if !match_single_char(&mut iter, *c) {
                            if is_skip {
                                continue 'input_loop;
                            } else {
                                return false;
                            }
                        }
                    }
                    PatternKind::ZeroOrOne(target) => {
                        if !match_zero_or_more(&mut iter, *target) {
                            if is_skip {
                                continue 'input_loop;
                            } else {
                                return false;
                            }
                        }
                    },
                    PatternKind::OneOrMore(target) => {
                        if !match_one_or_more(&mut iter, *target) {
                            if is_skip {
                                continue 'input_loop;
                            } else {
                                return false;
                            }
                        }
                    },
                    PatternKind::EndAnchor(index) if *index == self.pattern_len - 1 => {
                        if let Some(_) = iter.next() {
                            if is_skip {
                                continue 'input_loop;
                            } else {
                                return false;
                            }
                        }
                    }
                    PatternKind::StartAnchor(index) if *index == 0 => {
                        return self.match_input(input, &patterns[1..], false);
                    }
                    _ => panic!("[FATAL] Pattern is not valid!"),
                }
            }
            return true;
        }
        false
    }
}

fn match_alphanumeric(chars: &mut Chars) -> bool {
    let current = chars.next();
    current.is_some_and(|c| c.is_alphanumeric())
}

fn match_digit(chars: &mut Chars) -> bool {
    let current = chars.next();
    current.is_some_and(|c| c.is_ascii_digit())
}

fn match_charset(chars: &mut Chars, literal: &str, is_complement: bool) -> bool {
    let current = chars.next();
    match is_complement {
        true => current.is_some_and(|c| !literal.contains(c)),
        false => current.is_some_and(|c| literal.contains(c)),
    }
}

fn match_single_char(chars: &mut Chars, single_char: char) -> bool {
    let current = chars.next();
    current.is_some_and(|c| c == single_char)
}

fn match_one_or_more(chars: &mut Chars, target: char) -> bool {
    let mut index: usize = 0;
    while let Some(c) = chars.next() {
        if c == target {
            index += 1;
        } else {
            break;
        }
    }
    index >= 1
}

fn match_zero_or_more(chars: &mut Chars, target: char) -> bool {
    let mut index: usize = 0;
    while let Some(c) = chars.next() {
        if c == target {
            index += 1;
        } else {
            break;
        }
    }
    index == 0 || index == 1
}

fn _peek_char(chars: &mut Chars) -> Option<char> {
    chars.clone().next()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn multiple_character_classes() {
        let result: bool = match_pattern("sally has 3 apples", r"\d apples");
        assert_eq!(result, true)
    }

    #[test]
    fn start_anchor() {
        let result: bool = match_pattern("slog", r"^log");
        assert_eq!(result, false)
    }

    #[test]
    fn end_anchor() {
        let result: bool = match_pattern("log", r"log$");
        assert_eq!(result, true)
    }

    #[test]
    fn one_or_more() {
        let result: bool = match_pattern("cats", "^ca+ts");
        assert_eq!(result, true)
    }

    #[test]
    fn zero_or_more() {
        let result: bool = match_pattern("cat", "ca?t");
        assert_eq!(result, true)
    }
}
