use once_cell::sync::Lazy;
use regex::Regex;

struct KeywordGroup {
    pattern: Regex,
    color: &'static str,
}

const RESET: &str = "\x1b[0m";

const COLOR_STRING: &str = "\x1b[38;2;255;220;120m"; // yellow
const COLOR_KEYWORD: &str = "\x1b[38;2;220;120;80m"; // orange
const COLOR_TYPE: &str = "\x1b[38;2;100;180;240m"; // light blue
const COLOR_CONSTANT: &str = "\x1b[38;2;240;180;60m"; // gold
const COLOR_CONTROL: &str = "\x1b[38;2;200;120;200m"; // purple

static GROUPS: Lazy<Vec<KeywordGroup>> = Lazy::new(|| {
    vec![
        KeywordGroup {
            pattern: Regex::new(r#""([^"\\]|\\.)*""#).unwrap(),
            color: COLOR_STRING,
        },
        KeywordGroup {
            pattern: Regex::new(
                r"\b(let|fn|impl|trait|struct|enum|use|mod|pub|unsafe|async|await)\b",
            )
            .unwrap(),
            color: COLOR_KEYWORD,
        },
        KeywordGroup {
            pattern: Regex::new(
                r"\b(int|bool|string|char|float|double|void|i32|u32|f64|str|Vec|Option|Result)\b",
            )
            .unwrap(),
            color: COLOR_TYPE,
        },
        KeywordGroup {
            pattern: Regex::new(r"\b(if|else|for|while|loop|match|return|break|continue)\b")
                .unwrap(),
            color: COLOR_CONTROL,
        },
        KeywordGroup {
            pattern: Regex::new(r"\b(true|false|null|None|Some|Ok|Err)\b").unwrap(),
            color: COLOR_CONSTANT,
        },
    ]
});

pub fn highlight_line(line: &str) -> String {
    let mut result = line.to_string();
    for group in GROUPS.iter() {
        result = group
            .pattern
            .replace_all(&result, |caps: &regex::Captures| {
                format!("{}{}{}", group.color, &caps[0], RESET)
            })
            .to_string();
    }
    result
}
