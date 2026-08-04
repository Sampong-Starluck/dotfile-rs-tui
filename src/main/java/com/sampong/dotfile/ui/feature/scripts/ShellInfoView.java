package com.sampong.dotfile.ui.feature.scripts;

import dev.tamboui.style.Color;
import dev.tamboui.toolkit.element.Element;
import dev.tamboui.toolkit.elements.Column;
import dev.tamboui.toolkit.elements.Row;
import dev.tamboui.toolkit.elements.TextElement;
import com.sampong.dotfile.base.FeatureView;
import com.sampong.dotfile.model.ShellStatus;
import com.sampong.dotfile.ui.component.Logs;
import com.sampong.dotfile.ui.component.Responsive;
import com.sampong.dotfile.ui.component.Sized;
import com.sampong.dotfile.ui.state.AppState;

import java.nio.file.Path;
import java.util.List;

import static dev.tamboui.toolkit.Toolkit.column;
import static dev.tamboui.toolkit.Toolkit.row;
import static dev.tamboui.toolkit.Toolkit.text;

/** MAIN/SHELL_INFO: shell info (port of {@code render_info}) + action log, split vertically. */
public class ShellInfoView implements FeatureView {

    private static final String LABEL_SHELL = "  Shell      ";
    private static final String LABEL_BINARY = "  Binary     ";
    private static final String LABEL_DESC = "  Desc       ";
    private static final String LABEL_STATUS = "  Status     ";
    private static final String LABEL_PLATFORMS = "  Platforms  ";
    private static final String LABEL_PRIMARY = "  Primary    ";
    private static final String LABEL_SCRIPT_DIR = "  Script dir ";
    private static final String LABEL_PROFILE = "  Profile    ";

    @Override
    public Element render(AppState st) {
        ShellStatus status = selected(st);
        if (status == null) {
            return text(st.scripts.shells == null ? "Loading…" : "No shell selected.").dim();
        }

        return Responsive.of(area -> {
            int logHeight = clamp(area.height() / 3, 5, 12);
            return column(
                    Sized.fill(infoColumn(status, st)),
                    Sized.length(Logs.colored(st.scripts.log, Math.max(1, logHeight - 2)), logHeight));
        });
    }

    @Override
    public String title(AppState st) {
        ShellStatus status = selected(st);
        return status != null ? status.entry().name() : "Shell Info";
    }

    private static ShellStatus selected(AppState st) {
        List<ShellStatus> shells = st.scripts.shells;
        if (shells == null || st.scripts.shellCursor >= shells.size()) {
            return null;
        }
        return shells.get(st.scripts.shellCursor);
    }

    private static Column infoColumn(ShellStatus status, AppState st) {
        ShellIcon.Glyph icon = ShellIcon.of(status.detected(), status.deployed());
        boolean explicitPrimary = status.entry().id().equals(st.scripts.explicitPrimaryShell);
        boolean defaultPrimary = !explicitPrimary && status.entry().id().equals(st.scripts.primaryShell);

        Column col = column(
                text(""),
                labelRow(LABEL_SHELL, text(status.entry().name()).cyan()),
                labelRow(LABEL_BINARY, text(status.binary() != null ? status.binary() : "—").yellow()),
                labelRow(LABEL_DESC, text(status.entry().description())),
                labelRow(LABEL_STATUS, text(icon.symbol() + "  " + icon.label()).fg(icon.color()).bold()),
                platformsRow(status.entry().platforms(), st),
                primaryRow(explicitPrimary, defaultPrimary),
                labelRow(LABEL_SCRIPT_DIR, text(pathOrDash(status.targetPath()))),
                labelRow(LABEL_PROFILE, text(pathOrDash(status.profilePath()))),
                text(""));

        String hint = status.sourceHint();
        if (hint != null && !hint.isEmpty() && status.detected()) {
            col.add(text("  ── Source line (added automatically on deploy) ────────────────────────────").dim());
            col.add(text(""));
            col.add(row(text("    "), text(hint).yellow()));
        }

        if (!status.entry().requires().isEmpty()) {
            col.add(text(""));
            col.add(text("  Requires: " + String.join(", ", status.entry().requires())).dim());
        }

        return col;
    }

    private static Row labelRow(String label, TextElement value) {
        return row(text(label).dim(), value);
    }

    private static Row primaryRow(boolean explicit, boolean isDefault) {
        if (explicit) {
            return row(text(LABEL_PRIMARY).dim(), text("★  set as primary").yellow().bold());
        }
        if (isDefault) {
            return row(text(LABEL_PRIMARY).dim(), text("◇  system default ($SHELL)").cyan());
        }
        return row(text(LABEL_PRIMARY).dim(), text("—").dim());
    }

    private static Row platformsRow(List<String> platforms, AppState st) {
        Row r = row(text(LABEL_PLATFORMS).dim());
        String current = st.platform.os != null ? st.platform.os.key() : "";
        for (int i = 0; i < platforms.size(); i++) {
            if (i > 0) {
                r.add(text("  "));
            }
            String p = platforms.get(i);
            r.add(p.equals(current) ? text(p).bold().fg(Color.BLACK).bg(Color.CYAN) : text(p).dim());
        }
        return r;
    }

    private static String pathOrDash(Path p) {
        return p != null ? p.toString() : "—";
    }

    private static int clamp(int value, int min, int max) {
        return Math.min(Math.max(value, min), max);
    }
}
