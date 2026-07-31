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
@ImportRuntimeHints(NativeHints.TamboUiResources.class)
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
}
