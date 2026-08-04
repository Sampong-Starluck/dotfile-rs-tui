package com.sampong.dotfile.model;

import java.util.List;

/** Port of Rust {@code models/apps.rs::AppSection}. {@code apps.json} root is a JSON array of these. */
public record AppSection(String section, List<AppEntry> apps) {}
