package com.sampong.dotfile;

import org.springframework.boot.Banner;
import org.springframework.boot.CommandLineRunner;
import org.springframework.boot.SpringApplication;
import org.springframework.boot.WebApplicationType;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.boot.context.properties.ConfigurationPropertiesScan;
import org.springframework.context.annotation.Bean;

import com.sampong.dotfile.ui.TuiApp;

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
    CommandLineRunner tui(TuiApp app) {
        return args -> app.run();
    }
}
