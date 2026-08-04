package com.sampong.dotfile.service;

import com.sampong.dotfile.model.AppEntry;
import com.sampong.dotfile.model.AppSection;
import com.sampong.dotfile.service.implementation.CommandPlannerImp;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.Set;

import static org.assertj.core.api.Assertions.assertThat;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

class CommandPlannerTest {

    private InstallCommandService installCommandService;
    private CommandPlanner planner;
    private List<AppSection> apps;

    @BeforeEach
    void setUp() {
        installCommandService = mock(InstallCommandService.class);
        planner = new CommandPlannerImp(installCommandService);

        AppEntry git = new AppEntry("Git", "git-catalog-id", Map.of("windows", Map.of("winget", "Git.Git")));
        apps = List.of(new AppSection("Dev tools", List.of(git)));
    }

    @Test
    void buildInstallCommandsResolvesCatalogIdsViaInstallCommandFor() {
        AppEntry git = apps.getFirst().apps().getFirst();
        when(installCommandService.installCommandFor(git, "winget"))
                .thenReturn(Optional.of("winget install --id Git.Git -e"));

        List<String> commands = planner.buildInstallCommands(apps, Set.of("git-catalog-id"), "winget");

        assertThat(commands).containsExactly("winget install --id Git.Git -e");
    }

    @Test
    void buildInstallCommandsFallsBackToGenericInstallCommandForUnknownIds() {
        when(installCommandService.installCommand("winget", "Some.Unknown"))
                .thenReturn("winget install --id Some.Unknown -e");

        List<String> commands = planner.buildInstallCommands(apps, Set.of("Some.Unknown"), "winget");

        assertThat(commands).containsExactly("winget install --id Some.Unknown -e");
    }

    @Test
    void buildInstallCommandsHandlesAMixOfCatalogAndUnknownIds() {
        AppEntry git = apps.getFirst().apps().getFirst();
        when(installCommandService.installCommandFor(git, "choco"))
                .thenReturn(Optional.of("choco install git -y"));
        when(installCommandService.installCommand("choco", "custom-pkg"))
                .thenReturn("choco install custom-pkg -y");

        Set<String> selected = new LinkedHashSet<>(List.of("git-catalog-id", "custom-pkg"));
        List<String> commands = planner.buildInstallCommands(apps, selected, "choco");

        assertThat(commands).containsExactlyInAnyOrder("choco install git -y", "choco install custom-pkg -y");
    }

    @Test
    void buildRemoveCommandsDelegatesEveryIdToRemoveCommand() {
        when(installCommandService.removeCommand("choco", "git-catalog-id")).thenReturn("choco uninstall git-catalog-id -y");

        List<String> commands = planner.buildRemoveCommands(Set.of("git-catalog-id"), "choco");

        assertThat(commands).containsExactly("choco uninstall git-catalog-id -y");
    }

    @Test
    void displayNamesResolvesCatalogIdsToTheirNameAndFallsBackToTheIdOtherwise() {
        List<String> names = planner.displayNames(apps, Set.of("git-catalog-id", "unknown-id"));

        assertThat(names).containsExactlyInAnyOrder("Git", "unknown-id");
    }
}
