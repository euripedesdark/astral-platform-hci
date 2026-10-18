package com.astral.main.security;

import org.springframework.security.core.GrantedAuthority;
import org.springframework.security.oauth2.core.oidc.OidcIdToken;
import org.springframework.security.oauth2.core.oidc.OidcUserInfo;
import org.springframework.security.oauth2.core.oidc.user.OidcUser;
import java.util.Collection;
import java.util.List;
import java.util.Map;

/** Mantem o principal e os grupos usados pelo ACL, com identidade validada pelo OIDC. */
public class AstralOidcUser extends AstralPrincipal implements OidcUser {
    private final OidcUser delegate;
    public AstralOidcUser(OidcUser delegate, String username, String provider,
                          Collection<? extends GrantedAuthority> roles, List<String> groups) {
        super(username, provider, roles, groups);
        this.delegate = delegate;
    }
    @Override public String getName() { return getUsername(); }
    @Override public Map<String, Object> getAttributes() { return delegate.getAttributes(); }
    @Override public Map<String, Object> getClaims() { return delegate.getClaims(); }
    @Override public OidcIdToken getIdToken() { return delegate.getIdToken(); }
    @Override public OidcUserInfo getUserInfo() { return delegate.getUserInfo(); }
}
