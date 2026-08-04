package com.sampong.dotfile.service.implementation;

import com.sampong.dotfile.service.SystemService;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import java.util.Locale;
import java.util.Set;

/** Port of Rust {@code service/system_service.rs}. */
@Slf4j
@Service
@RequiredArgsConstructor
public class SystemServiceImp implements SystemService {

    private static final Set<String> SUDO_MANAGERS =
            Set.of("pacman", "apt", "apt-get", "dnf", "yum", "xbps-install", "apk");

    private static final Set<String> INTERACTIVE_MANAGERS =
            Set.of("pacman", "apt", "apt-get", "dnf", "yum", "xbps-install", "apk", "yay", "paru", "choco");

    @Override
    public boolean isRoot() {
        if (System.getProperty("os.name", "").toLowerCase(Locale.ROOT).contains("win")) return false;
        return "root".equals(System.getProperty("user.name"));
    }

    @Override
    public boolean requiresSudo(String mgr) {
        return SUDO_MANAGERS.contains(mgr);
    }

    @Override
    public boolean requiresInteractive(String mgr) {
        return INTERACTIVE_MANAGERS.contains(mgr);
    }
}
