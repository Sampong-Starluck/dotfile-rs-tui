package com.sampong.dotfile.feature.status;

import dev.tamboui.style.Color;
import dev.tamboui.toolkit.element.Element;
import dev.tamboui.toolkit.elements.Row;
import com.sampong.dotfile.base.FeatureView;
import com.sampong.dotfile.model.PackageManager;
import com.sampong.dotfile.model.PanelId;
import com.sampong.dotfile.state.AppState;
import com.sampong.dotfile.ui.component.Panels;

import java.util.Optional;

import static dev.tamboui.toolkit.Toolkit.row;
import static dev.tamboui.toolkit.Toolkit.text;

/** {@code [1]-Status}: OS + active manager, one line. */
public class StatusView implements FeatureView {

    @Override
    public Element render(AppState st) {
        String os = st.platform.os != null ? st.platform.os.toString() : "detecting…";
        Optional<PackageManager> pm = st.platform.selectedManager();

        Row line = row(text(os).cyan().bold(), text(" · ").fg(Color.DARK_GRAY));
        line.add(pm.isPresent()
                ? text(pm.get().label()).yellow().bold()
                : text("no manager").fg(Color.DARK_GRAY));

        return Panels.framed(1, PanelId.STATUS.elementId(), "Status", line);
    }
}
