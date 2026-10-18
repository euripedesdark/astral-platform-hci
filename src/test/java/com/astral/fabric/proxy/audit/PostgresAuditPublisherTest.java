package com.astral.fabric.proxy.audit;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Map;
import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.*;

class PostgresAuditPublisherTest {
    @TempDir Path dir;

    @Test void persistsQueuedAuditThroughJpa() {
        var repo = mock(AuditEventRepository.class);
        var publisher = new PostgresAuditPublisher(new ObjectMapper(), repo, true, 64, dir.resolve("audit.jsonl").toString());
        publisher.audit("alice", "proxy", "check", "example.org", "DENY", Map.of());
        assertEquals(1, publisher.drena());
        verify(repo).saveAndFlush(any(AuditEvent.class));
        assertEquals(1L, publisher.metricas().get("publicados"));
    }

    @Test void databaseFailurePreservesAuditInJsonl() throws Exception {
        var repo = mock(AuditEventRepository.class);
        when(repo.saveAndFlush(any())).thenThrow(new IllegalStateException("database unavailable"));
        var file = dir.resolve("audit.jsonl");
        var publisher = new PostgresAuditPublisher(new ObjectMapper(), repo, true, 64, file.toString());
        publisher.audit("alice", "proxy", "check", "example.org", "DENY", Map.of());
        publisher.drena();
        var event = new ObjectMapper().readTree(Files.readString(file));
        assertEquals("alice", event.get("usuario").asText());
        assertEquals("DENY", event.get("resultado").asText());
        assertEquals(1L, publisher.metricas().get("fallback"));
    }
}
