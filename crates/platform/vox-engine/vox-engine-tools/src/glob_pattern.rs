//! Glob pattern matching: `*` and `?` inside a path segment,
//! `**` spanning any number of directories.

/// Match a `/`-separated path against a segment pattern.
///
/// `**` matches any number of segments (including none), every
/// other segment matches with `*` and `?` inside the segment only.
pub(crate) fn matches(pattern: &str, path: &str) -> bool {
    let mut pattern: Vec<&str> = pattern.split('/').collect();
    let mut path: Vec<&str> = path.split('/').collect();
    while !pattern.is_empty() {
        if pattern[0] == "**" {
            pattern.remove(0);
            if pattern.is_empty() {
                return true;
            }
            for split in 0..=path.len() {
                if matches(&pattern.join("/"), &path[split..].join("/")) {
                    return true;
                }
            }
            return false;
        }
        if path.is_empty() || !segment(pattern[0], path[0]) {
            return false;
        }
        pattern.remove(0);
        path.remove(0);
    }
    path.is_empty()
}

/// Match one path segment with `*` and `?` wildcards.
fn segment(pattern: &str, text: &str) -> bool {
    let pattern: Vec<char> = pattern.chars().collect();
    let text: Vec<char> = text.chars().collect();
    let (mut p, mut t) = (0, 0);
    let mut star: Option<(usize, usize)> = None;
    while t < text.len() {
        match p < pattern.len() {
            true if pattern[p] == '*' => {
                star = Some((p, t));
                p += 1;
            }
            true if pattern[p] == '?' || pattern[p] == text[t] => {
                p += 1;
                t += 1;
            }
            _ => match star {
                Some((mark_p, mark_t)) => {
                    p = mark_p + 1;
                    t = mark_t + 1;
                    star = Some((mark_p, mark_t + 1));
                }
                None => return false,
            },
        }
    }
    pattern[p..].iter().all(|c| *c == '*')
}
