import React, { useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Dropdown } from 'primereact/dropdown';
import { Button } from 'primereact/button';
import forge from 'node-forge';
import i18n, { supportedLanguages } from '../../i18n';
import { api, login, me } from '../services/api';
import DonationModal from './DonationModal';

const HEXA = (
  <svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
    <polygon points="50,5 95,25 95,75 50,95 5,75 5,25" fill="none" stroke="#00d2ff" strokeWidth="4" />
    <path d="M50,25 L75,70 L65,70 L50,40 L35,70 L25,70 Z" fill="#00d2ff" />
  </svg>
);
const USERI = (
  <svg className="input-icon" viewBox="0 0 24 24">
    <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"></path>
    <circle cx="12" cy="7" r="4"></circle>
  </svg>
);
const LOCKI = (
  <svg className="input-icon" viewBox="0 0 24 24">
    <rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect>
    <path d="M7 11V7a5 5 0 0 1 10 0v4"></path>
  </svg>
);

export default function Login({ onAuth }: { onAuth: (data: any) => void }) {
  const { t } = useTranslation();
  const [u, setU] = useState('');
  const [p, setP] = useState('');
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState('');
  const [authMode, setAuthMode] = useState<'password' | 'cert'>('password');
  const [certType, setCertType] = useState<'manual' | 'browser'>('manual');
  const [certFile, setCertFile] = useState<File | null>(null);
  const [certPass, setCertPass] = useState('');
  const [showDonation, setShowDonation] = useState(false);
  const flying = useRef(false);

  async function handleLoginSuccess(data: any) {
    if (!(data && (data.token || data.authenticated === true))) throw new Error('bad-response');
    if (data.token) {
      document.cookie = 'astral_token=' + encodeURIComponent(data.token) + '; path=/';
      document.cookie = 'sso_token=' + encodeURIComponent(data.token) + '; path=/';
      try {
        localStorage.setItem('astral_token', data.token);
      } catch (_) {}
    }
    await me();
    onAuth(data);
  }

  async function goPassword(e: React.FormEvent) {
    e.preventDefault();
    if (flying.current) return;
    flying.current = true;
    setBusy(true);
    setErr('');
    try {
      const data = await login(u, p);
      await handleLoginSuccess(data);
    } catch (ex) {
      setErr(t('login.invalidCredentials'));
    } finally {
      flying.current = false;
      setBusy(false);
    }
  }

  async function goManualCert(e: React.FormEvent) {
    e.preventDefault();
    if (!certFile) {
      setErr('Selecione um arquivo de certificado digital (.p12 ou .pfx).');
      return;
    }
    if (flying.current) return;
    flying.current = true;
    setBusy(true);
    setErr('');

    try {
      // 1. Read file as ArrayBuffer
      const buffer = await certFile.arrayBuffer();
      const bytes = new Uint8Array(buffer);
      let binaryStr = '';
      for (let i = 0; i < bytes.length; i++) {
        binaryStr += String.fromCharCode(bytes[i]);
      }

      // 2. Parse PKCS#12 in browser RAM
      const p12Asn1 = forge.asn1.fromDer(binaryStr);
      const p12 = forge.pkcs12.pkcs12FromAsn1(p12Asn1, certPass);

      // 3. Extract Certificate Bag
      const certBags = p12.getBags({ bagType: forge.pki.oids.certBag });
      const certBag = certBags[forge.pki.oids.certBag]?.[0];
      if (!certBag || !certBag.cert) {
        throw new Error('Nenhum certificado X.509 encontrado no arquivo .p12/.pfx informado.');
      }
      const certPem = forge.pki.certificateToPem(certBag.cert);

      // 4. Extract Key Bag
      const keyBags = p12.getBags({ bagType: forge.pki.oids.pkcs8ShroudedKeyBag });
      let keyBag = keyBags[forge.pki.oids.pkcs8ShroudedKeyBag]?.[0];
      if (!keyBag) {
        const rawKeyBags = p12.getBags({ bagType: forge.pki.oids.keyBag });
        keyBag = rawKeyBags[forge.pki.oids.keyBag]?.[0];
      }
      if (!keyBag || !keyBag.key) {
        throw new Error('Chave privada correspondente não encontrada no arquivo .p12/.pfx.');
      }

      // 5. Fetch challenge nonce from backend
      const challRes = await fetch('/api/auth/cert/challenge');
      if (!challRes.ok) throw new Error('Falha ao obter desafio criptográfico.');
      const { challenge } = await challRes.json();

      // 6. Sign challenge locally using the private key (SHA256)
      const md = forge.md.sha256.create();
      md.update(challenge, 'utf8');
      const signatureBytes = keyBag.key.sign(md);
      const signatureBase64 = forge.util.encode64(signatureBytes);

      // 7. Verify at backend (Notice: private key NEVER leaves client browser!)
      const verifyRes = await fetch('/api/auth/cert/verify-manual', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          certificate: certPem,
          challenge,
          signature: signatureBase64
        })
      });

      if (!verifyRes.ok) {
        const errJson = await verifyRes.json().catch(() => ({}));
        throw new Error(errJson.message || 'Falha na validação do certificado pelo servidor.');
      }

      const authData = await verifyRes.json();
      await handleLoginSuccess(authData);
    } catch (err: any) {
      setErr(err?.message || 'Erro ao processar certificado digital.');
    } finally {
      flying.current = false;
      setBusy(false);
    }
  }

  async function goBrowserCert() {
    if (flying.current) return;
    flying.current = true;
    setBusy(true);
    setErr('');
    try {
      const res = await fetch('/api/auth/cert/browser', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' }
      });
      if (!res.ok) {
        const errJson = await res.json().catch(() => ({}));
        throw new Error(errJson.message || 'Nenhum certificado de cliente válido detectado pelo navegador.');
      }
      const data = await res.json();
      await handleLoginSuccess(data);
    } catch (err: any) {
      setErr(err?.message || 'Falha ao autenticar via certificado do navegador.');
    } finally {
      flying.current = false;
      setBusy(false);
    }
  }

  return (
    <div className="lg-page">
      <div style={{ position: 'absolute', top: 16, right: 20, display: 'flex', gap: 10, alignItems: 'center' }}>
        <Button
          icon="pi pi-heart-fill"
          label={t('donate.button') || 'Doações / Apoie'}
          className="p-button-rounded p-button-danger p-button-outlined p-button-sm"
          onClick={() => setShowDonation(true)}
        />
      </div>

      <div className="brand-title">{t('app.name')}</div>
      <div className="login-box">
        <div className="center-logo">{HEXA}</div>

        {/* Dropdown de Idiomas com optionLabel e optionValue corrigidos */}
        <div style={{ marginBottom: 12 }}>
          <Dropdown
            value={i18n.resolvedLanguage || i18n.language}
            options={supportedLanguages}
            optionLabel="label"
            optionValue="code"
            onChange={(e) => i18n.changeLanguage(e.value)}
            style={{ width: '100%' }}
            aria-label={t('login.language')}
          />
        </div>

        {/* Alternador de Método de Autenticação */}
        <div style={{ display: 'flex', gap: 6, marginBottom: 16 }}>
          <button
            type="button"
            className={`btn-submit ${authMode === 'password' ? '' : 'p-button-secondary'}`}
            style={{ flex: 1, padding: '8px 12px', fontSize: '0.85rem', backgroundColor: authMode === 'password' ? '#0284c7' : '#1e293b' }}
            onClick={() => { setAuthMode('password'); setErr(''); }}
          >
            <i className="pi pi-key" style={{ marginRight: 6 }}></i>
            {t('login.tabPassword') || 'Credenciais'}
          </button>
          <button
            type="button"
            className={`btn-submit ${authMode === 'cert' ? '' : 'p-button-secondary'}`}
            style={{ flex: 1, padding: '8px 12px', fontSize: '0.85rem', backgroundColor: authMode === 'cert' ? '#0284c7' : '#1e293b' }}
            onClick={() => { setAuthMode('cert'); setErr(''); }}
          >
            <i className="pi pi-shield" style={{ marginRight: 6 }}></i>
            {t('login.tabCert') || 'Certificado'}
          </button>
        </div>

        {/* Botão SSO / OAuth2 */}
        <button
          type="button"
          className="btn-submit"
          style={{ marginBottom: 12, backgroundColor: '#0f766e' }}
          onClick={async () => {
            try {
              const sso = await api<{ enabled: boolean; authorizationUrl: string }>('/api/auth/sso');
              if (sso.enabled) window.location.assign(sso.authorizationUrl);
              else setErr(t('login.ssoNotConfigured'));
            } catch (_) {
              setErr(t('login.ssoUnavailable'));
            }
          }}
        >
          {t('login.ssoButton')}
        </button>

        {new URLSearchParams(window.location.search).has('ssoError') && (
          <p role="alert" style={{ color: '#fc8181', fontSize: '0.85rem' }}>{t('login.ssoFailed')}</p>
        )}

        {/* MODO 1: CREDENCIAIS USUÁRIO E SENHA */}
        {authMode === 'password' && (
          <form id="loginForm" onSubmit={goPassword}>
            <div className="input-group">
              <label htmlFor="username">{t('login.username')}</label>
              <div className="input-wrapper">
                <input
                  type="text"
                  id="username"
                  autoComplete="off"
                  required
                  value={u}
                  onChange={(e) => setU(e.target.value)}
                />
                {USERI}
              </div>
            </div>
            <div className="input-group">
              <label htmlFor="password">{t('login.password')}</label>
              <div className="input-wrapper">
                <input
                  type="password"
                  id="password"
                  required
                  value={p}
                  onChange={(e) => setP(e.target.value)}
                />
                {LOCKI}
              </div>
            </div>
            <div className="divider"></div>
            <button type="submit" className="btn-submit" id="submitBtn" disabled={busy}>
              {busy ? t('login.authenticating') : t('login.submit')}
            </button>
          </form>
        )}

        {/* MODO 2: CERTIFICADO DIGITAL */}
        {authMode === 'cert' && (
          <div>
            <div style={{ display: 'flex', gap: 6, marginBottom: 12 }}>
              <button
                type="button"
                style={{
                  flex: 1,
                  padding: '6px 8px',
                  fontSize: '0.8rem',
                  borderRadius: 4,
                  border: '1px solid #334155',
                  background: certType === 'manual' ? '#1e293b' : 'transparent',
                  color: certType === 'manual' ? '#38bdf8' : '#94a3b8',
                  cursor: 'pointer'
                }}
                onClick={() => setCertType('manual')}
              >
                Opção A: Arquivo (.p12/.pfx)
              </button>
              <button
                type="button"
                style={{
                  flex: 1,
                  padding: '6px 8px',
                  fontSize: '0.8rem',
                  borderRadius: 4,
                  border: '1px solid #334155',
                  background: certType === 'browser' ? '#1e293b' : 'transparent',
                  color: certType === 'browser' ? '#38bdf8' : '#94a3b8',
                  cursor: 'pointer'
                }}
                onClick={() => setCertType('browser')}
              >
                Opção B: Navegador (mTLS)
              </button>
            </div>

            {certType === 'manual' ? (
              <form onSubmit={goManualCert}>
                <div className="input-group">
                  <label htmlFor="certFileInput">Arquivo do Certificado (.p12, .pfx)</label>
                  <div style={{ display: 'flex', alignItems: 'center', marginTop: 4 }}>
                    <input
                      type="file"
                      id="certFileInput"
                      accept=".p12,.pfx"
                      required
                      onChange={(e) => setCertFile(e.target.files?.[0] || null)}
                      style={{ color: '#cbd5e1', fontSize: '0.85rem' }}
                    />
                  </div>
                  <small style={{ color: '#64748b', fontSize: '0.75rem', marginTop: 4, display: 'block' }}>
                    Chave privada descriptografada e assinada localmente no navegador (nunca enviada ao servidor).
                  </small>
                </div>

                <div className="input-group">
                  <label htmlFor="certPassInput">Senha do Certificado (PKCS#12)</label>
                  <div className="input-wrapper">
                    <input
                      type="password"
                      id="certPassInput"
                      value={certPass}
                      onChange={(e) => setCertPass(e.target.value)}
                      placeholder="Senha do arquivo .p12/.pfx"
                    />
                    {LOCKI}
                  </div>
                </div>

                <div className="divider"></div>
                <button type="submit" className="btn-submit" disabled={busy || !certFile}>
                  {busy ? t('login.authenticating') : 'Entrar com Arquivo de Certificado'}
                </button>
              </form>
            ) : (
              <div>
                <p style={{ fontSize: '0.85rem', color: '#94a3b8', lineHeight: 1.4, margin: '8px 0 16px 0' }}>
                  Autenticação direta através do certificado de cliente instalado no repositório de chaves do seu navegador ou sistema operacional (via sessão mTLS/HTTPS).
                </p>
                <button
                  type="button"
                  className="btn-submit"
                  disabled={busy}
                  onClick={goBrowserCert}
                  style={{ backgroundColor: '#2563eb' }}
                >
                  <i className="pi pi-id-card" style={{ marginRight: 8 }}></i>
                  {busy ? t('login.authenticating') : 'Entrar com Certificado do Navegador'}
                </button>
              </div>
            )}
          </div>
        )}

        <div className="error-msg" id="errorMsg" style={err ? { display: 'block', marginTop: 12 } : {}}>
          {err}
        </div>
      </div>

      <DonationModal visible={showDonation} onHide={() => setShowDonation(false)} />
    </div>
  );
}
