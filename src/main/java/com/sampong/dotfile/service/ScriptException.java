package com.sampong.dotfile.service;

/** Checked, mirroring the Rust {@code Result<_, String>} — the UI shows {@link #getMessage()}. */
public class ScriptException extends Exception {
    public ScriptException(String message) {
        super(message);
    }
}
