package io.github.sampongstarluck.dotfile;

import org.springframework.boot.Banner;
import org.springframework.boot.CommandLineRunner;
import org.springframework.boot.SpringApplication;
import org.springframework.boot.WebApplicationType;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.boot.context.properties.ConfigurationPropertiesScan;
import org.springframework.context.annotation.Bean;

import io.github.sampongstarluck.dotfile.ui.SmokeTest;

@SpringBootApplication
@ConfigurationPropertiesScan
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
        return args -> new SmokeTest().run();
    }
}
