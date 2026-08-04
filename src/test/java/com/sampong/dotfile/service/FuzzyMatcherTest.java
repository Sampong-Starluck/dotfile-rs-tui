package com.sampong.dotfile.service;

import org.junit.jupiter.api.Test;

import static org.assertj.core.api.Assertions.assertThat;

class FuzzyMatcherTest {

    @Test
    void emptyQueryMatchesEverything() {
        assertThat(FuzzyMatcher.matches("", "anything")).isTrue();
    }

    @Test
    void matchesInOrderSubsequenceCaseInsensitive() {
        assertThat(FuzzyMatcher.matches("dbv", "DBeaver")).isTrue();
        assertThat(FuzzyMatcher.matches("obs", "OBS Studio")).isTrue();
        assertThat(FuzzyMatcher.matches("zed", "Zed")).isTrue();
    }

    @Test
    void rejectsOutOfOrderOrMissingCharacters() {
        assertThat(FuzzyMatcher.matches("vbd", "DBeaver")).isFalse();
        assertThat(FuzzyMatcher.matches("xyz", "OBS Studio")).isFalse();
    }

    @Test
    void rejectsQueryLongerThanCandidateMatch() {
        assertThat(FuzzyMatcher.matches("obsidianx", "Obsidian")).isFalse();
    }
}
