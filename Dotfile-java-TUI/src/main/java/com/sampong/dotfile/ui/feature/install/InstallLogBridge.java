package com.sampong.dotfile.ui.feature.install;

import com.sampong.dotfile.event.InstallLogEvent;
import lombok.extern.slf4j.Slf4j;
import org.jspecify.annotations.Nullable;
import org.springframework.context.event.EventListener;
import org.springframework.stereotype.Component;

import java.util.concurrent.ConcurrentLinkedQueue;

/**
 * Bridges {@link InstallLogEvent}s published from the async streaming worker to the queue the
 * UI drains per frame (PLAN.md §6/§9.2) — the single listener, SRP: it only appends + wakes the
 * render loop.
 * <p>
 * {@code renderWaker} is set once by {@code TuiApp.onStart()} rather than constructor-injected,
 * to avoid a {@code TuiApp <-> InstallLogBridge} circular bean dependency (see FEATURE-PARITY.md).
 */
@Slf4j
@Component
public class InstallLogBridge {

    private volatile @Nullable ConcurrentLinkedQueue<String> sink;
    private volatile @Nullable Runnable renderWaker;

    public void attach(ConcurrentLinkedQueue<String> queue) {
        this.sink = queue;
    }

    /** Detaches the current sink (e.g. on Esc, while the worker still runs to completion
     *  detached). Without this, late events from an abandoned worker would silently leak into
     *  whichever queue a later install attaches next, since {@link #sink} is a single field
     *  shared across every install/remove run. */
    public void detach() {
        this.sink = null;
    }

    public void setRenderWaker(Runnable waker) {
        this.renderWaker = waker;
    }

    @EventListener
    public void on(InstallLogEvent event) {
        ConcurrentLinkedQueue<String> queue = sink;
        if (queue != null) {
            queue.add(event.line());
        }
        Runnable waker = renderWaker;
        if (waker != null) {
            waker.run();
        }
    }
}
