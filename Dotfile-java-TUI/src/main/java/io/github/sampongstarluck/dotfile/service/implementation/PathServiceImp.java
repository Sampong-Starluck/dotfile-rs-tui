package io.github.sampongstarluck.dotfile.service.implementation;

import io.github.sampongstarluck.dotfile.service.PathService;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import java.io.File;
import java.nio.file.Files;
import java.nio.file.InvalidPathException;
import java.nio.file.LinkOption;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import java.util.Optional;

/** Java has no {@code which} crate; port it once, including Windows PATHEXT handling. */
@Slf4j
@Service
@RequiredArgsConstructor
public class PathServiceImp implements PathService {

    @Override
    public Optional<Path> which(String binary) {
        String pathEnv = System.getenv("PATH");
        if (pathEnv == null) return Optional.empty();

        boolean windows = System.getProperty("os.name", "").toLowerCase(Locale.ROOT).contains("win");
        List<String> extensions = new ArrayList<>();
        extensions.add("");
        if (windows) {
            String pathext = System.getenv("PATHEXT");
            if (pathext == null) pathext = ".COM;.EXE;.BAT;.CMD;.PS1";
            for (String ext : pathext.split(";")) {
                if (!ext.isBlank()) extensions.add(ext.toLowerCase(Locale.ROOT));
            }
        }

        for (String dir : pathEnv.split(File.pathSeparator)) {
            if (dir.isBlank()) continue;
            for (String ext : extensions) {
                Path candidate;
                try {
                    candidate = Path.of(dir, binary + ext);
                } catch (InvalidPathException e) {
                    log.debug("Skipping unparseable PATH entry: {}", dir, e);
                    continue;
                }
                if (isExecutableCandidate(candidate, windows)) {
                    return Optional.of(candidate);
                }
            }
        }
        return Optional.empty();
    }

    /**
     * Windows App Execution Alias stubs (e.g. winget.exe under WindowsApps) are
     * reparse points {@code Files.isRegularFile} cannot stat through, so on
     * Windows we check existence without following the reparse point instead.
     */
    private boolean isExecutableCandidate(Path candidate, boolean windows) {
        if (windows) {
            return Files.exists(candidate, LinkOption.NOFOLLOW_LINKS)
                    && !Files.isDirectory(candidate, LinkOption.NOFOLLOW_LINKS);
        }
        return Files.isRegularFile(candidate) && Files.isExecutable(candidate);
    }

    @Override
    public boolean isOnPath(String binary) {
        return which(binary).isPresent();
    }
}
