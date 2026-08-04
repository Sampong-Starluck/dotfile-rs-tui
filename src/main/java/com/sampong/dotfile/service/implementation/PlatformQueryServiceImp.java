package com.sampong.dotfile.service.implementation;

import com.sampong.dotfile.model.PackageManager;
import com.sampong.dotfile.service.PackageManagerService;
import com.sampong.dotfile.service.PlatformQueryService;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.scheduling.annotation.Async;
import org.springframework.stereotype.Service;

import java.util.List;
import java.util.concurrent.CompletableFuture;

@Slf4j
@Service
@RequiredArgsConstructor
public class PlatformQueryServiceImp implements PlatformQueryService {

    private final PackageManagerService packageManagerService;

    @Override
    @Async
    public CompletableFuture<List<PackageManager>> detectManagers() {
        long start = System.nanoTime();
        List<PackageManager> managers = packageManagerService.detect();
        log.debug("startup: package manager detection took {}ms", (System.nanoTime() - start) / 1_000_000);
        return CompletableFuture.completedFuture(managers);
    }
}
