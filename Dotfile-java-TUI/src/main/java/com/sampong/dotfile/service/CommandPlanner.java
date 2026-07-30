package com.sampong.dotfile.service;

import com.sampong.dotfile.model.AppSection;

import java.util.List;
import java.util.Set;

/**
 * Port of {@code app_tab.rs::build_commands}/{@code build_remove_commands}/
 * {@code selected_display_names}: pure command-string logic extracted out of the UI (SRP) so
 * controllers stay free of command-string knowledge.
 */
public interface CommandPlanner {
    /** Catalog ids resolve via {@link InstallCommandService#installCommandFor}; unknown ids fall
     *  back to {@link InstallCommandService#installCommand}. */
    List<String> buildInstallCommands(List<AppSection> apps, Set<String> selectedIds, String mgr);

    List<String> buildRemoveCommands(Set<String> selectedIds, String mgr);

    /** Catalog ids resolve to their {@code AppEntry.name()}; unknown ids display as-is. */
    List<String> displayNames(List<AppSection> apps, Set<String> ids);
}
