package com.sampong.dotfile.service.implementation;

import com.sampong.dotfile.event.InstallLogEvent;
import com.sampong.dotfile.service.DecodeUtil;
import com.sampong.dotfile.service.InstallExecutionService;
import com.sampong.dotfile.service.TextUtil;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.jspecify.annotations.Nullable;
import org.springframework.context.ApplicationEventPublisher;
import org.springframework.scheduling.annotation.Async;
import org.springframework.stereotype.Service;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.io.OutputStreamWriter;
import java.io.PrintWriter;
import java.nio.charset.StandardCharsets;
import java.util.List;
import java.util.concurrent.BlockingQueue;

/**
 * Port of {@code app_tab.rs}'s install/remove executor thread + {@code drain_stdout_to_log}
 * (PLAN.md phase-09 §9.2/§9.3). {@code runStreaming} itself runs on a Spring async virtual
 * thread; the stdin-relay and stderr-pump helper threads it spawns are the only raw
 * {@code Thread.ofVirtual} usages in the app (phase-09 Definition of Done).
 */
@Slf4j
@Service
@RequiredArgsConstructor
public class InstallExecutionServiceImp implements InstallExecutionService {

    private final ApplicationEventPublisher events;

    @Override
    @Async
    public void runStreaming(List<String> commands, @Nullable String sudoPassword, BlockingQueue<String> stdinQueue) {
        for (String cmd : commands) {
            events.publishEvent(new InstallLogEvent("▶ Running: " + cmd));
            List<String> parts = List.of(cmd.trim().split("\\s+"));
            if (parts.isEmpty() || parts.getFirst().isEmpty()) {
                continue;
            }
            try {
                runOne(parts, sudoPassword, stdinQueue);
            } catch (Exception e) {
                log.warn("failed to run {}", parts.getFirst(), e);
                events.publishEvent(new InstallLogEvent("✗ Could not run " + parts.getFirst() + ": " + e.getMessage()));
            }
        }
        events.publishEvent(new InstallLogEvent("═══ All done ═══"));
    }

    private void runOne(List<String> parts, @Nullable String sudoPassword, BlockingQueue<String> stdinQueue)
            throws IOException, InterruptedException {
        Process child = new ProcessBuilder(parts).start();

        PrintWriter stdin = new PrintWriter(
                new OutputStreamWriter(child.getOutputStream(), StandardCharsets.UTF_8), true);
        if (sudoPassword != null) {
            stdin.println(sudoPassword);
        }
        Thread relay = Thread.ofVirtual().start(() -> {
            try {
                while (true) {
                    stdin.println(stdinQueue.take());
                }
            } catch (InterruptedException stop) {
                Thread.currentThread().interrupt();
            }
        });

        Thread errPump = Thread.ofVirtual().start(() -> pumpStderr(child));
        drainStdout(child);
        errPump.join();
        int code = child.waitFor();
        relay.interrupt();

        events.publishEvent(new InstallLogEvent(code == 0
                ? "✓ Done: " + parts.getFirst()
                : "✗ Failed (exit " + code + ")"));
    }

    /** Port of {@code drain_stdout_to_log}: winget's {@code \r} progress animation makes
     *  line-streaming meaningless, so the whole stream is read to EOF first, then split. */
    private void drainStdout(Process child) {
        byte[] raw;
        try (var in = child.getInputStream()) {
            raw = in.readAllBytes();
        } catch (IOException e) {
            return;
        }
        String text = new String(raw, StandardCharsets.UTF_8);
        for (String segment : text.split("[\r\n]")) {
            String clean = TextUtil.stripAnsi(segment);
            if (DecodeUtil.isNoiseLine(clean)) {
                continue;
            }
            events.publishEvent(new InstallLogEvent(clean));
        }
    }

    /** Port of the Rust stderr pass: sanitize, drop noise, drop sudo-prompt echoes. */
    private void pumpStderr(Process child) {
        try (var reader = new BufferedReader(new InputStreamReader(child.getErrorStream(), StandardCharsets.UTF_8))) {
            String line;
            while ((line = reader.readLine()) != null) {
                String clean = TextUtil.sanitizeLine(line);
                if (DecodeUtil.isNoiseLine(clean) || clean.contains("[sudo]") || clean.contains("password for")) {
                    continue;
                }
                events.publishEvent(new InstallLogEvent("[err] " + clean));
            }
        } catch (IOException e) {
            // best-effort — process may have already exited
        }
    }
}
