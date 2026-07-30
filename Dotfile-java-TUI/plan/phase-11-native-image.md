# Phase 11 — GraalVM native-image (best-effort)

**Goal:** a native `dotfile-java-tui.exe` via GraalVM native-image.
**This phase is allowed to fail on this machine** — the original reason for
the migration is a Windows driver/security issue that blocks native linking
toolchains. If `native-image` linking fails for environment reasons, document
it and stop; the fat jar from Phase 10 remains the shipped product.

## 11.1 Prerequisites (Windows)

- GraalVM 25 already provisioned by mise (Phase 1).
- native-image needs the MSVC toolchain:
  `winget install Microsoft.VisualStudio.2022.BuildTools` with the
  "Desktop development with C++" workload, then build from a shell where
  `cl.exe` is on PATH (or let native-image locate VS itself — it does on
  recent versions).

## 11.2 pom: the `native` profile

Add to `pom.xml` (Spring Boot 4.1 parent already manages the plugin version):

```xml
<profiles>
  <profile>
    <id>native</id>
    <build>
      <plugins>
        <plugin>
          <groupId>org.graalvm.buildtools</groupId>
          <artifactId>native-maven-plugin</artifactId>
          <configuration>
            <buildArgs>
              <buildArg>--enable-native-access=ALL-UNNAMED</buildArg>
              <buildArg>-H:+ForeignAPISupport</buildArg> <!-- only if the FFM backend needs it; check TamboUI docs -->
            </buildArgs>
          </configuration>
        </plugin>
      </plugins>
    </build>
  </profile>
</profiles>
```

Spring Boot's AOT engine runs automatically under this profile
(`mise run native` → `mvn -Pnative native:compile`).

## 11.3 Reachability metadata

- TamboUI advertises GraalVM native-image support — check
  https://tamboui.dev/ docs / the GitHub repo for shipped reachability
  metadata or required flags for the **Panama backend** (FFM downcalls must
  be registered at build time on native-image).
- Jackson-mapped records (`AppEntry`, `ShellEntry`, …) need reflection
  entries: Spring AOT handles beans it can see; for the `TypeReference`
  reads, add `@RegisterReflectionForBinding({AppEntry.class, AppSection.class,
  ShellEntry.class, ShellsFile.class, DotfileConfig.class, SearchResult.class})`
  on a `config/NativeHints.java` class.
- If something is still missing at runtime, run the app once on JVM with
  `-agentlib:native-image-agent=config-output-dir=src/main/resources/META-INF/native-image`
  and exercise every screen (acceptance script §10.5), then rebuild.

## 11.4 Verify

- `mise run native` → binary in `target/`.
- Run the Phase 10 acceptance script against the native binary.
- Startup should be near-instant; record binary size + startup time in
  README.

## Definition of Done (Phase 11)

- [ ] `mise run native` completes **or** the exact failure (toolchain/driver)
      is documented in FEATURE-PARITY.md with the error output
- [ ] If built: acceptance script passes on the native exe; README updated
      with native build instructions
