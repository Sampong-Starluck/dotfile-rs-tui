package com.sampong.dotfile.service;

import java.util.List;
import java.util.Optional;

public interface SearchService {
    /** (binary, args) for the search command of each manager. */
    record Cmd(String binary, List<String> args) {}

    Cmd searchCommand(String mgr, String query);

    /** (binary, args) for listing installed packages. */
    Cmd listCommand(String mgr);

    /** (binary, args) for checking available updates (Phase 13, net-new — no Rust prior art).
     *  Empty for managers with no known update-check command yet; callers must not spawn a
     *  process on an empty result. */
    Optional<Cmd> upgradeListCommand(String mgr);

    /** Hint shown in the search input box for each manager. */
    String searchHint(String mgr);
}
