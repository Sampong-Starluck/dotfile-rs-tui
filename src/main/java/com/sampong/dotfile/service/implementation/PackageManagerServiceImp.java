package com.sampong.dotfile.service.implementation;

import com.sampong.dotfile.model.OperatingSystem;
import com.sampong.dotfile.model.PackageManager;
import com.sampong.dotfile.service.OsService;
import com.sampong.dotfile.service.PackageManagerService;
import com.sampong.dotfile.service.PathService;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import java.util.List;

import static com.sampong.dotfile.model.PackageManager.*;

/** Detection half of Rust {@code models/package_manager.rs}. */
@Slf4j
@Service
@RequiredArgsConstructor
public class PackageManagerServiceImp implements PackageManagerService {

    private final OsService os;
    private final PathService path;

    @Override
    public List<PackageManager> detect() {
        return candidatesFor(os.detect()).stream()
                .filter(pm -> path.isOnPath(pm.binary()))
                .toList();
    }

    @Override
    public List<PackageManager> candidatesFor(OperatingSystem o) {
        return switch (o.kind()) {
            case WINDOWS -> List.of(WINGET, SCOOP, CHOCO);
            case MACOS -> List.of(BREW);
            case LINUX -> {
                if (o.distro() == null) yield List.of(APT, DNF, PACMAN, YAY, XBPS);
                yield switch (o.distro()) {
                    case ARCH -> List.of(PACMAN, YAY, PARU);
                    case DEBIAN -> List.of(APT);
                    case FEDORA -> List.of(DNF);
                    case VOID -> List.of(XBPS);
                    case OTHER -> List.of(APT, DNF, PACMAN, YAY, XBPS);
                };
            }
            case UNKNOWN -> List.of();
        };
    }
}
