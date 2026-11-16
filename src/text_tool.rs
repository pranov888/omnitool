#[derive(Debug, PartialEq, Eq)]
pub struct TextStats {
    pub lines: usize,
    pub words: usize,
    pub characters: usize,
    pub bytes: usize,
}

pub fn analyze_text(input: &str) -> TextStats {
    let lines = if input.is_empty() {
        0
    } else {
        input.lines().count()
    };
    let words = input.split_whitespace().count();
    let characters = input.chars().count();
    let bytes = input.len();

    TextStats {
        lines,
        words,
        characters,
        bytes,
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum CaseStyle {
    Snake,
    Camel,
    Kebab,
    Upper,
    Lower,
    Title,
}

pub fn convert_case(input: &str, style: CaseStyle) -> String {
    // Split on spaces, underscores, hyphens, and camelCase boundaries
    let mut words = Vec::new();
    let mut current_word = String::new();

    let mut prev_is_lower = false;
    for ch in input.chars() {
        if ch == ' ' || ch == '_' || ch == '-' {
            if !current_word.is_empty() {
                words.push(current_word.to_lowercase());
                current_word = String::new();
            }
            prev_is_lower = false;
        } else if ch.is_uppercase() {
            if prev_is_lower && !current_word.is_empty() {
                words.push(current_word.to_lowercase());
                current_word = String::new();
            }
            current_word.push(ch);
            prev_is_lower = false;
        } else {
            current_word.push(ch);
            prev_is_lower = true;
        }
    }

    if !current_word.is_empty() {
        words.push(current_word.to_lowercase());
    }

    if words.is_empty() {
        return String::new();
    }

    match style {
        CaseStyle::Snake => words.join("_"),
        CaseStyle::Kebab => words.join("-"),
        CaseStyle::Upper => words.join("_").to_uppercase(),
        CaseStyle::Lower => words.join(" "),
        CaseStyle::Title => words
            .iter()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    Some(first) => {
                        let mut s = first.to_uppercase().to_string();
                        s.push_str(chars.as_str());
                        s
                    }
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" "),
        CaseStyle::Camel => {
            let mut result = words[0].clone();
            for word in words.iter().skip(1) {
                let mut chars = word.chars();
                if let Some(first) = chars.next() {
                    result.push_str(&first.to_uppercase().to_string());
                    result.push_str(chars.as_str());
                }
            }
            result
        }
    }
}

pub fn slugify(input: &str) -> String {
    let mut slug = String::with_capacity(input.len());
    let mut last_was_dash = false;

    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_was_dash = false;
        } else if !last_was_dash && !slug.is_empty() {
            slug.push('-');
            last_was_dash = true;
        }
    }

    if slug.ends_with('-') {
        slug.pop();
    }

    slug
}

pub fn reverse_text(input: &str) -> String {
    input.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_text() {
        let text = "Line one\nLine two has more words\nLine three";
        let stats = analyze_text(text);
        assert_eq!(stats.lines, 3);
        assert_eq!(stats.words, 9);
        assert_eq!(stats.characters, 43);
        assert_eq!(stats.bytes, 43);
    }

    #[test]
    fn test_analyze_empty() {
        let stats = analyze_text("");
        assert_eq!(stats.lines, 0);
        assert_eq!(stats.words, 0);
        assert_eq!(stats.characters, 0);
        assert_eq!(stats.bytes, 0);
    }

    #[test]
    fn test_convert_case_snake() {
        assert_eq!(convert_case("HelloWorld", CaseStyle::Snake), "hello_world");
        assert_eq!(
            convert_case("kebab-case-text", CaseStyle::Snake),
            "kebab_case_text"
        );
    }

    #[test]
    fn test_convert_case_kebab() {
        assert_eq!(convert_case("Hello_World", CaseStyle::Kebab), "hello-world");
    }

    #[test]
    fn test_convert_case_camel() {
        assert_eq!(
            convert_case("hello_world_text", CaseStyle::Camel),
            "helloWorldText"
        );
    }

    #[test]
    fn test_convert_case_upper() {
        assert_eq!(convert_case("hello world", CaseStyle::Upper), "HELLO_WORLD");
    }

    #[test]
    fn test_convert_case_title() {
        assert_eq!(convert_case("hello_world_text", CaseStyle::Title), "Hello World Text");
        assert_eq!(convert_case("kebab-case-slug", CaseStyle::Title), "Kebab Case Slug");
    }

    #[test]
    fn test_slugify() {
        assert_eq!(
            slugify("Hello, World! This is Rust in 2026."),
            "hello-world-this-is-rust-in-2026"
        );
        assert_eq!(slugify("   --Complex String & Symbols--   "), "complex-string-symbols");
        assert_eq!(slugify("already-a-slug"), "already-a-slug");
        assert_eq!(slugify(""), "");
    }

    #[test]
    fn test_reverse_text() {
        assert_eq!(reverse_text("hello"), "olleh");
        assert_eq!(reverse_text("Rust 2026"), "6202 tsuR");
        assert_eq!(reverse_text(""), "");
    }
}
