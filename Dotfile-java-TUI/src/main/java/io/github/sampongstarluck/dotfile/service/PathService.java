package io.github.sampongstarluck.dotfile.service;

import java.nio.file.Path;
import java.util.Optional;

public interface PathService {
    /** Returns the full path of an executable on PATH, or empty. Port of {@code which::which()}. */
    Optional<Path> which(String binary);

    boolean isOnPath(String binary);
}
