package com.sampong.dotfile.feature.status;

import dev.tamboui.toolkit.element.Element;
import com.sampong.dotfile.base.FeatureView;
import com.sampong.dotfile.model.PanelId;
import com.sampong.dotfile.state.AppState;
import com.sampong.dotfile.ui.component.Panels;

import static dev.tamboui.toolkit.Toolkit.text;

/** {@code [1]-Status}: OS + active manager, one line. */
public class StatusView implements FeatureView {

    @Override
    public Element render(AppState st) {
        String os = st.platform.os != null ? st.platform.os.toString() : "detecting…";
        return Panels.framed(1, PanelId.STATUS.elementId(), "Status",
                text(os + " · " + st.platform.activeBinary()));
    }
}
