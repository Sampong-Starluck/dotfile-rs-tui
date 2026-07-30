package com.sampong.dotfile.ui.component;

import dev.tamboui.layout.Constraint;
import dev.tamboui.layout.Rect;
import dev.tamboui.terminal.Frame;
import dev.tamboui.toolkit.element.Element;
import dev.tamboui.toolkit.element.RenderContext;
import dev.tamboui.toolkit.element.Size;

import java.util.function.Function;

/**
 * Rebuilds its content from the real rendered area on every frame.
 * <p>
 * The fluent tree is built once per frame, before the Cassowary layout solver assigns
 * areas — so width/height-dependent content (narrow-panel degrade rules) can't be
 * decided while building the tree. This defers that decision to {@link #render}, where
 * the real {@link Rect} is known, mirroring how {@code LazyElement} defers content
 * evaluation itself just without area awareness.
 */
public final class Responsive implements Element {
    private final Function<Rect, ? extends Element> factory;
    private Element last;

    private Responsive(Function<Rect, ? extends Element> factory) {
        this.factory = factory;
    }

    public static Responsive of(Function<Rect, ? extends Element> factory) {
        return new Responsive(factory);
    }

    @Override
    public void render(Frame frame, Rect area, RenderContext context) {
        last = factory.apply(area);
        if (last != null) {
            context.renderChild(last, frame, area);
        }
    }

    @Override
    public Size preferredSize(int availableWidth, int availableHeight, RenderContext context) {
        return Size.UNKNOWN;
    }

    @Override
    public Constraint constraint() {
        return last != null ? last.constraint() : null;
    }
}
