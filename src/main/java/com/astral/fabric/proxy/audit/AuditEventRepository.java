package com.astral.fabric.proxy.audit;

import org.springframework.data.jpa.repository.JpaRepository;

public interface AuditEventRepository extends JpaRepository<AuditEvent, Long> {
    @org.springframework.transaction.annotation.Transactional
    @org.springframework.data.jpa.repository.Modifying
    @org.springframework.data.jpa.repository.Query("delete from AuditEvent e where e.ts < :cutoff")
    int purgeBefore(@org.springframework.data.repository.query.Param("cutoff") java.time.Instant cutoff);
}
