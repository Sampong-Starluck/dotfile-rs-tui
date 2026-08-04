package com.sampong.dotfile.model;

import java.util.Map;

/** Port of Rust {@code models/apps.rs::AppEntry}. */
public record AppEntry(
        String name,
        String id,
        Map<String, Map<String, String>> platforms) {}  // platforms.get("windows").get("winget") -> pkg id
