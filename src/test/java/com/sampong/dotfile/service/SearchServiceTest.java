package com.sampong.dotfile.service;

import com.sampong.dotfile.service.implementation.SearchServiceImp;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.assertj.core.api.Assertions.assertThat;

class SearchServiceTest {

    private SearchService service;

    @BeforeEach
    void setUp() {
        service = new SearchServiceImp();
    }

    @Test
    void searchCommandForChocoIncludesLimitOutput() {
        assertThat(service.searchCommand("choco", "git"))
                .isEqualTo(new SearchService.Cmd("choco", List.of("search", "git", "--limit-output")));
    }

    @Test
    void listCommandForXbpsUsesXbpsQuery() {
        assertThat(service.listCommand("xbps-install"))
                .isEqualTo(new SearchService.Cmd("xbps-query", List.of("-l")));
    }
}
