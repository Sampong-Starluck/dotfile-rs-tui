/// Decode winget's stdout bytes to a String.
/// Handles UTF-16 LE BOM, UTF-8 BOM, and plain UTF-8.
pub fn decode_winget_output(bytes: &[u8]) -> String {
    if bytes.starts_with(&[0xFF, 0xFE]) {
        let words: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
            .collect();
        return String::from_utf16_lossy(&words);
    }
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return String::from_utf8_lossy(&bytes[3..]).into_owned();
    }
    String::from_utf8_lossy(bytes).into_owned()
}

/// Returns true for lines that should be hidden in the install log.
///
/// Winget uses bare \r to animate a progress spinner and download bar in
/// place. When stdout is piped those \r sequences arrive inside a single
/// BufReader "line". We split on \r here and filter out the noise segments
/// so only meaningful status lines reach the UI.
pub fn is_noise_line(s: &str) -> bool {
    let t = s.trim();
    if t.is_empty() { return true; }
    if t == "-" || t == "\\" || t == "|" || t == "/" { return true; }
    if t.contains('█') || t.contains('▒') { return true; }
    if t.contains("KB /") || t.contains("MB /") || t.contains("GB /") { return true; }
    if t.ends_with('%') && t.trim_end_matches('%').trim().chars().all(|c| c.is_ascii_digit()) {
        return true;
    }
    false
}