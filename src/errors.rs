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
            detail: indent(raw),
            suggestion: String::new(),
        }
    }
}

pub fn translate_nix_error(raw: &str) -> ErrorMessage {
    let patterns: [(&str, &str, &str); 7] = [
        (
            r"undefined variable 'pkgs\.([^']+)'",
            "'{}' wasn't found in nixpkgs.",
            "try: nina search {}",
        ),
        (
            r"attribute '([^']+)' missing",
            "'{}' doesn't exist in this version of nixpkgs.",
            "it may have moved or been removed.",
        ),
        (
            r"collision between",
            "two packages are claiming the same file.",
            "remove one and try again.",
        ),
        (
            r"infinite recursion",
            "there's a circular reference in your config.",
            "check recent edits to configuration.nix.",
        ),
        (
            r"No space left on device",
            "disk is full.",
            "run: nina clean",
        ),
        (
            r"hash mismatch in fixed-output derivation",
            "download checksum mismatch — usually temporary.",
            "try again.",
        ),
        (
            r"cannot write lock file",
            "can't write flake.lock.",
            "run with: --no-write-lock-file",
        ),
    ];

    for (pattern, summary, suggestion) in patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(caps) = re.captures(raw) {
                let cap1 = caps.get(1).map_or("".into(), |m| m.as_str().to_string());
                let summary_out = summary.replace("{}", &cap1);
                let suggestion_out = suggestion.replace("{}", &cap1);
                return ErrorMessage {
                    summary: summary_out,
                    detail: indent(raw),
                    suggestion: suggestion_out,
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
