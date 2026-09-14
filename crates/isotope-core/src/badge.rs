#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Badge {
    Count(u32),
    Dot,
}

impl Badge {
    pub fn label(self) -> String {
        match self {
            Badge::Count(n) if n > 99 => "99+".into(),
            Badge::Count(n) => n.to_string(),
            Badge::Dot => "•".into(),
        }
    }
}

/// Extracts an unread indicator from a page title (spec §4.5).
pub fn parse_badge(title: &str) -> Option<Badge> {
    let title = title.trim();
    if title.starts_with('•') || title.starts_with('●') {
        return Some(Badge::Dot);
    }
    let count = leading_count(title).or_else(|| trailing_count(title))?;
    (count > 0).then_some(Badge::Count(count))
}

fn leading_count(title: &str) -> Option<u32> {
    let close = match title.chars().next()? {
        '(' => ')',
        '[' => ']',
        _ => return None,
    };
    let end = title.find(close)?;
    parse_number(&title[1..end])
}

fn trailing_count(title: &str) -> Option<u32> {
    let inner = title.strip_suffix(')')?;
    let start = inner.rfind('(')?;
    parse_number(&inner[start + 1..])
}

fn parse_number(text: &str) -> Option<u32> {
    let text = text.trim();
    let digits = text.strip_suffix('+').unwrap_or(text);
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some(digits.parse::<u32>().unwrap_or(u32::MAX))
}
