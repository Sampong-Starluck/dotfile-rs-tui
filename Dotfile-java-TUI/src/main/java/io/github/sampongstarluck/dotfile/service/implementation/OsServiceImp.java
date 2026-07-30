package io.github.sampongstarluck.dotfile.service.implementation;

import io.github.sampongstarluck.dotfile.model.LinuxDistro;
import io.github.sampongstarluck.dotfile.model.OperatingSystem;
import io.github.sampongstarluck.dotfile.service.OsService;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Locale;

/** Port of Rust {@code models/os.rs}. */
@Slf4j
@Service
@RequiredArgsConstructor
public class OsServiceImp implements OsService {

    private record DistroResult(LinuxDistro distro, String name) {}

    @Override
    public boolean isWindows() { return osName().contains("win"); }

    @Override
    public boolean isMac() { return osName().contains("mac"); }

    @Override
    public boolean isLinux() { return osName().contains("linux") || osName().contains("nix"); }

    private String osName() {
        return System.getProperty("os.name", "").toLowerCase(Locale.ROOT);
    }

    @Override
    public String osKey() {
        if (isWindows()) return "windows";
        if (isMac()) return "macos";
        return "linux";
    }

    @Override
    public OperatingSystem detect() {
        if (isWindows()) return new OperatingSystem(OperatingSystem.Kind.WINDOWS, null, null);
        if (isMac()) return new OperatingSystem(OperatingSystem.Kind.MACOS, null, null);
        if (isLinux()) {
            DistroResult d = detectDistro();
            return new OperatingSystem(OperatingSystem.Kind.LINUX, d.distro(), d.name());
        }
        return new OperatingSystem(OperatingSystem.Kind.UNKNOWN, null, null);
    }

    /** Port of LinuxDistro::detect() — read /etc/os-release, check ID= then ID_LIKE=. */
    private DistroResult detectDistro() {
        Path osRelease = Path.of("/etc/os-release");
        if (!Files.exists(osRelease)) return new DistroResult(null, null);

        String id = null;
        String idLike = null;
        try {
            for (String line : Files.readAllLines(osRelease)) {
                if (id == null && line.startsWith("ID=")) {
                    id = stripQuotes(line.substring("ID=".length())).toLowerCase(Locale.ROOT);
                } else if (idLike == null && line.startsWith("ID_LIKE=")) {
                    idLike = stripQuotes(line.substring("ID_LIKE=".length())).toLowerCase(Locale.ROOT);
                }
            }
        } catch (IOException e) {
            log.debug("Failed to read /etc/os-release", e);
            return new DistroResult(null, null);
        }

        String combined = (id == null ? "" : id) + " " + (idLike == null ? "" : idLike);
        if (combined.contains("arch")) return new DistroResult(LinuxDistro.ARCH, id);
        if (combined.contains("fedora") || combined.contains("rhel")) return new DistroResult(LinuxDistro.FEDORA, id);
        if (combined.contains("debian") || combined.contains("ubuntu")) return new DistroResult(LinuxDistro.DEBIAN, id);
        if (combined.contains("void")) return new DistroResult(LinuxDistro.VOID, id);
        return id == null ? new DistroResult(null, null) : new DistroResult(LinuxDistro.OTHER, id);
    }

    private static String stripQuotes(String s) {
        String t = s.trim();
        if (t.length() >= 2 && t.charAt(0) == '"' && t.charAt(t.length() - 1) == '"') {
            return t.substring(1, t.length() - 1);
        }
        return t;
    }
}
