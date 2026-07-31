package com.sampong.dotfile.base;

import dev.tamboui.toolkit.element.Element;
import dev.tamboui.toolkit.elements.Column;
import dev.tamboui.toolkit.elements.DialogElement;
import dev.tamboui.toolkit.event.EventResult;
import dev.tamboui.widgets.scrollbar.ScrollbarState;
import com.sampong.dotfile.ui.Bindings;
import com.sampong.dotfile.ui.Keys;
import com.sampong.dotfile.ui.component.Popups;
import com.sampong.dotfile.ui.component.Responsive;
import com.sampong.dotfile.ui.component.UiText;

import java.util.ArrayList;
import java.util.List;

import static dev.tamboui.toolkit.Toolkit.column;
import static dev.tamboui.toolkit.Toolkit.row;
import static dev.tamboui.toolkit.Toolkit.text;

/**
 * {@code ?} help overlay. Content is generated from {@link Bindings#sections()} — this popup
 * owns zero key text itself (PLAN.md phase-10 §10.2, single source of truth with the hint bar).
 * <p>
 * The full binding list (~60 lines) rarely fits one screen, so the body scrolls: {@code j/k}/
 * arrows one line, {@code pgup}/{@code pgdn} one page. {@link #scroll} is a real
 * {@code ScrollbarState} (a toolkit widget-state object, same pattern as {@code TextInputState}
 * on the input popups) held on the record so position survives across the per-frame rebuild —
 * see the windowing note on {@link #view()}.
 * <p>
 * Lives in {@code base} (not its own feature package) because {@code permits} on a sealed
 * type requires same-package subtypes when the project has no {@code module-info.java}
 * (unnamed module) — see FEATURE-PARITY.md deviations.
 */
public record HelpPopup(ScrollbarState scroll) implements Popup {

    private static final int KEY_CHIP_WIDTH = 16;
    /** Fixed dialog height, clamped to the real terminal by {@code DialogElement} itself
     *  (see {@code renderContent}'s {@code Math.min(dialogHeight, area.height())}) — needed
     *  because a scrollable body can't report a real {@code preferredSize()} for the normal
     *  sum-of-children height calc to work from (see FEATURE-PARITY.md Phase 10 deviations). */
    private static final int DIALOG_HEIGHT = 30;

    public HelpPopup() {
        this(new ScrollbarState());
    }

    @Override
    public FeatureView view() {
        return st -> {
            List<Element> lines = buildLines();

            // Width must be measured from the FULL (unwindowed) content: the real body below is
            // wrapped in Responsive, whose preferredSize() is deliberately unknown (its content
            // depends on the render-time area), so DialogElement's own children-preferredSize
            // width calc can't see it — same root cause Popups.overlay's own width fix exists for.
            int width = Popups.measureWidth("? Keybindings", lines.toArray(new Element[0]));

            Element body = Responsive.of(area -> {
                int footerRows = 1;
                int visibleRows = Math.max(1, area.height() - footerRows);
                scroll.contentLength(lines.size()).viewportContentLength(visibleRows);
                if (lines.size() <= visibleRows) {
                    scroll.position(0);
                }
                int start = scroll.position();
                int end = Math.min(lines.size(), start + visibleRows);

                Column col = column();
                for (int i = start; i < end; i++) {
                    col.add(lines.get(i));
                }
                col.add(text(""));
                col.add(text((start + 1) + "-" + end + "/" + lines.size()
                        + "  j/k scroll · pgup/pgdn page").dim());
                return col;
            });

            DialogElement dialog = Popups.overlay("? Keybindings", body).length(DIALOG_HEIGHT);
            return dialog.width(width);
        };
    }

    private static List<Element> buildLines() {
        List<Element> lines = new ArrayList<>();
        List<Bindings.Section> sections = Bindings.sections();
        for (int i = 0; i < sections.size(); i++) {
            if (i > 0) {
                lines.add(text(""));
            }
            Bindings.Section section = sections.get(i);
            lines.add(text("── " + section.label() + " " + "─".repeat(30)).yellow().bold().dim());
            for (Bindings.Binding b : section.bindings()) {
                lines.add(row(
                        text(UiText.padRight(b.key(), KEY_CHIP_WIDTH)).cyan().reversed(),
                        text(" " + b.desc()).white()));
            }
        }
        return lines;
    }

    @Override
    public KeyController controller() {
        return (key, st) -> {
            if (key.isChar('?') || key.isChar('q') || Keys.isEsc(key)) {
                st.popup = null;
                return EventResult.HANDLED;
            }
            if (Keys.isUp(key)) {
                scroll.prev();
                return EventResult.HANDLED;
            }
            if (Keys.isDown(key)) {
                scroll.next();
                return EventResult.HANDLED;
            }
            if (Keys.isPageUp(key)) {
                scroll.pageUp();
                return EventResult.HANDLED;
            }
            if (Keys.isPageDown(key)) {
                scroll.pageDown();
                return EventResult.HANDLED;
            }
            return EventResult.HANDLED;
        };
    }
}
