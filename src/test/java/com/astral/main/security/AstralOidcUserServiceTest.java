package com.astral.main.security;

import org.junit.jupiter.api.Test;
import org.springframework.security.oauth2.core.oidc.OidcIdToken;
import org.springframework.security.oauth2.core.oidc.user.DefaultOidcUser;
import java.time.Instant;
import java.util.List;
import java.util.Map;
import static org.junit.jupiter.api.Assertions.*;

class AstralOidcUserServiceTest {
    private DefaultOidcUser user(List<String> groups) {
        return new DefaultOidcUser(List.of(), new OidcIdToken("signed-id-token", Instant.now(),
                Instant.now().plusSeconds(300), Map.of("sub", "stable-subject", "preferred_username", "alice",
                "provider", "AD", "groups", groups)));
    }
    @Test void mapsIdentityAndGroupsWithoutElevatingOrdinaryUsers() {
        var mapped = new AstralOidcUserService("Domain Admins").map(user(List.of("FW-Operadores")));
        assertEquals("alice", mapped.getName());
        assertEquals("stable-subject", mapped.getSubject());
        assertEquals("AD", mapped.getSource());
        assertEquals(List.of("FW-Operadores"), mapped.getAdGroups());
        assertTrue(mapped.getAuthorities().stream().anyMatch(a->a.getAuthority().equals("ROLE_ASTRAL_USER")));
        assertFalse(mapped.getAuthorities().stream().anyMatch(a->a.getAuthority().equals("ROLE_ASTRAL_ADMIN")));
    }
    @Test void onlyConfiguredAdminGroupGrantsAstralAdministration() {
        var service = new AstralOidcUserService("Domain Admins");
        var admin = service.map(user(List.of("CN=Domain Admins,OU=Groups,DC=example,DC=org")));
        assertTrue(admin.getAuthorities().stream().anyMatch(a->a.getAuthority().equals("ROLE_ASTRAL_ADMIN")));
        var other = service.map(user(List.of("GRP_DIRETORIA")));
        assertFalse(other.getAuthorities().stream().anyMatch(a->a.getAuthority().equals("ROLE_ASTRAL_ADMIN")));
    }
}
