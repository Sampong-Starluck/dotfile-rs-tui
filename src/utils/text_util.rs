/// Remove ANSI escape sequences from a string.
/// Used when piped output contains cursor/erase sequences.
pub fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next();
                for sc in chars.by_ref() {
                    if sc.is_ascii_alphabetic() { break; }
                }
            } else {
                chars.next();
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Strip trailing `\r` from a line — needed on Windows piped output.
pub fn sanitize_line(line: String) -> String {
    line.trim_end_matches('\r').to_string()
}

/// Find the char-index of the first occurrence of `needle` in `haystack`.
/// Used for column offset detection in tabular CLI output (winget, scoop).
pub fn find_col(haystack: &str, needle: &str) -> Option<usize> {
    let h: Vec<char> = haystack.chars().collect();
    let n: Vec<char> = needle.chars().collect();
    let n_len = n.len();
    if n_len == 0 || h.len() < n_len { return None; }
    for i in 0..=(h.len() - n_len) {
        if h[i..i + n_len] == n[..] { return Some(i); }
    }
    None
}

/// Split "neovim-0.9.5_1" → ("neovim", "0.9.5_1").
/// The version starts at the last hyphen followed by a digit.
/// Used by xbps and apk parsers.
pub fn split_pkg_name_version(pkg: &str) -> (String, String) {
    let chars: Vec<char> = pkg.chars().collect();
    for i in (1..chars.len()).rev() {
        if chars[i - 1] == '-' && chars[i].is_ascii_digit() {
            return (
                chars[..i - 1].iter().collect(),
                chars[i..].iter().collect(),
            );
        }
    }
    (pkg.to_string(), String::new())
}