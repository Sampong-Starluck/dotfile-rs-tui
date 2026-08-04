package com.sampong.dotfile.ui.feature.managers;

import dev.tamboui.style.Color;
import dev.tamboui.style.Style;
import dev.tamboui.toolkit.element.Element;
import dev.tamboui.toolkit.elements.TableElement;
import dev.tamboui.widgets.table.Cell;
import dev.tamboui.widgets.table.Row;
import com.sampong.dotfile.base.FeatureView;
import com.sampong.dotfile.model.PackageManager;
import com.sampong.dotfile.model.PanelId;
import com.sampong.dotfile.model.PmCommand;
import com.sampong.dotfile.ui.state.AppState;
import com.sampong.dotfile.ui.component.Responsive;

import java.util.List;
import java.util.Optional;

import static dev.tamboui.toolkit.Toolkit.column;
import static dev.tamboui.toolkit.Toolkit.length;
import static dev.tamboui.toolkit.Toolkit.table;
import static dev.tamboui.toolkit.Toolkit.text;

/**
 * MAIN/COMMANDS: per-manager command cheat sheet. Port of {@code home_tab.rs::render_command_table}.
 * <p>
 * Previews the manager under the Managers-panel cursor (lazygit-style preview — moving the
 * cursor previews without activating), falling back to the active manager otherwise.
 * <p>
 * Returns bare content — {@code ui.TuiApp} wraps every main-view body in the single
 * {@code panel-main} frame (DRY: one place owns the MAIN panel's id/border/focus).
 */
public class CommandsView implements FeatureView {

    private static final Style HEADER_STYLE = Style.EMPTY.fg(Color.CYAN).bold();

    @Override
    public Element render(AppState st) {
        Optional<PackageManager> pm = previewManager(st);
        if (pm.isEmpty()) {
            return text("No package manager selected").dim();
        }
        List<PmCommand> commands = pm.get().commands();

        return Responsive.of(area -> {
            if (area.width() < 20 || area.height() < 4) {
                return text("terminal too small").dim();
            }

            int visibleRows = Math.max(1, area.height() - 2);
            int maxScroll = Math.max(0, commands.size() - visibleRows);
            int scroll = Math.min(st.platform.commandScroll, maxScroll);

            int cmdWidth = Math.max((int) (area.width() * 0.45), 15);
            int descWidth = Math.max(area.width() - cmdWidth, 0);
            boolean showDescription = descWidth >= 15;

            TableElement t = table()
                    .header(Row.from(Cell.from("Command").style(HEADER_STYLE), Cell.from("Description").style(HEADER_STYLE)))
                    .widths(length(cmdWidth), length(descWidth));
            for (int i = scroll; i < commands.size(); i++) {
                t.row(commandRow(commands.get(i), showDescription));
            }

            return commands.size() > visibleRows
                    ? column(text("[" + (scroll + 1) + "/" + commands.size() + "]").dim(), t)
                    : t;
        });
    }

    @Override
    public String title(AppState st) {
        return previewManager(st).map(pm -> "Commands — " + pm.label()).orElse("Commands");
    }

    private static Row commandRow(PmCommand cmd, boolean showDescription) {
        return Row.from(
                Cell.from(cmd.command()).style(Style.EMPTY.fg(Color.YELLOW)),
                Cell.from(showDescription ? cmd.description() : "").style(Style.EMPTY.fg(Color.WHITE)));
    }

    private static Optional<PackageManager> previewManager(AppState st) {
        if (st.focused == PanelId.MANAGERS && !st.platform.packageManagers.isEmpty()) {
            return Optional.of(st.platform.packageManagers.get(st.platform.managersCursor));
        }
        return st.platform.selectedManager();
    }
}
