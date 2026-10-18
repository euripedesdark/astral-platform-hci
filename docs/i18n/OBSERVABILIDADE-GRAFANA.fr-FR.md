# Astral sur le Grafana existant du BI

Astral réutilise **la même instance Grafana sur le port 3000** déjà utilisée par
le module BI du Brasil SaaS ERP, y compris OAuth, URL publique, cookies et base
de données de métadonnées existante. Il n'installe pas un autre Grafana et ne
reconfigure pas l'existant.

La carte Observabilité ouvre un iframe dans la même fenêtre avec son propre
tableau de bord `astral-observability` (audit, CPU et mémoire). La source
`astral-postgres` interroge la base PostgreSQL d'Astral, `astral` par défaut, à
l'aide d'un rôle de lecture exclusif. Le tableau de bord, la source de données
et les données du BI de l'ERP sont préservés ; la source de données d'Astral
n'est jamais marquée par défaut.

## Réutiliser Grafana

1. Déployer/redémarrer Astral pour appliquer la migration V101 d'audit.
2. Fournir `ASTRAL_GRAFANA_URL` avec l'**URL publique déjà utilisée par le BI**,
   y compris son préfixe actuel (ex. `http://192.168.2.10:3000/grafana`). Ne
   pas modifier `root_url`, issuer ni callback de Grafana pour accommoder Astral.
3. Fournir `ASTRAL_GRAFANA_SERVICE_TOKEN` d'un service account autorisé à
   provisionner une source de données et un tableau de bord dans l'organisation
   correcte. Ce token n'est utilisé que par le script, jamais par le navigateur.
   Exécuter sur l'hôte PostgreSQL :
   `sudo --preserve-env=ASTRAL_GRAFANA_URL,ASTRAL_GRAFANA_SERVICE_TOKEN bash scripts/install-grafana.sh`.
   Pour une autre base, préserver aussi `ASTRAL_DB_NAME`.
4. Configurer dans l'environnement **d'Astral** `ASTRAL_GRAFANA_DASHBOARD_URL`
   avec l'URL indiquée par le script, se terminant par `/d/astral-observability`,
   et redémarrer uniquement Astral. Le navigateur doit pouvoir atteindre cette
   URL ; un portail HTTPS exige que Grafana soit aussi publié en HTTPS. Réutiliser
   l'accès public existant.

Le script n'appelle que les API de dossiers, sources de données et tableaux de
bord et ajoute un rôle de lecture sur la base Astral. Il n'écrit pas dans
`/etc/grafana`, n'installe pas de paquets, ne change pas l'unité systemd de
Grafana, ne redémarre pas Grafana, ne change pas OAuth, ne change pas la base
de métadonnées et ne modifie pas les ressources de l'ERP. Des UID conflictuels
en dehors des ressources Astral provoquent une erreur. Le secret du rôle Astral
reste root-only dans `/etc/astral/grafana/reader.env` et est réutilisé lors des
réexécutions.

Les fichiers `deploy/grafana`, `scripts/bi/instalar_grafana_bi.sh` et le
composant `GrafanaBiDashboard` de l'ERP sont la référence de l'intégration
existante. La documentation principale mentionne encore LDAP dans des fichiers
anciens ; la configuration OAuth réellement en cours d'exécution est préservée
intégralement, sans tenter de la déduire ni de la remplacer.

## Login d'Astral dans l'auth-service

Le login OIDC est configuré **uniquement dans Astral**. Le script
`configure-grafana-sso.sh` a besoin de `ASTRAL_PUBLIC_URL`,
`ASTRAL_SSO_ISSUER` et `ASTRAL_SSO_CLIENT_SECRET` (et éventuellement
`ASTRAL_SSO_CLIENT_ID`). Il crée `/etc/astral/sso.env` et un drop-in pour
`astral-platform.service` uniquement.

Lors de l'enregistrement du client dans l'auth-service, préserver les callbacks
existants et ajouter seulement
`https://ORIGINE-D-ASTRAL/login/oauth2/code/auth-service`. Le client
configurable actuel peut être réutilisé ; aucun callback de Grafana n'a besoin de
changer. La configuration utilise les endpoints réels `/oauth2/authorize`,
`/oauth2/token`, `/oauth2/jwks` et `/userinfo`.

Avec SSO activé, l'entrée normale d'Astral redirige vers le fournisseur et crée
sa session centrale ; le Grafana déjà configuré réutilise cette session. Le
formulaire local d'Astral ne crée pas de session chez le fournisseur et
n'équivaut pas au SSO. La connexion locale de contingence reste à
`/app/?local=1`. La déconnexion d'Astral met fin uniquement à sa session
locale. Les groupes du fournisseur préservent l'ACL et seul le groupe admin
configuré accorde l'administration d'Astral. Le cookie `ASTRAL_SESSION` évite
la collision avec la session de l'auth-service.

Les règles d'embedding/cookies déjà établies pour le BI doivent aussi accepter
l'origine d'Astral. Utiliser le même site et schéma pour le portail, Grafana et
l'issuer lorsque le flux OAuth a lieu dans un iframe ; le blocage des cookies
tiers peut empêcher le SSO s'ils sont sur des sites différents. Aucune exception
navigateur ni changement global d'authentification n'est appliqué automatiquement.

## Données

Audit via JPA dans `public.observability_audit_event`, avec une rétention par
défaut de 30 jours (`ASTRAL_AUDIT_RETENTION_DAYS`), un nettoyage horaire et un
fallback JSONL dans `/var/log/astral/audit-fallback.jsonl`. Il ne réimporte pas
le JSONL automatiquement ni ne migre l'historique Graylog. Les données Astral
restent dans le PostgreSQL propre à Astral ; la base de métadonnées Grafana
existante reste où elle est.