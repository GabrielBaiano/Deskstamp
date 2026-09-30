use chrono::Local;

pub fn resolve_tokens(template: &str) -> String {
    resolve_tokens_with_context(template, None, 1, None)
}

pub fn resolve_tokens_with_workspace(template: &str, workspace: Option<&str>) -> String {
    resolve_tokens_with_context(template, workspace, 1, None)
}

pub fn resolve_tokens_with_context(
    template: &str,
    workspace: Option<&str>,
    screen_idx: usize,
    screen_name: Option<&str>,
) -> String {
    let now = Local::now();
    let user = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
    let host = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "localhost".to_string());
    let ws = workspace.unwrap_or("1");

    let screen_str = screen_idx.to_string();
    let screen_name_str = screen_name
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("Display {}", screen_idx));

    let mut result = template
        .replace("{user}", &user)
        .replace("{hostname}", &host)
        .replace("{workspace}", ws)
        .replace("{workspace_num}", ws)
        .replace("{workspace_name}", ws)
        .replace("{screen}", &screen_str)
        .replace("{display}", &screen_str)
        .replace("{screen_name}", &screen_name_str);

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

    #[test]
    fn test_resolve_workspace_token() {
        let res = resolve_tokens_with_workspace("Workspace: {workspace}", Some("Dev"));
        assert_eq!(res, "Workspace: Dev");

        let res_default = resolve_tokens("Workspace: {workspace}");
        assert_eq!(res_default, "Workspace: 1");
    }

    #[test]
    fn test_resolve_screen_token() {
        let res = resolve_tokens_with_context("Tela {screen} ({screen_name}) - Área {workspace}", Some("2"), 2, Some("DP-1"));
        assert_eq!(res, "Tela 2 (DP-1) - Área 2");
    }
}
