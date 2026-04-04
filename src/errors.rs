use regex::Regex;

#[derive(Debug, Clone)]
pub struct ErrorMessage {
    pub summary: String,
    pub detail: String,
    pub suggestion: String,
}

impl ErrorMessage {
    pub fn fallback(raw: &str) -> Self {
        Self {
            summary: "something went wrong.".to_string(),
            detail: format!("nix said:\n{}", indent(raw)),
            suggestion: "if this looks confusing, try: nina doctor ♡".to_string(),
        }
    }
}

pub fn translate_nix_error(raw: &str) -> ErrorMessage {
    let patterns: [(&str, &str, &str); 8] = [
        (
            r"undefined variable '([^']+)'",
            "that package name wasn't found.",
            "try: nina search <name>",
        ),
        (
            r"attribute '([^']+)' missing",
            "something in your config references a missing attribute.",
            "double-check package names or search with: nina search <name>",
        ),
        (
            r"collision between",
            "two packages are trying to install the same file.",
            "remove one of the conflicting packages, then apply again.",
        ),
        (
            r"infinite recursion",
            "your config has a circular reference.",
            "review recent edits around let/in and imports.",
        ),
        (
            r"SSL peer certificate.*was not ok",
            "there was a network or certificate issue.",
            "check internet connectivity, then retry.",
        ),
        (
            r"No space left on device",
            "your disk is full.",
            "try: nina clean",
        ),
        (
            r"hash mismatch in fixed-output derivation",
            "a fetched source checksum did not match.",
            "this is often temporary, try again in a moment.",
        ),
        (
            r"renamed to '([^']+)'",
            "a package has been renamed.",
            "update your config to the new package name.",
        ),
    ];

    for (pattern, summary, suggestion) in patterns {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(raw) {
                return ErrorMessage {
                    summary: summary.to_string(),
                    detail: indent(raw),
                    suggestion: suggestion.to_string(),
                };
            }
        }
    }

    ErrorMessage::fallback(raw)
}

fn indent(raw: &str) -> String {
    raw.lines()
        .map(|line| format!("  {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}
