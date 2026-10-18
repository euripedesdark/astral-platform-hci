package com.astral.fabric.proxy.audit;

import jakarta.persistence.*;
import java.time.Instant;

@Entity
@Table(name = "observability_audit_event", schema = "public")
public class AuditEvent {
    @Id @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;
    @Column(nullable = false) private Instant ts;
    @Column(nullable = false, columnDefinition = "text") private String facility;
    @Column(nullable = false, columnDefinition = "text") private String message;
    @Column(nullable = false, columnDefinition = "text") private String payload;
    protected AuditEvent() {}
    public AuditEvent(Instant ts, String facility, String message, String payload) {
        this.ts = ts; this.facility = facility; this.message = message; this.payload = payload;
    }
}
