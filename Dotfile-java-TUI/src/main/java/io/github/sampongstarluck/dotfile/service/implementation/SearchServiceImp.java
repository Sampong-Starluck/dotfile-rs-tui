package io.github.sampongstarluck.dotfile.service.implementation;

import io.github.sampongstarluck.dotfile.service.SearchService;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import java.util.List;
import java.util.Set;

/** Command tables ported from Rust {@code service/search_service.rs}; parsers come in Phase 4. */
@Slf4j
@Service
@RequiredArgsConstructor
public class SearchServiceImp implements SearchService {

    private static final Set<String> PACMAN_LIKE = Set.of("pacman", "yay", "paru");
    private static final Set<String> APT_LIKE = Set.of("apt", "apt-get");
    private static final Set<String> DNF_LIKE = Set.of("dnf", "yum");

    @Override
    public Cmd searchCommand(String mgr, String query) {
        return switch (mgr) {
            case "scoop" -> new Cmd("scoop", List.of("search", query));
            case "choco" -> new Cmd("choco", List.of("search", query, "--limit-output"));
            case "pacman" -> new Cmd("pacman", List.of("-Ss", query));
            case "yay" -> new Cmd("yay", List.of("-Ss", query));
            case "paru" -> new Cmd("paru", List.of("-Ss", query));
            case "apt" -> new Cmd("apt", List.of("search", query));
            case "apt-get" -> new Cmd("apt-cache", List.of("search", query));
            case "dnf" -> new Cmd("dnf", List.of("search", query));
            case "yum" -> new Cmd("yum", List.of("search", query));
            case "xbps-install" -> new Cmd("xbps-query", List.of("-Rs", query));
            case "apk" -> new Cmd("apk", List.of("search", query));
            default -> new Cmd("winget", List.of("search", query));
        };
    }

    @Override
    public Cmd listCommand(String mgr) {
        if (PACMAN_LIKE.contains(mgr)) return new Cmd(mgr, List.of("-Q"));
        if (APT_LIKE.contains(mgr)) return new Cmd("apt", List.of("list", "--installed"));
        if (DNF_LIKE.contains(mgr)) return new Cmd(mgr, List.of("list", "installed"));
        return switch (mgr) {
            case "xbps-install" -> new Cmd("xbps-query", List.of("-l"));
            case "apk" -> new Cmd("apk", List.of("list", "--installed"));
            case "brew" -> new Cmd("brew", List.of("list", "--formula"));
            case "winget" -> new Cmd("winget", List.of("list"));
            case "scoop" -> new Cmd("scoop", List.of("list"));
            case "choco" -> new Cmd("choco", List.of("list", "--local-only"));
            default -> new Cmd(mgr, List.of("list"));
        };
    }

    @Override
    public String searchHint(String mgr) {
        return switch (mgr) {
            case "winget" -> "Press [i] to search winget";
            case "scoop" -> "Press [i] to search scoop";
            case "choco" -> "Press [i] to search choco";
            case "pacman" -> "Press [i] to search pacman";
            case "yay" -> "Press [i] to search yay (AUR)";
            case "paru" -> "Press [i] to search paru (AUR)";
            case "apt", "apt-get" -> "Press [i] to search apt";
            case "dnf", "yum" -> "Press [i] to search dnf";
            case "xbps-install" -> "Press [i] to search xbps";
            case "apk" -> "Press [i] to search apk";
            default -> "Press [i] to search packages";
        };
    }
}
