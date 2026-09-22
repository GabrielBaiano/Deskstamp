use chrono::Local;

pub fn resolve_tokens(template: &str) -> String {
    let now = Local::now();
    let user = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
    let host = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "localhost".to_string());

    let mut result = template
        .replace("{user}", &user)
        .replace("{hostname}", &host);

    // Fast resolution for simple tokens
    result = result.replace("{date}", &now.format("%Y-%m-%d").to_string());
    result = result.replace("{time}", &now.format("%H:%M:%S").to_string());

    // Dynamic custom format tokens: {date:FORMAT} or {time:FORMAT}
    while let Some(start) = result.find("{date:") {
        if let Some(end) = result[start..].find('}') {
            let full_match = &result[start..start + end + 1];
            let fmt = &full_match[6..full_match.len() - 1];
            let formatted = now.format(fmt).to_string();
            result = result.replace(full_match, &formatted);
        } else {
            break;
        }
    }

    while let Some(start) = result.find("{time:") {
        if let Some(end) = result[start..].find('}') {
            let full_match = &result[start..start + end + 1];
            let fmt = &full_match[6..full_match.len() - 1];
            let formatted = now.format(fmt).to_string();
            result = result.replace(full_match, &formatted);
        } else {
            break;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_tokens() {
        let res = resolve_tokens("Hello {user} at {date:%Y}");
        assert!(!res.contains("{user}"));
        assert!(!res.contains("{date:"));
    }
}
