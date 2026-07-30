package com.sampong.dotfile.ui.state;

import com.sampong.dotfile.base.Popup;
import com.sampong.dotfile.model.MainView;
import com.sampong.dotfile.model.PanelId;
import org.jspecify.annotations.Nullable;

/**
 * Root state: navigation only + one composed state object per feature.
 * A plain mutable class created inside {@code ui.TuiApp}, never a Spring bean.
 */
public class AppState {
    public PanelId focused = PanelId.MANAGERS;
    public MainView mainView = MainView.COMMANDS;
    public @Nullable Popup popup = null;
    public boolean running = true;

    /**
     * A controller-requested programmatic focus jump (e.g. Enter-into-MAIN), applied by
     * {@code ui.TuiApp} through the toolkit's own focus manager before the next
     * {@code syncFocus()} — controllers never touch the toolkit focus manager directly.
     */
    public @Nullable PanelId pendingFocus = null;

    public final PlatformState platform = new PlatformState();
    public final CatalogState catalog = new CatalogState();
    public final SearchState search = new SearchState();
    public final InstalledState installed = new InstalledState();
    public final InstallState install = new InstallState();
    public final ScriptsState scripts = new ScriptsState();

    /**
     * Cross-feature reset: switching the active manager invalidates everything
     * derived from it (port of the Rust reset block).
     */
    public void activateManager(int idx) {
        if (!platform.activate(idx)) {
            return;
        }
        catalog.reset();
        search.reset();
        installed.reset();
    }

    /** Request a programmatic focus jump; applied by {@code TuiApp} on the next frame. */
    public void requestFocus(PanelId id) {
        pendingFocus = id;
    }
}
