package com.sampong.dotfile.ui.component;

import dev.tamboui.layout.Constraint;
import dev.tamboui.layout.Rect;
import dev.tamboui.terminal.Frame;
import dev.tamboui.toolkit.element.Element;
import dev.tamboui.toolkit.element.RenderContext;
import dev.tamboui.toolkit.element.Size;
import dev.tamboui.toolkit.event.EventResult;
import dev.tamboui.toolkit.event.KeyEventHandler;
import dev.tamboui.toolkit.event.MouseEventHandler;
import dev.tamboui.tui.event.KeyEvent;
import dev.tamboui.tui.event.MouseEvent;

/**
 * Attaches an explicit {@link Constraint} to an opaque {@code Element} (e.g. a {@code Panel}
 * returned by a {@code FeatureView}, typed as {@code Element} by the time it reaches
 * {@code LazygitLayout}) without the sizing bug a plain {@code column(child).fill()}/{@code
 * .length(n)} wrapper has.
 * <p>
 * That plain-column wrapper only reports the RIGHT constraint to the wrapper's own parent; when
 * the wrapper column then lays out its single child, it re-derives a constraint for the child from
 * scratch via {@code child.constraint()}. Since a bare {@code Panel} has no explicit constraint of
 * its own, the column falls back to the child's {@code preferredSize()} — which for a bordered
 * panel is always a real, content-sized number, never "unknown" — so it hands the child a fixed
 * {@code Constraint.length(contentHeight)} instead of stretching it, leaving the rest of the
 * wrapper's allocated area blank. {@code Sized} sidesteps that by exposing the constraint directly
 * on itself (so its parent's per-child logic uses it as-is, per {@code Column.renderContent})
 * and rendering with a direct pass-through of the whole given area to the child, skipping that
 * inner re-derivation entirely.
 */
public final class Sized implements Element {
    private final Element child;
    private final Constraint constraint;

    private Sized(Element child, Constraint constraint) {
        this.child = child;
        this.constraint = constraint;
    }

    public static Sized length(Element child, int rows) {
        return new Sized(child, Constraint.length(rows));
    }

    public static Sized fill(Element child) {
        return new Sized(child, Constraint.fill());
    }

    @Override
    public void render(Frame frame, Rect area, RenderContext context) {
        context.renderChild(child, frame, area);
    }

    @Override
    public Size preferredSize(int availableWidth, int availableHeight, RenderContext context) {
        return child.preferredSize(availableWidth, availableHeight, context);
    }

    @Override
    public Constraint constraint() {
        return constraint;
    }

    @Override
    public String id() {
        return child.id();
    }

    @Override
    public boolean isFocusable() {
        return child.isFocusable();
    }

    @Override
    public KeyEventHandler keyEventHandler() {
        return child.keyEventHandler();
    }

    @Override
    public MouseEventHandler mouseEventHandler() {
        return child.mouseEventHandler();
    }

    @Override
    public boolean isDraggable() {
        return child.isDraggable();
    }

    @Override
    public Rect renderedArea() {
        return child.renderedArea();
    }

    @Override
    public EventResult handleKeyEvent(KeyEvent event, boolean focused) {
        return child.handleKeyEvent(event, focused);
    }

    @Override
    public EventResult handleMouseEvent(MouseEvent event) {
        return child.handleMouseEvent(event);
    }
}
