use crate::utils::{decode_winget_output, find_col, split_pkg_name_version, strip_ansi};

/// A single result row from any package manager search.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub name:    String,
    pub id:      String,
    pub version: String,
}

/// Returns (binary, args) for the search command of each manager.
pub fn search_command(mgr: &str, query: &str) -> (String, Vec<String>) {
    match mgr {
        "scoop"        => ("scoop".into(),     vec!["search".into(), query.into()]),
        "choco"        => ("choco".into(),     vec!["search".into(), query.into(), "--limit-output".into()]),
        "pacman"       => ("pacman".into(),    vec!["-Ss".into(), query.into()]),
        "yay"          => ("yay".into(),       vec!["-Ss".into(), query.into()]),
        "paru"         => ("paru".into(),      vec!["-Ss".into(), query.into()]),
        "apt"          => ("apt".into(),       vec!["search".into(), query.into()]),
        "apt-get"      => ("apt-cache".into(), vec!["search".into(), query.into()]),
        "dnf"          => ("dnf".into(),       vec!["search".into(), query.into()]),
        "yum"          => ("yum".into(),       vec!["search".into(), query.into()]),
        "xbps-install" => ("xbps-query".into(), vec!["-Rs".into(), query.into()]),
        "apk"          => ("apk".into(),       vec!["search".into(), query.into()]),
        _              => ("winget".into(),    vec!["search".into(), query.into()]),
    }
}

/// Decode raw stdout — winget may emit UTF-16, everything else is plain UTF-8.
pub fn decode_search_output(mgr: &str, bytes: &[u8]) -> String {
    if mgr == "winget" {
        decode_winget_output(bytes)
    } else {
        String::from_utf8_lossy(bytes).into_owned()
    }
}

/// Dispatch to the correct parser for each manager.
pub fn parse_search_output(mgr: &str, text: &str) -> Vec<SearchResult> {
    match mgr {
        "scoop"                    => parse_scoop_search(text),
        "choco"                    => parse_choco_search(text),
        "pacman" | "yay" | "paru"  => parse_pacman_search(text),
        "apt" | "apt-get"          => parse_apt_search(text),
        "dnf" | "yum"              => parse_dnf_search(text),
        "xbps-install"             => parse_xbps_search(text),
        "apk"                      => parse_apk_search(text),
        _                          => parse_winget_search(text),
    }
}

/// Hint shown in the search input box for each manager.
pub fn search_hint(mgr: &str) -> &'static str {
    match mgr {
        "winget"          => "Press [i] to search winget",
        "scoop"           => "Press [i] to search scoop",
        "choco"           => "Press [i] to search choco",
        "pacman"          => "Press [i] to search pacman",
        "yay"             => "Press [i] to search yay (AUR)",
        "paru"            => "Press [i] to search paru (AUR)",
        "apt" | "apt-get" => "Press [i] to search apt",
        "dnf" | "yum"     => "Press [i] to search dnf",
        "xbps-install"    => "Press [i] to search xbps",
        "apk"             => "Press [i] to search apk",
        _                 => "Press [i] to search packages",
    }
}

// ─── Parsers ─────────────────────────────────────────────────────────────────

/// Parse `winget search` tabular output into structured results.
///
/// Winget output looks like:
/// ```
/// Name             Id                      Version   Source
/// ---------------------------------------------------------------
/// PowerShell       Microsoft.PowerShell    7.4.1     winget
/// ```
/// We locate the header row to find column offsets, then slice each
/// data row at those offsets.
fn parse_winget_search(output: &str) -> Vec<SearchResult> {
    let mut results = Vec::new();
    let mut header_offsets: Option<(usize, usize, usize)> = None;

    // Winget uses bare \r (carriage return) to overwrite its progress spinner
    // in place, so the entire output arrives as one giant "line" when split on
    // \n.  We must split on \r first, then also on \n, deduplicate blanks,
    // and strip ANSI codes from each segment before column detection.
    let segments: Vec<String> = output.split(['\r', '\n'])
        .map(strip_ansi)
        .collect();

    for line in &segments {
        let line = line.as_str();

        // Skip blank lines, pure-control lines, and winget progress bar lines.
        // Progress lines contain block-drawing chars (█ ▒) or KB/MB markers.
        if line.trim().is_empty() { continue; }
        if line.contains('█') || line.contains('▒')
            || line.contains("KB /") || line.contains("MB /") { continue; }
        let trimmed = line.trim();
        // Skip the spinner frames winget emits before the table (-, \, |, /)
        if trimmed == "-" || trimmed == "\\" || trimmed == "|" || trimmed == "/" { continue; }

        // Detect header row
        if header_offsets.is_none() {
            let lower = line.to_lowercase();
            if lower.contains("name") && lower.contains("id") && lower.contains("version") {
                let id_pos  = find_col(line, "Id").or_else(|| find_col(line, "id"));
                let ver_pos = find_col(line, "Version").or_else(|| find_col(line, "version"));
                let src_pos = find_col(line, "Source").or_else(|| find_col(line, "source"));
                if let (Some(id), Some(ver)) = (id_pos, ver_pos) && id > 0 && ver > id {
                    header_offsets = Some((id, ver, src_pos.unwrap_or(usize::MAX)));
                }
            }
            continue;
        }

        // Skip separator lines (all dashes/spaces)
        if line.chars().all(|c| c == '-' || c == ' ') { continue; }

        if let Some((id_start, ver_start, src_start)) = header_offsets {
            let chars: Vec<char> = line.chars().collect();
            let len = chars.len();
            if len < id_start { continue; }

            let name = chars[..id_start].iter().collect::<String>().trim().to_string();
            let id   = chars[id_start..ver_start.min(len)].iter().collect::<String>().trim().to_string();
            let version = if len > ver_start {
                chars[ver_start..src_start.min(len)].iter().collect::<String>().trim().to_string()
            } else {
                String::new()
            };

            if !name.is_empty() && !id.is_empty() {
                tracing::debug!(
                    "[parse_winget] row: name={:?} id={:?} ver={:?}",
                    name,
                    id,
                    version
                );
                results.push(SearchResult { name, id, version });
            }
        }
    }
    results
}

fn parse_scoop_search(output: &str) -> Vec<SearchResult> {
    let mut results = Vec::new();
    let mut header_offsets: Option<(usize, usize)> = None;

    for line in output.lines() {
        let line = line.trim_end_matches('\r');
        if line.trim().is_empty()
            || line.starts_with("Results from")
            || line.chars().all(|c| c == '-' || c == ' ')
        {
            continue;
        }
        if header_offsets.is_none() {
            let lower = line.to_lowercase();
            if lower.contains("name") && lower.contains("version") {
                let ver_pos = find_col(line, "Version").or_else(|| find_col(line, "version"));
                let src_pos = find_col(line, "Source").or_else(|| find_col(line, "source"));
                if let Some(ver) = ver_pos {
                    header_offsets = Some((ver, src_pos.unwrap_or(usize::MAX)));
                }
            }
            continue;
        }
        if let Some((ver_start, src_start)) = header_offsets {
            let chars: Vec<char> = line.chars().collect();
            let len = chars.len();
            if len < ver_start { continue; }
            let name    = chars[..ver_start].iter().collect::<String>().trim().to_string();
            let version = chars[ver_start..src_start.min(len)].iter().collect::<String>().trim().to_string();
            if !name.is_empty() {
                tracing::debug!("[parse_scoop] name={:?} ver={:?}", name, version);
                // scoop id == name (no separate winget-style ID)
                results.push(SearchResult { id: name.clone(), name, version });
            }
        }
    }
    results
}

fn parse_choco_search(output: &str) -> Vec<SearchResult> {
    output
        .lines()
        .filter_map(|line| {
            let line = line.trim().trim_end_matches('\r');
            if line.is_empty() { return None; }
            let mut parts = line.splitn(2, '|');
            let name    = parts.next()?.trim().to_string();
            let version = parts.next().unwrap_or("").trim().to_string();
            if name.is_empty() { return None; }
            tracing::debug!("[parse_choco] name={:?} ver={:?}", name, version);
            Some(SearchResult { id: name.clone(), name, version })
        })
        .collect()
}

fn parse_pacman_search(output: &str) -> Vec<SearchResult> {
    let mut results = Vec::new();
    for line in output.lines() {
        let line = line.trim_end_matches('\r');
        if line.starts_with("::") || line.trim().is_empty()
            || line.starts_with(' ') || line.starts_with('\t') { continue; }
        let mut parts = line.splitn(2, '/');
        let _repo = parts.next().unwrap_or("").trim();
        let rest  = parts.next().unwrap_or(line).trim();
        let mut tokens = rest.split_whitespace();
        let name    = tokens.next().unwrap_or("").to_string();
        let version = tokens.next().unwrap_or("").to_string();
        if !name.is_empty() {
            tracing::debug!("[parse_pacman] name={:?} ver={:?}", name, version);
            results.push(SearchResult { id: name.clone(), name, version });
        }
    }
    results
}

fn parse_apt_search(output: &str) -> Vec<SearchResult> {
    let mut results = Vec::new();
    for line in output.lines() {
        let line = line.trim_end_matches('\r');
        if line.trim().is_empty() || line.starts_with(' ') || line.starts_with('\t')
            || line.starts_with("Sorting") || line.starts_with("Full Text")
            || line.starts_with("WARNING") { continue; }
        let mut parts = line.splitn(2, '/');
        let name    = parts.next().unwrap_or("").trim().to_string();
        let version = parts.next().unwrap_or("").split_whitespace().next().unwrap_or("").to_string();
        if !name.is_empty() {
            tracing::debug!("[parse_apt] name={:?} ver={:?}", name, version);
            results.push(SearchResult { id: name.clone(), name, version });
        }
    }
    results
}

fn parse_dnf_search(output: &str) -> Vec<SearchResult> {
    let mut results = Vec::new();
    for line in output.lines() {
        let line = line.trim_end_matches('\r');
        if line.trim().is_empty() || line.starts_with("Last metadata")
            || line.starts_with("=====") || line.starts_with("Error") { continue; }
        if !line.contains(".x86_64") && !line.contains(".noarch") && !line.contains(".src") { continue; }
        let mut tokens = line.split_whitespace();
        let full_name = tokens.next().unwrap_or("").to_string();
        let version   = tokens.next().unwrap_or("").to_string();
        let name = full_name.split('.').next().unwrap_or(&full_name).to_string();
        if !name.is_empty() {
            tracing::debug!("[parse_dnf] name={:?} ver={:?}", name, version);
            results.push(SearchResult { id: name.clone(), name, version });
        }
    }
    results
}

fn parse_xbps_search(output: &str) -> Vec<SearchResult> {
    output
        .lines()
        .filter_map(|line| {
            let line = line.trim_end_matches('\r').trim();
            if line.is_empty() { return None; }
            let rest = if line.starts_with("[-]") || line.starts_with("[*]") {
                line[3..].trim()
            } else { line };
            let pkg = rest.split_whitespace().next().unwrap_or("");
            let (name, version) = split_pkg_name_version(pkg);
            if name.is_empty() { return None; }
            tracing::debug!("[parse_xbps] name={:?} ver={:?}", name, version);
            Some(SearchResult { id: name.clone(), name, version })
        })
        .collect()
}

fn parse_apk_search(output: &str) -> Vec<SearchResult> {
    output
        .lines()
        .filter_map(|line| {
            let line = line.trim_end_matches('\r').trim();
            if line.is_empty() { return None; }
            let pkg = line.split_whitespace().next().unwrap_or("");
            let (name, version) = split_pkg_name_version(pkg);
            if name.is_empty() { return None; }
            tracing::debug!("[parse_apk] name={:?} ver={:?}", name, version);
            Some(SearchResult { id: name.clone(), name, version })
        })
        .collect()
}
