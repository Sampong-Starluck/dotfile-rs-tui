package io.github.sampongstarluck.dotfile.service;

import java.util.List;

public interface SearchService {
    /** (binary, args) for the search command of each manager. */
    record Cmd(String binary, List<String> args) {}

    Cmd searchCommand(String mgr, String query);

    /** (binary, args) for listing installed packages. */
    Cmd listCommand(String mgr);

    /** Hint shown in the search input box for each manager. */
    String searchHint(String mgr);
}
