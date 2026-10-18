package com.astral.main.security;

import org.springframework.beans.factory.annotation.Value;
import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.security.oauth2.client.registration.*;
import org.springframework.security.oauth2.core.*;

@Configuration
@ConditionalOnProperty(name = "astral.sso.enabled", havingValue = "true")
public class OidcClientConfiguration {
    @Bean ClientRegistrationRepository clientRegistrationRepository(
            @Value("${astral.sso.issuer}") String issuer,
            @Value("${astral.sso.client-id:grafana}") String clientId,
            @Value("${astral.sso.client-secret}") String secret) {
        if (secret.isBlank()) throw new IllegalStateException("ASTRAL_SSO_CLIENT_SECRET obrigatorio");
        issuer = issuer.replaceAll("/+$", "");
        // Rotas reais do AuthorizationServerConfig do auth-service; sem discovery no boot.
        return new InMemoryClientRegistrationRepository(ClientRegistration.withRegistrationId("auth-service")
                .clientId(clientId).clientSecret(secret)
                .clientAuthenticationMethod(ClientAuthenticationMethod.CLIENT_SECRET_BASIC)
                .authorizationGrantType(AuthorizationGrantType.AUTHORIZATION_CODE)
                .redirectUri("{baseUrl}/login/oauth2/code/{registrationId}")
                .scope("openid", "profile", "email", "groups")
                .authorizationUri(issuer + "/oauth2/authorize")
                .tokenUri(issuer + "/oauth2/token")
                .jwkSetUri(issuer + "/oauth2/jwks")
                .userInfoUri(issuer + "/userinfo")
                .userNameAttributeName("sub").issuerUri(issuer)
                .clientName("Auth Service").build());
    }
}
