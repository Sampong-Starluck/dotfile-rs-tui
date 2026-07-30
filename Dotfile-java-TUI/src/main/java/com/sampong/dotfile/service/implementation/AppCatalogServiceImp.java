package com.sampong.dotfile.service.implementation;

import com.sampong.dotfile.model.AppEntry;
import com.sampong.dotfile.model.AppSection;
import com.sampong.dotfile.model.ShellEntry;
import com.sampong.dotfile.model.ShellsFile;
import com.sampong.dotfile.service.AppCatalogService;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;
import tools.jackson.core.type.TypeReference;
import tools.jackson.databind.ObjectMapper;

import java.io.InputStream;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Map;
import java.util.Set;

/** Port of Rust {@code service/app_service.rs} + {@code service/script_service.rs::read_shells}. */
@Slf4j
@Service
@RequiredArgsConstructor
public class AppCatalogServiceImp implements AppCatalogService {

    private static final Set<String> KNOWN_SCRIPT_SHELLS =
            Set.of("bash", "zsh", "fish", "nushell", "powershell");

    private final ObjectMapper mapper;

    @Override
    public List<AppSection> readAppsJson() {
        try (InputStream in = getClass().getResourceAsStream("/data/apps.json")) {
            return mapper.readValue(in, new TypeReference<>() {
            });
        } catch (Exception e) {
            log.error("Failed to load apps.json", e);
            return List.of();
        }
    }

    @Override
    public List<AppSection> filterByPlatform(List<AppSection> apps, String osKey, String detectedMgr) {
        List<AppSection> out = new ArrayList<>();
        for (AppSection section : apps) {
            List<AppEntry> keep = section.apps().stream()
                    .filter(e -> {
                        Map<String, String> platform = e.platforms().get(osKey);
                        return platform != null && platform.containsKey(detectedMgr);
                    })
                    .toList();
            if (!keep.isEmpty()) out.add(new AppSection(section.section(), keep));
        }
        return out;
    }

    @Override
    public List<ShellEntry> readShellsJson() {
        try (InputStream in = getClass().getResourceAsStream("/data/shells.json")) {
            ShellsFile file = mapper.readValue(in, ShellsFile.class);
            return file.shells().stream()
                    .filter(s -> !s.hidden() && KNOWN_SCRIPT_SHELLS.contains(s.id()))
                    .sorted(Comparator.comparingInt(ShellEntry::order))
                    .toList();
        } catch (Exception e) {
            log.error("Failed to load shells.json", e);
            return List.of();
        }
    }
}
