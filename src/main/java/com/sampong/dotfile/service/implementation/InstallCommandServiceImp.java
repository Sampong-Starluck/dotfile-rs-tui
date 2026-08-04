package com.sampong.dotfile.service.implementation;

import com.sampong.dotfile.model.AppEntry;
import com.sampong.dotfile.service.InstallCommandService;
import com.sampong.dotfile.service.OsService;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import java.util.List;
import java.util.Map;
import java.util.Optional;

/** Port of Rust {@code service/install_service.rs}. */
@Slf4j
@Service
@RequiredArgsConstructor
public class InstallCommandServiceImp implements InstallCommandService {

    private final OsService os;

    @Override
    public Optional<String> installCommandFor(AppEntry entry, String detectedMgr) {
        Map<String, String> platform = entry.platforms().get(os.osKey());
        if (platform == null) return Optional.empty();

        String mgr = detectedMgr;
        if (!platform.containsKey(mgr)) {
            List<String> fallbacks = switch (os.osKey()) {
                case "windows" -> List.of("winget", "scoop", "choco");
                case "macos" -> List.of("brew");
                default -> List.of("pacman", "yay", "apt", "dnf", "xbps-install");
            };
            mgr = fallbacks.stream().filter(platform::containsKey).findFirst().orElse(null);
            if (mgr == null) return Optional.empty();
        }
        return Optional.of(installCommand(mgr, platform.get(mgr)));
    }

    @Override
    public String installCommand(String mgr, String pkg) {
        return switch (mgr) {
            case "pacman" -> "pacman -S " + pkg;
            case "yay" -> "yay -S " + pkg;
            case "paru" -> "paru -S " + pkg;
            case "apt" -> "apt install -y " + pkg;
            case "apt-get" -> "apt-get install -y " + pkg;
            case "dnf" -> "dnf install -y " + pkg;
            case "yum" -> "yum install -y " + pkg;
            case "xbps-install" -> "xbps-install -S " + pkg;
            case "apk" -> "apk add " + pkg;
            case "brew" -> "brew install " + pkg;
            case "winget" -> "winget install --id " + pkg + " -e";
            case "scoop" -> "scoop install " + pkg;
            case "choco" -> "choco install " + pkg + " -y";
            default -> mgr + " install " + pkg;
        };
    }

    @Override
    public String removeCommand(String mgr, String pkg) {
        return switch (mgr) {
            case "pacman" -> "pacman -R " + pkg;
            case "yay" -> "yay -R " + pkg;
            case "paru" -> "paru -R " + pkg;
            case "apt" -> "apt remove -y " + pkg;
            case "apt-get" -> "apt-get remove -y " + pkg;
            case "dnf" -> "dnf remove -y " + pkg;
            case "yum" -> "yum remove -y " + pkg;
            case "xbps-install" -> "xbps-remove " + pkg;
            case "apk" -> "apk del " + pkg;
            case "brew" -> "brew uninstall " + pkg;
            case "winget" -> "winget uninstall --id " + pkg + " -e";
            case "scoop" -> "scoop uninstall " + pkg;
            case "choco" -> "choco uninstall " + pkg + " -y";
            default -> mgr + " remove " + pkg;
        };
    }

    @Override
    public String updateCommand(String mgr, String pkg) {
        return switch (mgr) {
            case "pacman" -> "pacman -S " + pkg;
            case "yay" -> "yay -S " + pkg;
            case "paru" -> "paru -S " + pkg;
            case "apt" -> "apt install --only-upgrade -y " + pkg;
            case "apt-get" -> "apt-get install --only-upgrade -y " + pkg;
            case "dnf" -> "dnf upgrade -y " + pkg;
            case "yum" -> "yum update -y " + pkg;
            case "xbps-install" -> "xbps-install -u " + pkg;
            case "apk" -> "apk upgrade " + pkg;
            case "brew" -> "brew upgrade " + pkg;
            case "winget" -> "winget upgrade --id " + pkg + " -e";
            case "scoop" -> "scoop update " + pkg;
            case "choco" -> "choco upgrade " + pkg + " -y";
            default -> mgr + " upgrade " + pkg;
        };
    }
}
