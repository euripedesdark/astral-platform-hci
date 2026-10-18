package com.astral.main.security;

import org.springframework.beans.factory.annotation.Value;
import org.springframework.security.core.authority.SimpleGrantedAuthority;
import org.springframework.security.oauth2.client.oidc.userinfo.OidcUserRequest;
import org.springframework.security.oauth2.client.oidc.userinfo.OidcUserService;
import org.springframework.security.oauth2.core.OAuth2AuthenticationException;
import org.springframework.security.oauth2.core.oidc.user.OidcUser;
import org.springframework.stereotype.Service;
import javax.naming.ldap.LdapName;
import java.util.ArrayList;
import java.util.List;

@Service
public class AstralOidcUserService extends OidcUserService {
    private final String adminGroup;
    public AstralOidcUserService(@Value("${astral.auth.ad.admin-group:Domain Admins}") String adminGroup) {
        this.adminGroup = adminGroup;
    }
    @Override public OidcUser loadUser(OidcUserRequest request) throws OAuth2AuthenticationException {
        return map(super.loadUser(request));
    }
    AstralOidcUser map(OidcUser user) {
        List<String> groups = user.getClaimAsStringList("groups");
        if (groups == null) groups = List.of();
        groups = groups.stream().map(AstralOidcUserService::groupName).toList();
        var roles = new ArrayList<SimpleGrantedAuthority>();
        roles.add(new SimpleGrantedAuthority("ROLE_ASTRAL_USER"));
        if (groups.stream().anyMatch(adminGroup::equalsIgnoreCase))
            roles.add(new SimpleGrantedAuthority("ROLE_ASTRAL_ADMIN"));
        String username = user.getPreferredUsername();
        if (username == null || username.isBlank()) username = user.getSubject();
        String provider = user.getClaimAsString("provider");
        return new AstralOidcUser(user, username, provider == null ? "OIDC" : provider, roles, groups);
    }
    private static String groupName(String group) {
        if (group == null) return "";
        try {
            for (var rdn : new LdapName(group).getRdns())
                if ("CN".equalsIgnoreCase(rdn.getType())) return String.valueOf(rdn.getValue());
        } catch (javax.naming.InvalidNameException ignored) { }
        return group;
    }
}
