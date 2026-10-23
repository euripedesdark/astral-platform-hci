import React, { useState, useRef, useEffect } from 'react';
import { Card } from 'primereact/card';
import { Dialog } from 'primereact/dialog';
import { Button } from 'primereact/button';
import { InputText } from 'primereact/inputtext';
import { api } from '../../../shared/services/api';

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
    url: '/ad/',
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
    id: 'terminal',
    title: 'Terminal do Sistema',
    subtitle: 'Console Web Shell',
    desc: 'Terminal interativo para intervenções operacionais (Exclusivo para contas Samba AD e Linux Sudo).',
    img: '/images/terminal.jpeg',
    url: '#terminal',
    badge: 'Terminal'
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

interface OverviewProps {
  user?: any;
}

interface TermLine {
  cmd?: string;
  output?: string;
  isError?: boolean;
}

export default function OverviewGo({ user }: OverviewProps) {
  const [showTerminal, setShowTerminal] = useState(false);
  const [deniedDialog, setDeniedDialog] = useState(false);
  const [termLines, setTermLines] = useState<TermLine[]>([
    { output: 'Astral Linux Shell [srvcloud] v2.0 - Intervenção Operacional' },
    { output: 'Digite qualquer comando do sistema (ex: systemctl status, samba-tool, ip a, df -h)' }
  ]);
  const [command, setCommand] = useState('');
  const [executing, setExecuting] = useState(false);
  const terminalEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (showTerminal) {
      terminalEndRef.current?.scrollIntoView({ behavior: 'smooth' });
    }
  }, [termLines, showTerminal]);

  const handleLaunch = (mod: AstralModule) => {
    if (mod.id === 'terminal') {
      const userSource = (user?.source || '').toUpperCase();
      if (userSource === 'POSTGRES') {
        setDeniedDialog(true);
        return;
      }
      setShowTerminal(true);
      return;
    }

    const token = localStorage.getItem('astral_token') || sessionStorage.getItem('token');
    let targetUrl = mod.url;
    if (token && !targetUrl.startsWith('#') && !targetUrl.startsWith('http')) {
      const sep = targetUrl.includes('?') ? '&' : '?';
      targetUrl = `${targetUrl}${sep}token=${encodeURIComponent(token)}`;
    }

    if (mod.target === '_blank') {
      window.open(targetUrl, '_blank');
    } else {
      window.location.href = targetUrl;
    }
  };

  const executeCommand = async () => {
    const trimmed = command.trim();
    if (!trimmed || executing) return;

    if (trimmed === 'clear') {
      setTermLines([]);
      setCommand('');
      return;
    }

    const currentCmd = trimmed;
    setCommand('');
    setExecuting(true);
    setTermLines((prev) => [...prev, { cmd: currentCmd }]);

    try {
      const res = await api<{
        success: boolean;
        stdout?: string;
        stderr?: string;
        error?: string;
        exit_code?: number;
      }>('/api/terminal/exec', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ command: currentCmd })
      });

      if (res.success) {
        const out = (res.stdout || '') + (res.stderr ? (res.stdout ? '\n' : '') + res.stderr : '');
        setTermLines((prev) => [...prev, { output: out || '(Comando executado sem retorno)' }]);
      } else {
        setTermLines((prev) => [...prev, { output: res.error || 'Erro na execução', isError: true }]);
      }
    } catch (err: any) {
      setTermLines((prev) => [
        ...prev,
        { output: err.message || 'Falha de comunicação com o servidor', isError: true }
      ]);
    } finally {
      setExecuting(false);
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
              <span className={`p-badge ${m.id === 'terminal' ? 'p-badge-warning' : 'p-badge-info'}`} style={{ marginTop: '8px', marginBottom: '8px' }}>
                {m.badge}
              </span>
              <p className="muted" style={{ textAlign: 'center', fontSize: '0.9rem' }}>
                {m.desc}
              </p>
            </div>
          </Card>
        ))}
      </div>

      {/* Terminal Modal Dialog */}
      <Dialog
        header={`Terminal Operacional — [${user?.username || 'admin'}@srvcloud] (${user?.source || 'AD'})`}
        visible={showTerminal}
        style={{ width: '85vw', maxWidth: '1000px' }}
        onHide={() => setShowTerminal(false)}
        footer={
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', width: '100%' }}>
            <span style={{ fontSize: '0.8rem', color: '#888' }}>
              Exclusivo: Samba Active Directory &bull; Linux Sudo
            </span>
            <div>
              <Button label="Limpar" icon="pi pi-trash" text onClick={() => setTermLines([])} />
              <Button label="Fechar" icon="pi pi-times" onClick={() => setShowTerminal(false)} />
            </div>
          </div>
        }
      >
        <div
          style={{
            backgroundColor: '#0a0d14',
            color: '#38ef7d',
            fontFamily: 'monospace',
            padding: '16px',
            borderRadius: '8px',
            height: '420px',
            overflowY: 'auto',
            border: '1px solid #1f293d',
            display: 'flex',
            flexDirection: 'column'
          }}
        >
          <div style={{ flex: 1 }}>
            {termLines.map((line, idx) => (
              <div key={idx} style={{ marginBottom: '6px' }}>
                {line.cmd && (
                  <div style={{ color: '#63b3ed', fontWeight: 'bold' }}>
                    [{user?.username || 'admin'}@srvcloud ~]$ <span style={{ color: '#fff' }}>{line.cmd}</span>
                  </div>
                )}
                {line.output && (
                  <pre
                    style={{
                      margin: 0,
                      whiteSpace: 'pre-wrap',
                      wordBreak: 'break-all',
                      color: line.isError ? '#fc8181' : '#a0aec0',
                      fontSize: '0.88rem'
                    }}
                  >
                    {line.output}
                  </pre>
                )}
              </div>
            ))}
            <div ref={terminalEndRef} />
          </div>

          <div style={{ display: 'flex', alignItems: 'center', marginTop: '12px', borderTop: '1px solid #1f293d', paddingTop: '8px' }}>
            <span style={{ color: '#63b3ed', fontWeight: 'bold', marginRight: '8px' }}>
              [{user?.username || 'admin'}@srvcloud ~]$
            </span>
            <InputText
              value={command}
              onChange={(e) => setCommand(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') executeCommand();
              }}
              disabled={executing}
              placeholder="Digite um comando..."
              style={{
                flex: 1,
                backgroundColor: '#111827',
                color: '#fff',
                border: '1px solid #374151',
                fontFamily: 'monospace'
              }}
              autoFocus
            />
            <Button
              icon={executing ? 'pi pi-spin pi-spinner' : 'pi pi-arrow-right'}
              onClick={executeCommand}
              disabled={executing || !command.trim()}
              style={{ marginLeft: '8px' }}
            />
          </div>
        </div>
      </Dialog>

      {/* Permission Denied Modal */}
      <Dialog
        header="Acesso ao Terminal Negado"
        visible={deniedDialog}
        style={{ width: '450px' }}
        onHide={() => setDeniedDialog(false)}
        footer={<Button label="Entendido" icon="pi pi-check" onClick={() => setDeniedDialog(false)} />}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
          <i className="pi pi-exclamation-triangle" style={{ fontSize: '2.5rem', color: '#e53e3e' }}></i>
          <div>
            <h4 style={{ margin: '0 0 8px 0', color: '#fff' }}>Permissão Insuficiente</h4>
            <p style={{ margin: 0, color: '#a0aec0', fontSize: '0.9rem', lineHeight: '1.4' }}>
              O terminal do sistema operacional é restrito exclusivamente a administradores do <strong>Samba Active Directory</strong> ou operadores locais <strong>Linux (sudo/root)</strong>.
            </p>
            <p style={{ margin: '8px 0 0 0', color: '#fc8181', fontSize: '0.85rem' }}>
              Superusuários do PostgreSQL possuem acesso técnico restrito ao banco de dados e não têm autorização para invocar o shell do host.
            </p>
          </div>
        </div>
      </Dialog>
    </div>
  );
}
