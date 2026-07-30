# Phase 1 — Toolchain (mise + GraalVM 25), project setup, Panama smoke test

**Goal:** mise-managed GraalVM 25 toolchain; a Spring Boot 4.1 CLI app that
opens a TamboUI full-screen terminal through the **Panama FFM backend**,
shows "Hello dotfile-java-tui", quits on `q`, logs to `debug.log` only.

## 1.1 mise toolchain

Install mise if absent (`winget install jdx.mise`), then create
`mise.toml` in this folder:

```toml
[tools]
# GraalVM JDK 25. Find the exact available name first:
#   mise ls-remote java | Select-String graalvm | Select-String 25
# Prefer "oracle-graalvm-25.x", else "graalvm-community-25.x".
java = "oracle-graalvm-25"
maven = "3.9"

[env]
# Panama FFM backend performs native downcalls — required on JDK 24+:
JAVA_TOOL_OPTIONS = "--enable-native-access=ALL-UNNAMED"

[tasks.dev]
run = "mvn -q spring-boot:run"
description = "Run the TUI in dev mode"

[tasks.test]
run = "mvn -q test"

[tasks.build]
run = "mvn -q clean package"

[tasks.native]
run = "mvn -q -Pnative clean native:compile"
description = "GraalVM native image (Phase 11)"
```

Then:

```powershell
mise trust
mise install
mise exec -- java -version    # must print GraalVM, version 25.x
```

Record the exact resolved `java` tool string in this file once known.

## 1.2 Maven skeleton

```powershell
mise exec -- mvn -N wrapper:wrapper -Dmaven=3.9.9
```

### `pom.xml` (complete — use verbatim; only bump versions if unresolvable)

```xml
<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0"
         xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
         xsi:schemaLocation="http://maven.apache.org/POM/4.0.0 https://maven.apache.org/xsd/maven-4.0.0.xsd">
  <modelVersion>4.0.0</modelVersion>

  <parent>
    <groupId>org.springframework.boot</groupId>
    <artifactId>spring-boot-starter-parent</artifactId>
    <version>4.1.0</version> <!-- if unresolvable, newest 4.1.x on Central -->
    <relativePath/>
  </parent>

  <groupId>io.github.sampongstarluck</groupId>
  <artifactId>dotfile-java-tui</artifactId>
  <version>0.1.0</version>
  <name>dotfile-java-tui</name>
  <description>Lazygit-style TUI wrapper for package managers (Java port of dotfile-rs-tui)</description>

  <properties>
    <java.version>25</java.version>
    <tamboui.version>0.4.0</tamboui.version>
  </properties>

  <dependencyManagement>
    <dependencies>
      <dependency>
        <groupId>dev.tamboui</groupId>
        <artifactId>tamboui-bom</artifactId>
        <version>${tamboui.version}</version>
        <type>pom</type>
        <scope>import</scope>
      </dependency>
    </dependencies>
  </dependencyManagement>

  <dependencies>
    <dependency>
      <groupId>org.springframework.boot</groupId>
      <artifactId>spring-boot-starter</artifactId>
    </dependency>

    <!-- TamboUI: Toolkit DSL (the fluent API — pulls widgets/core transitively)
         + Panama FFM terminal backend -->
    <dependency>
      <groupId>dev.tamboui</groupId>
      <artifactId>tamboui-toolkit</artifactId>
    </dependency>
    <dependency>
      <groupId>dev.tamboui</groupId>
      <artifactId>tamboui-panama-backend</artifactId>
    </dependency>

    <!-- Lombok: Boot-managed version; compile-time only (excluded from the jar) -->
    <dependency>
      <groupId>org.projectlombok</groupId>
      <artifactId>lombok</artifactId>
      <optional>true</optional>
    </dependency>

    <dependency>
      <groupId>org.springframework.boot</groupId>
      <artifactId>spring-boot-starter-test</artifactId>
      <scope>test</scope>
    </dependency>
  </dependencies>

  <build>
    <plugins>
      <plugin>
        <groupId>org.apache.maven.plugins</groupId>
        <artifactId>maven-compiler-plugin</artifactId>
        <configuration>
          <annotationProcessorPaths>
            <path>
              <groupId>org.projectlombok</groupId>
              <artifactId>lombok</artifactId>
            </path>
          </annotationProcessorPaths>
        </configuration>
      </plugin>
      <plugin>
        <groupId>org.springframework.boot</groupId>
        <artifactId>spring-boot-maven-plugin</artifactId>
        <configuration>
          <jvmArguments>--enable-native-access=ALL-UNNAMED</jvmArguments>
          <excludes>
            <exclude>
              <groupId>org.projectlombok</groupId>
              <artifactId>lombok</artifactId>
            </exclude>
          </excludes>
        </configuration>
      </plugin>
    </plugins>
  </build>
  <!-- Phase 11 adds the <profiles><profile><id>native</id>… block -->
</project>
```

If `tamboui-bom` does not resolve: drop the `dependencyManagement` block and
version each `dev.tamboui` dependency with `${tamboui.version}` directly.

**Lombok on JDK 25:** the Boot-managed Lombok version should support JDK 25;
if annotation processing fails with a JDK-version error, override
`<lombok.version>` in `<properties>` to the newest release on Central.
Lombok usage rules are in PLAN.md §4a (`@Slf4j` + `@RequiredArgsConstructor`
only). MapStruct is deliberately absent (PLAN.md §3 stack table).
If `tamboui-panama-backend:0.4.0` does not exist on Central, check which
version the BOM manages / what exists (search `dev.tamboui` on
central.sonatype.com), pin that explicitly, and note it in FEATURE-PARITY.md.

## 1.3 Discover the real TamboUI Toolkit API (RULE #0)

```powershell
mise exec -- mvn -q dependency:resolve
# jars land in ~/.m2/repository/dev/tamboui/**
jar tf <tamboui-toolkit-0.4.0.jar>        > api-toolkit.txt
jar tf <tamboui-widgets-0.4.0.jar>        > api-widgets.txt
jar tf <tamboui-core-0.4.0.jar>           > api-core.txt
jar tf <tamboui-panama-backend-…​.jar>     > api-panama.txt
```

Locate and write the real names under each bullet (edit this file). All UI
code uses the **Toolkit DSL (fluent API)** — these are the names every later
phase depends on:

- [ ] `ToolkitApp` base class (`render()` override, `run()`) and/or
      `ToolkitRunner` — and **how the Panama backend is selected/passed**
- [ ] `Element` interface (incl. its `render(Frame, Rect, RenderContext)`
      method — needed for custom overlay popups)
- [ ] The static factory class + methods: `panel()`, `text()`, `row()`,
      `column()`, `columns()`, `list()`, `table()`, `spacer()`, `spinner()`
      (write down the exact class to static-import)
- [ ] Fluent styling chain: `.bold() .dim() .reversed() .cyan() .green()
      .yellow() .red() .rounded() .title(…) .borderColor(…)
      .focusedBorderColor(…)` — note exact color enum/values
- [ ] Focus API: `.id(String)`, `.focusable()`, programmatic focus request
      (for the `1-4` jump keys), and how to query/observe the focused id
- [ ] Key handling: `.onKeyEvent(handler)`, `KeyEvent` accessors
      (char, Enter/Esc/Backspace/Tab/Shift-Tab), `EventResult.HANDLED/UNHANDLED`
- [ ] Sizing/constraints on rows/columns (fixed height/width, fill,
      percentage) — needed for the lazygit layout
- [ ] Stateful element state objects: list selection state, text-input state
- [ ] Refresh/tick: how the runner re-renders for animations
      (built-in `spinner()` implies a tick) and how to request a re-render
      when a background task finishes
- [ ] Suspend/resume or pause API on the runner (Phase 9 external commands)
- [ ] Mouse: confirmed click-to-focus behavior; any row-click events
- [ ] **Windows check:** does the Panama backend support Windows consoles in
      this version? Run the smoke test (1.5) to prove it. If it hard-fails on
      Windows, STOP and report to the user before proceeding (do not silently
      swap backends — the backend choice is a user decision).

## 1.4 Spring entry point + config package

`DotfileTuiApplication.java`:

```java
package io.github.sampongstarluck.dotfile;

import org.springframework.boot.Banner;
import org.springframework.boot.CommandLineRunner;
import org.springframework.boot.SpringApplication;
import org.springframework.boot.WebApplicationType;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.context.annotation.Bean;

@SpringBootApplication
public class DotfileTuiApplication {

    public static void main(String[] args) {
        var app = new SpringApplication(DotfileTuiApplication.class);
        app.setWebApplicationType(WebApplicationType.NONE);
        app.setBannerMode(Banner.Mode.OFF);
        app.setLogStartupInfo(false);
        System.exit(SpringApplication.exit(app.run(args)));
    }

    @Bean
    CommandLineRunner tui() {
        return args -> new io.github.sampongstarluck.dotfile.ui.SmokeTest().run();
        // Phase 5 replaces this with the injected ui.TuiApp bean
    }
}
```

`config/AsyncConfig.java` (used from Phase 9, created now):

```java
package io.github.sampongstarluck.dotfile.config;

import org.springframework.context.annotation.Configuration;
import org.springframework.scheduling.annotation.EnableAsync;

@Configuration
@EnableAsync
public class AsyncConfig { }
// virtual threads enabled via application.yml: spring.threads.virtual.enabled=true
```

`config/AppProperties.java`:

```java
package io.github.sampongstarluck.dotfile.config;

import org.springframework.boot.context.properties.ConfigurationProperties;

@ConfigurationProperties(prefix = "dotfile")
public record AppProperties(String dataDirName) {
    public AppProperties {
        if (dataDirName == null || dataDirName.isBlank()) dataDirName = "dotfile-rs";
        // keep "dotfile-rs" so deployments from the Rust app stay compatible
    }
}
```
Enable with `@ConfigurationPropertiesScan` on the application class.

## 1.5 Smoke test TUI (fluent Toolkit DSL)

`ui/SmokeTest.java` — adapt imports/names to §1.3 findings. Target shape
(from the official getting-started example):

```java
package io.github.sampongstarluck.dotfile.ui;

// static-import the toolkit factory class found in api-toolkit.txt

public class SmokeTest extends ToolkitApp {

    @Override
    protected Element render() {
        return panel("dotfile-java-tui",
                text("Hello from the Toolkit DSL + Panama backend").bold().cyan(),
                spacer(),
                text("press q to quit").dim()
            ).rounded()
             .onKeyEvent(e -> {
                 if (e.isChar('q')) { quit(); return EventResult.HANDLED; }
                 return EventResult.UNHANDLED;
             });
    }

    public void run() throws Exception {
        /* launch per the real API — e.g. super.run() / runner with Panama backend */
    }
}
```

Also verify here that the built-in `spinner()` element animates when added
temporarily — that confirms the tick mechanism for later phases.

## 1.6 Logging + resources

`src/main/resources/application.yml`:

```yaml
spring:
  main:
    web-application-type: none
    banner-mode: off
  threads:
    virtual:
      enabled: true
dotfile:
  data-dir-name: dotfile-rs
```

`logback-spring.xml`: single `FileAppender` → `debug.log`, pattern
`%d{HH:mm:ss.SSS} %-5level %logger{30}:%line - %msg%n`, root INFO, package
`io.github.sampongstarluck.dotfile` DEBUG. **No ConsoleAppender.**

Copy resources (paths relative to this folder):

| From | To |
|---|---|
| `../src/json/apps.json` | `src/main/resources/data/apps.json` |
| `../src/json/shells.json` | `src/main/resources/data/shells.json` |
| `../src/scripts/bash/main_profile.sh` | `src/main/resources/scripts/bash/main_profile.sh` |
| `../src/scripts/zsh/main_profile.zsh` | `src/main/resources/scripts/zsh/main_profile.zsh` |
| `../src/scripts/fish/main_profile.fish` | `src/main/resources/scripts/fish/main_profile.fish` |
| `../src/scripts/nu/main_profile.nu` | `src/main/resources/scripts/nu/main_profile.nu` |
| `../src/scripts/posh/main_profile.ps1` | `src/main/resources/scripts/posh/main_profile.ps1` |

`.gitignore`: `target/`, `debug.log`, `api-*.txt`, `.mise.local.toml`.

## Definition of Done (Phase 1)

- [ ] `mise install` provisions GraalVM 25 + Maven; `mise exec -- java -version` shows GraalVM 25
- [ ] `mise run dev` opens the fluent-DSL smoke TUI **through the Panama backend on Windows Terminal**; `q` exits; prompt restored intact
- [ ] Temporary `spinner()` element animates (tick mechanism confirmed), then removed
- [ ] No console output except the TUI; `debug.log` created
- [ ] All 7 resources copied; `application.yml` has `spring.threads.virtual.enabled=true`
- [ ] `api-toolkit.txt` / `api-widgets.txt` / `api-core.txt` / `api-panama.txt` generated and §1.3 checklist filled in with real names
- [ ] `config/` package exists with `AsyncConfig` + `AppProperties`
- [ ] Lombok processes on JDK 25: a `@Slf4j` class compiles and logs to `debug.log` (verify once, e.g. on `SmokeTest`)
