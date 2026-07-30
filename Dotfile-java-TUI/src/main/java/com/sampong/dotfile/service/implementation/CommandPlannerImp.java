package com.sampong.dotfile.service.implementation;

import com.sampong.dotfile.model.AppEntry;
import com.sampong.dotfile.model.AppSection;
import com.sampong.dotfile.service.CommandPlanner;
import com.sampong.dotfile.service.InstallCommandService;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Set;

/** Port of Rust {@code app_tab.rs::build_commands}/{@code build_remove_commands}/{@code selected_display_names}. */
@Slf4j
@Service
@RequiredArgsConstructor
public class CommandPlannerImp implements CommandPlanner {

    private final InstallCommandService installCommandService;

    @Override
    public List<String> buildInstallCommands(List<AppSection> apps, Set<String> selectedIds, String mgr) {
        Set<String> knownIds = new HashSet<>();
        List<String> commands = new ArrayList<>();
        for (AppSection section : apps) {
            for (AppEntry entry : section.apps()) {
                knownIds.add(entry.id());
                if (selectedIds.contains(entry.id())) {
                    installCommandService.installCommandFor(entry, mgr).ifPresent(commands::add);
                }
            }
        }
        for (String id : selectedIds) {
            if (!knownIds.contains(id)) {
                commands.add(installCommandService.installCommand(mgr, id));
            }
        }
        log.debug("buildInstallCommands: {} command(s) for mgr={}", commands.size(), mgr);
        return commands;
    }

    @Override
    public List<String> buildRemoveCommands(Set<String> selectedIds, String mgr) {
        return selectedIds.stream().map(id -> installCommandService.removeCommand(mgr, id)).toList();
    }

    @Override
    public List<String> displayNames(List<AppSection> apps, Set<String> ids) {
        return ids.stream().map(id -> displayName(apps, id)).toList();
    }

    private static String displayName(List<AppSection> apps, String id) {
        return apps.stream()
                .flatMap(s -> s.apps().stream())
                .filter(e -> e.id().equals(id))
                .findFirst()
                .map(e -> e.name())
                .orElse(id);
    }
}
