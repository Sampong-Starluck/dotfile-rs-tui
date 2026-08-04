package com.sampong.dotfile.model;

/** Side panels + main, in Tab-cycle order. Number keys 1-4 map to the first four. */
public enum PanelId {
    STATUS, MANAGERS, SECTIONS, SHELLS, MAIN;

    public PanelId next() { return values()[(ordinal() + 1) % values().length]; }
    public PanelId prev() { return values()[(ordinal() + values().length - 1) % values().length]; }

    /** The toolkit element id this panel is registered under (focus lookups, {@code ui/}). */
    public String elementId() { return "panel-" + name().toLowerCase(); }
}
