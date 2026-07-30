package io.github.sampongstarluck.dotfile.config;

import org.springframework.boot.context.properties.ConfigurationProperties;

@ConfigurationProperties(prefix = "dotfile")
public record AppProperties(String dataDirName) {
    public AppProperties {
        if (dataDirName == null || dataDirName.isBlank()) dataDirName = "dotfile-rs";
    }
}
