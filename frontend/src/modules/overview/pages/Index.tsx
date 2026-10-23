import React from 'react';
import {Card} from 'primereact/card';

export interface AstralModule {
  id: string;
  title: string;
  subtitle: string;
  desc: string;
  img: string;
  url: string;
  badge?: string;
  target?: string;
}

export const ASTRAL_MODULES: AstralModule[] = [
  {
    id: 'internet-security',
    title: 'Astral Internet Security',
    subtitle: 'Firewall & Gateway',
    desc: 'Firewall de borda, regras de tráfego, NAT, Masquerade, rate limiting e modo pânico.',
    img: '/images/firewall.png',
    url: '/firewall/',
    badge: 'Firewall'
  },
  {
    id: 'traffic-guard',
    title: 'Astral Traffic Guard',
    subtitle: 'Proxy & Content Filter',
    desc: 'Proxy reverso e direto, controle de acesso web (ACL), políticas de navegação e inspeção.',
    img: '/images/proxy_system.png',
    url: '/proxy/',
    badge: 'Proxy'
  },
  {
    id: 'dbm',
    title: 'Astral DBM',
    subtitle: 'Database Manager',
    desc: 'Gerenciador avançado de bancos de dados PostgreSQL, schemas, queries e backups.',
    img: '/images/postgresql_admin.png',
    url: '/dbm/',
    badge: 'Banco'
  },
  {
    id: 'erp',
    title: 'Astral Saas ERP',
    subtitle: 'Gestão Empresarial Integrada',
    desc: 'ERP completo multiempresa, fiscal, financeiro, suprimentos, vendas e serviços.',
    img: '/images/web_server_admin.png',
    url: '/erp/',
    badge: 'ERP'
  },
  {
    id: 'ad-dns',
    title: 'Astral AD & DNS',
    subtitle: 'Samba AD DC & DNS Híbrido',
    desc: 'Controlador de domínio corporativo Samba 4, autenticação Kerberos/LDAP e resolução DNS.',
    img: '/images/domain_controllers.png',
    url: '/ad-dns/',
    badge: 'Identidade'
  },
  {
    id: 'network',
    title: 'Astral Network',
    subtitle: 'Rede & DHCP Server',
    desc: 'Configuração de interfaces físicas e virtuais, escopos DHCP, leases e roteamento.',
    img: '/images/network_config_vlan.png',
    url: '/network/',
    badge: 'Rede'
  },
  {
    id: 'observability',
    title: 'Observabilidade & Auditoria',
    subtitle: 'Métricas & Grafana BI',
    desc: 'Monitoramento em tempo real de CPU, memória, I/O e trilha de auditoria do ecossistema.',
    img: '/images/storage.png',
    url: 'http://192.168.2.10:3000/grafana',
    badge: 'Monitoramento',
    target: '_blank'
  }
];

export default function OverviewGo() {
  const handleLaunch = (mod: AstralModule) => {
    if (mod.target === '_blank') {
      window.open(mod.url, '_blank');
    } else {
      window.location.href = mod.url;
    }
  };

  return (
    <div>
      <div className="grid">
        {ASTRAL_MODULES.map((m) => (
          <Card
            key={m.id}
            title={m.title}
            subTitle={m.subtitle}
            className="modcard"
            style={{ cursor: 'pointer' }}
            onClick={() => handleLaunch(m)}
          >
            <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
              <img
                src={m.img}
                alt={m.title}
                className="modimg"
                onError={(e) => {
                  (e.currentTarget as HTMLElement).style.display = 'none';
                }}
              />
              <span className="p-badge p-badge-info" style={{ marginTop: '8px', marginBottom: '8px' }}>
                {m.badge}
              </span>
              <p className="muted" style={{ textAlign: 'center', fontSize: '0.9rem' }}>
                {m.desc}
              </p>
            </div>
          </Card>
        ))}
      </div>
    </div>
  );
}
