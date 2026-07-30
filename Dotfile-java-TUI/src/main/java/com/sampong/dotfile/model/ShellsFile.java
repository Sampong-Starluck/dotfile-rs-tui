package com.sampong.dotfile.model;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import java.util.List;

/** Port of Rust {@code models/shell_model.rs::ShellsFile}. */
@JsonIgnoreProperties(ignoreUnknown = true)
public record ShellsFile(List<ShellEntry> shells) {}
