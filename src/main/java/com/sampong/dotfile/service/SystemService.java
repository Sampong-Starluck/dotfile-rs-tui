package com.sampong.dotfile.service;

public interface SystemService {
    boolean isRoot();

    boolean requiresSudo(String mgr);

    /** managers whose prompts go to the real terminal → must suspend the TUI */
    boolean requiresInteractive(String mgr);
}
