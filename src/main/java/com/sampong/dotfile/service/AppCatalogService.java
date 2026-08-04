package com.sampong.dotfile.service;

import com.sampong.dotfile.model.AppSection;
import com.sampong.dotfile.model.ShellEntry;
import java.util.List;

public interface AppCatalogService {
    List<AppSection> readAppsJson();
    List<AppSection> filterByPlatform(List<AppSection> apps, String osKey, String detectedMgr);
    List<ShellEntry> readShellsJson();
}
