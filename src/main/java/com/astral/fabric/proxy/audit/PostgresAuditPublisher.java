package com.astral.fabric.proxy.audit;

import com.fasterxml.jackson.databind.ObjectMapper;
import jakarta.annotation.PostConstruct;
import jakarta.annotation.PreDestroy;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardOpenOption;
import java.time.Instant;
import java.util.ArrayDeque;
import java.util.Deque;
import java.util.LinkedHashMap;
import java.util.Map;
import java.util.concurrent.ArrayBlockingQueue;
import java.util.concurrent.BlockingQueue;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicLong;

/** Auditoria assincrona no PostgreSQL compartilhado, consultada pelo Grafana. */
@Component
public class PostgresAuditPublisher {

    private static final Logger log = LoggerFactory.getLogger(PostgresAuditPublisher.class);

    private final ObjectMapper json;
    private final AuditEventRepository repository;
    private final boolean habilitado;
    private final Path fallback;
    private final BlockingQueue<Map<String, Object>> fila;
    private final AtomicLong publicados = new AtomicLong();
    private final AtomicLong descartados = new AtomicLong();
    private final AtomicLong fallbacks = new AtomicLong();

    private Thread trabalhador;
    private volatile boolean rodando;

    public PostgresAuditPublisher(ObjectMapper json, AuditEventRepository repository,
            @Value("${astral.audit.enabled:true}") boolean habilitado,
            @Value("${astral.audit.queue-capacity:8192}") int capacidade,
            @Value("${astral.audit.fallback-file:${java.io.tmpdir}/astral/audit-fallback.jsonl}") String fallback) {
        this.json = json;
        this.repository = repository;
        this.habilitado = habilitado;
        this.fila = new ArrayBlockingQueue<>(Math.max(64, capacidade));
        this.fallback = Path.of(fallback);
    }

    @PostConstruct
    void sobe() {
        try { Files.createDirectories(fallback.toAbsolutePath().getParent()); }
        catch (IOException e) { log.warn("audit: diretorio de fallback indisponivel", e); }
        if (!habilitado) return;
        rodando = true;
        trabalhador = Thread.ofVirtual().name("postgres-audit").start(this::loop);
    }

    @PreDestroy
    void desce() {
        rodando = false;
        if (trabalhador != null) {
            trabalhador.interrupt();
            try { trabalhador.join(5000); }
            catch (InterruptedException e) { Thread.currentThread().interrupt(); }
        }
        drena();
    }

    /**
     * Enfileira o evento. Nunca bloqueia, nunca lanca: o chamador e' o caminho
     * de decisao de ACL e nao pode depender do destino do log.
     */
    public void publicar(String facility, String shortMessage, Map<String, Object> campos) {
        Map<String, Object> evento = new LinkedHashMap<>();
        evento.put("facility", facility);
        evento.put("short_message", shortMessage);
        evento.put("timestamp", Instant.now().toString());
        evento.putAll(campos);
        if (!habilitado || !fila.offer(evento)) {
            descartados.incrementAndGet();
            gravaFallback(evento);
        }
    }

    /** Variante ja' serializada, para nao reconstruir o mapa no chamador. */
    public void audit(String usuario, String modulo, String acao, String alvo, String resultado, Map<String, Object> extra) {
        Map<String, Object> campos = new LinkedHashMap<>();
        campos.put("usuario", usuario);
        campos.put("modulo", modulo);
        campos.put("acao", acao);
        campos.put("alvo", alvo);
        campos.put("resultado", resultado);
        campos.put("evento", "astral.audit");
        if (extra != null) campos.putAll(extra);
        publicar("astral-audit", acao + " " + alvo + " -> " + resultado, campos);
    }

    private void loop() {
        while (rodando) {
            Map<String, Object> evento;
            try {
                evento = fila.poll(500, TimeUnit.MILLISECONDS);
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
                break;
            }
            if (evento == null) continue;
            persiste(evento);
        }
    }

    private void persiste(Map<String, Object> evento) {
        try {
            repository.saveAndFlush(new AuditEvent(Instant.parse(str(evento.get("timestamp"))),
                    str(evento.get("facility")), str(evento.get("short_message")),
                    json.writeValueAsString(evento)));
            publicados.incrementAndGet();
        } catch (Exception e) {
            fallbacks.incrementAndGet();
            gravaFallback(evento);
        }
    }

    /**
     * Fallback: JSON de uma linha, append. e' a prova de que "auditoria sobre
     * conveniencia" e' regra e nao slogan -- se o PostgreSQL sumir, a trilha fica.
     */
    private void gravaFallback(Map<String, Object> evento) {
        try {
            String linha = json.writeValueAsString(evento) + System.lineSeparator();
            Files.writeString(fallback, linha, StandardCharsets.UTF_8,
                    StandardOpenOption.CREATE, StandardOpenOption.APPEND);
        } catch (IOException e) {
            log.error("audit: fallback tambem falhou, evento perdido: {}", e.getMessage());
        }
    }

    public Map<String, Object> metricas() {
        Map<String, Object> m = new LinkedHashMap<>();
        m.put("publicados", publicados.get());
        m.put("descartados", descartados.get());
        m.put("fallback", fallbacks.get());
        m.put("naFila", fila.size());
        m.put("alvo", "PostgreSQL/public.observability_audit_event");
        m.put("habilitado", habilitado);
        return m;
    }

    /** Drena o que restou na fila. Chamado no shutdown para nao perder auditoria. */
    public int drena() {
        int n = 0;
        Deque<Map<String, Object>> resto = new ArrayDeque<>();
        fila.drainTo(resto);
        for (Map<String, Object> e : resto) {
            persiste(e);
            n++;
        }
        return n;
    }

    private static String str(Object o) {
        return o == null ? "" : String.valueOf(o);
    }

    /** Caminho do arquivo de fallback -- exposto ao endpoint de saude. */
    public Path arquivoFallback() {
        return fallback;
    }
}
