package io.github.sampongstarluck.dotfile.service;

import io.github.sampongstarluck.dotfile.model.AppEntry;
import io.github.sampongstarluck.dotfile.model.AppSection;
import io.github.sampongstarluck.dotfile.model.ShellEntry;
import io.github.sampongstarluck.dotfile.service.implementation.AppCatalogServiceImp;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import tools.jackson.databind.ObjectMapper;

import java.util.List;
import java.util.Optional;

import static org.assertj.core.api.Assertions.assertThat;

class AppCatalogServiceTest {

    private AppCatalogService service;

    @BeforeEach
    void setUp() {
        service = new AppCatalogServiceImp(new ObjectMapper());
    }

    @Test
    void readAppsJsonLoadsTheCatalog() {
        List<AppSection> apps = service.readAppsJson();

        assertThat(apps).isNotEmpty();
        assertThat(apps.getFirst().section()).isEqualTo("Terminal and Shells");

        AppEntry powershell = findEntry(apps, "PowerShell 7");
        assertThat(powershell.platforms().get("windows").get("winget")).isEqualTo("Microsoft.PowerShell");
    }

    @Test
    void filterByPlatformDropsEntriesMissingTheManager() {
        List<AppSection> apps = service.readAppsJson();
        List<AppSection> filtered = service.filterByPlatform(apps, "windows", "winget");

        assertThat(findEntryOptional(filtered, "Zsh")).isEmpty();
        assertThat(findEntryOptional(filtered, "PowerShell 7")).isPresent();
    }

    @Test
    void readShellsJsonReturnsKnownShellsSortedByOrder() {
        List<ShellEntry> shells = service.readShellsJson();

        assertThat(shells).hasSize(5);
        assertThat(shells.getFirst().id()).isEqualTo("powershell");
        assertThat(shells).isSortedAccordingTo((a, b) -> Integer.compare(a.order(), b.order()));
    }

    private static AppEntry findEntry(List<AppSection> apps, String name) {
        return findEntryOptional(apps, name)
                .orElseThrow(() -> new AssertionError("no app named " + name));
    }

    private static Optional<AppEntry> findEntryOptional(List<AppSection> apps, String name) {
        return apps.stream()
                .flatMap(s -> s.apps().stream())
                .filter(e -> e.name().equals(name))
                .findFirst();
    }
}
