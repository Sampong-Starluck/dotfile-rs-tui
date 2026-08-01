package com.sampong.dotfile.config;

import com.sampong.dotfile.model.AppEntry;
import com.sampong.dotfile.model.AppSection;
import com.sampong.dotfile.model.DotfileConfig;
import com.sampong.dotfile.model.SearchResult;
import com.sampong.dotfile.model.ShellEntry;
import com.sampong.dotfile.model.ShellsFile;
import org.jspecify.annotations.Nullable;
import org.springframework.aot.hint.RuntimeHints;
import org.springframework.aot.hint.RuntimeHintsRegistrar;
import org.springframework.aot.hint.annotation.RegisterReflectionForBinding;
import org.springframework.context.annotation.Configuration;
import org.springframework.context.annotation.ImportRuntimeHints;

/** GraalVM native-image reflection hints for records read via Jackson {@code TypeReference}/{@code Class} overloads. */
@Configuration
@RegisterReflectionForBinding({
        AppEntry.class,
        AppSection.class,
        ShellEntry.class,
        ShellsFile.class,
        DotfileConfig.class,
        SearchResult.class
})
@ImportRuntimeHints({NativeHints.TamboUiResources.class, NativeHints.AppDataResources.class})
public class NativeHints {

    /**
     * tamboui-tui ships its built-in key-binding sets ({@code dev/tamboui/tui/bindings/*.properties})
     * as plain classpath resources with no native-image metadata of its own (unlike
     * tamboui-panama-backend, which ships a reachability-metadata.json for its FFM downcalls) —
     * without this hint, {@code BindingSets.loadBuiltIn()} throws at startup under native-image.
     */
    static final class TamboUiResources implements RuntimeHintsRegistrar {
        @Override
        public void registerHints(RuntimeHints hints, @Nullable ClassLoader classLoader) {
            hints.resources().registerPattern("dev/tamboui/tui/bindings/*.properties");
        }
    }

    /**
     * GraalVM native-image excludes all classpath resources by default unless a hint says
     * otherwise — {@code AppCatalogServiceImp.readAppsJson/readShellsJson} and
     * {@code ScriptServiceImp.scriptContent}'s {@code getResourceAsStream} calls resolve fine on
     * the real JVM classpath (fat jar / {@code mvn spring-boot:run}) but return {@code null} in
     * the native exe without this, which is what native-image's Sections/Shells panels showing
     * empty traces back to ({@code IllegalArgumentException: argument "src" is null} from
     * Jackson, caught and logged as "Failed to load apps.json"/"...shells.json").
     */
    static final class AppDataResources implements RuntimeHintsRegistrar {
        @Override
        public void registerHints(RuntimeHints hints, @Nullable ClassLoader classLoader) {
            hints.resources().registerPattern("data/apps.json");
            hints.resources().registerPattern("data/shells.json");
            hints.resources().registerPattern("scripts/*/*");
        }
    }
}
