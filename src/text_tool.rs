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
        assert_eq!(convert_case("kebab-case-text", CaseStyle::Snake), "kebab_case_text");
    }

    #[test]
    fn test_convert_case_kebab() {
        assert_eq!(convert_case("Hello_World", CaseStyle::Kebab), "hello-world");
    }

    #[test]
    fn test_convert_case_camel() {
        assert_eq!(convert_case("hello_world_text", CaseStyle::Camel), "helloWorldText");
    }

    #[test]
    fn test_convert_case_upper() {
        assert_eq!(convert_case("hello world", CaseStyle::Upper), "HELLO_WORLD");
    }
}
