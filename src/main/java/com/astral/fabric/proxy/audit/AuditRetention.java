package com.astral.fabric.proxy.audit;

import org.springframework.beans.factory.annotation.Value;
import org.springframework.scheduling.annotation.Scheduled;
import org.springframework.stereotype.Component;
import java.time.Instant;
import java.time.temporal.ChronoUnit;

@Component
public class AuditRetention {
    private final AuditEventRepository repository;
    private final long days;
    public AuditRetention(AuditEventRepository repository, @Value("${astral.audit.retention-days:30}") long days) {
        if (days < 1) throw new IllegalArgumentException("audit.retention-days deve ser >= 1");
        this.repository = repository; this.days = days;
    }
    @Scheduled(fixedDelayString = "${astral.audit.retention-interval-ms:3600000}", initialDelay = 60000)
    public void cleanup() { repository.purgeBefore(Instant.now().minus(days, ChronoUnit.DAYS)); }
}
