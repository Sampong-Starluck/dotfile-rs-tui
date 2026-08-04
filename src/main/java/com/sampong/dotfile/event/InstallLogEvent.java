package com.sampong.dotfile.event;

/** One line of streaming install/remove output, published by {@code InstallExecutionService}
 *  and appended by {@code ui.feature.install.InstallLogBridge} to the queue the UI drains
 *  per frame (PLAN.md §6/§9.2). */
public record InstallLogEvent(String line) {}
