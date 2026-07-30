package io.github.sampongstarluck.dotfile.service;

import io.github.sampongstarluck.dotfile.model.AppSection;
import io.github.sampongstarluck.dotfile.model.ShellEntry;
import java.util.List;

public interface AppCatalogService {
    List<AppSection> readAppsJson();
    List<AppSection> filterByPlatform(List<AppSection> apps, String osKey, String detectedMgr);
    List<ShellEntry> readShellsJson();
}
