package com.astral.main.controller;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

class ObservabilityControllerTest {
    @Test void usesExistingGrafanaAddressWithAstralDashboard() {
        String url = "https://shared.example/grafana/d/astral-observability";
        assertEquals(url, new ObservabilityController(url).dashboard().get("url"));
    }
    @Test void rejectsErpDashboardAndEmbeddedCredentials() {
        assertThrows(IllegalArgumentException.class,
                () -> new ObservabilityController("https://shared.example/grafana/d/bi-brasil-saas"));
        assertThrows(IllegalArgumentException.class,
                () -> new ObservabilityController("https://user:password@shared.example/d/astral-observability"));
    }
}
