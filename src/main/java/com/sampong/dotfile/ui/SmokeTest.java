package com.sampong.dotfile.ui;

import dev.tamboui.toolkit.app.ToolkitApp;
import dev.tamboui.toolkit.element.Element;
import dev.tamboui.toolkit.event.EventResult;
import lombok.extern.slf4j.Slf4j;

import static dev.tamboui.toolkit.Toolkit.*;

@Slf4j
public class SmokeTest extends ToolkitApp {

    @Override
    protected Element render() {
        return panel("dotfile-java-tui",
                text("Hello from the Toolkit DSL + Panama backend").bold().cyan(),
                spacer(),
                text("press q to quit").dim()
            ).rounded()
             .onKeyEvent(e -> {
                 if (e.isChar('q')) {
                     quit();
                     return EventResult.HANDLED;
                 }
                 return EventResult.UNHANDLED;
             });
    }

    public void run() throws Exception {
        log.debug("startup: SmokeTest");
        super.run();
    }
}
