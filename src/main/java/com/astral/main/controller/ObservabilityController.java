package com.astral.main.controller;

import org.springframework.beans.factory.annotation.Value;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RestController;
import java.net.URI;
import java.util.Map;

@RestController
public class ObservabilityController {
    private final String url;
    public ObservabilityController(@Value("${astral.grafana.dashboard-url:}") String url) {
        if (!url.isBlank()) {
            URI uri = URI.create(url);
            if (!java.util.Set.of("http", "https").contains(uri.getScheme())
                    || uri.getHost() == null || uri.getUserInfo() != null
                    || !uri.getPath().matches(".*/d/astral-observability(?:/[^/]*)?"))
                throw new IllegalArgumentException("Use a URL publica do dashboard astral-observability no Grafana existente");
        }
        this.url = url;
    }
    @GetMapping("/api/observability/dashboard")
    public Map<String,String> dashboard() { return Map.of("url", url, "uid", "astral-observability"); }
}
