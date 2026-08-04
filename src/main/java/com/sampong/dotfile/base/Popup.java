package com.sampong.dotfile.base;

/**
 * One modal at a time; a popup bundles its overlay view + key controller.
 * <p>
 * Permitted types live in this same package (not in their feature packages, per the original
 * plan) because {@code permits} on a sealed type requires same-package subtypes when the
 * project has no {@code module-info.java} (unnamed module) — see FEATURE-PARITY.md deviations.
 */
public sealed interface Popup
        permits SearchInputPopup, CustomInputPopup, ConfirmActionPopup,
                SudoPopup, InstallLogPopup, HelpPopup {
    FeatureView view();
    KeyController controller();
}
