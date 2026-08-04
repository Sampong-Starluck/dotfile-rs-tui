package com.sampong.dotfile.service;

import org.jspecify.annotations.Nullable;

import java.util.List;
import java.util.concurrent.BlockingQueue;

/**
 * Streaming install/remove executor (port of {@code app_tab.rs}'s install/remove worker
 * thread, PLAN.md phase-09 §9.2). Runs each command with piped stdio, publishes an
 * {@code event.InstallLogEvent} per output line, and relays queued stdin to the child
 * process for interactive prompts (sudo password, y/n).
 */
public interface InstallExecutionService {
    void runStreaming(List<String> commands, @Nullable String sudoPassword, BlockingQueue<String> stdinQueue);
}
