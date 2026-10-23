import React, { useState } from 'react';
import { Dialog } from 'primereact/dialog';
import { Button } from 'primereact/button';
import { useTranslation } from 'react-i18next';

interface DonationModalProps {
  visible: boolean;
  onHide: () => void;
}

const DONATION_DATA = {
  pix: '24adc62c-b073-4587-974d-03fe35f6733f',
  wise: {
    name: 'Euripedes Batista de Paiva Junior',
    type: 'Checking',
    routing: '101019628',
    account: '215822927677',
    bank: 'Wise US Inc, 108 W 13th St, Wilmington, DE, 19801, United States',
    swift: 'TRWIUS35XXX'
  }
};

const TEXTS: Record<string, {
  title: string;
  subtitle: string;
  pixTitle: string;
  pixDesc: string;
  copyPix: string;
  copied: string;
  wiseTitle: string;
  wiseDesc: string;
  name: string;
  type: string;
  routing: string;
  account: string;
  bank: string;
  swift: string;
  note: string;
  close: string;
}> = {
  'pt-BR': {
    title: 'Apoie o Astral Platform & HCI',
    subtitle: 'Astral Platform & HCI é uma plataforma de infraestrutura hiperconvergente Open Source mantida por um único desenvolvedor.',
    pixTitle: 'PIX (Brasil)',
    pixDesc: 'Copie a chave aleatória abaixo para realizar uma doação instantânea via PIX:',
    copyPix: 'Copiar Chave PIX',
    copied: 'Copiado para a área de transferência!',
    wiseTitle: 'Transferência Internacional (Wise)',
    wiseDesc: 'Para transferências dos Estados Unidos (ACH/Wire) ou de qualquer outro país via SWIFT:',
    name: 'Titular / Beneficiário',
    type: 'Tipo de Conta',
    routing: 'Routing Number (ACH / Wire)',
    account: 'Número da Conta',
    bank: 'Banco e Endereço',
    swift: 'Código SWIFT / BIC',
    note: 'O Routing Number aplica-se a envios dos EUA. Para os demais países, utilize o código SWIFT/BIC.',
    close: 'Fechar'
  },
  'en-US': {
    title: 'Support Astral Platform & HCI',
    subtitle: 'Astral Platform & HCI is an open-source hyperconverged infrastructure platform maintained by a single developer.',
    pixTitle: 'PIX (Instant Transfer - Brazil)',
    pixDesc: 'Copy the random PIX key below for instant transfer in Brazil:',
    copyPix: 'Copy PIX Key',
    copied: 'Copied to clipboard!',
    wiseTitle: 'International Bank Transfer (Wise)',
    wiseDesc: 'For domestic US transfers (ACH/Wire) or international transfers via SWIFT worldwide:',
    name: 'Account Holder',
    type: 'Account Type',
    routing: 'Routing Number (ACH / Wire)',
    account: 'Account Number',
    bank: 'Bank Name & Address',
    swift: 'SWIFT / BIC Code',
    note: 'The Routing Number is used within the US. For transfers outside the US, please use SWIFT/BIC.',
    close: 'Close'
  },
  'es-ES': {
    title: 'Apoya a Astral Platform & HCI',
    subtitle: 'Astral Platform & HCI es una plataforma de infraestructura hiperconvergente de código abierto desarrollada por un único programador.',
    pixTitle: 'PIX (Brasil)',
    pixDesc: 'Copia la clave aleatoria a continuación para donar al instante mediante PIX:',
    copyPix: 'Copiar Clave PIX',
    copied: '¡Copiado al portapapeles!',
    wiseTitle: 'Transferencia Internacional (Wise)',
    wiseDesc: 'Para transferencias domésticas desde EE.UU. (ACH/Wire) o internacionales vía SWIFT:',
    name: 'Titular / Beneficiario',
    type: 'Tipo de Cuenta',
    routing: 'Routing Number (ACH / Wire)',
    account: 'Número de Cuenta',
    bank: 'Nombre y Dirección del Banco',
    swift: 'Código SWIFT / BIC',
    note: 'El Routing Number se utiliza en envíos dentro de EE.UU. Fuera de allí, utiliza el código SWIFT/BIC.',
    close: 'Cerrar'
  },
  'fr-FR': {
    title: 'Soutenez Astral Platform & HCI',
    subtitle: 'Astral Platform & HCI est une plateforme d\'infrastructure hyperconvergée open source développée par un seul contributeur.',
    pixTitle: 'PIX (Brésil)',
    pixDesc: 'Copiez la clé aléatoire ci-dessous pour faire un don instantané via PIX :',
    copyPix: 'Copier la clé PIX',
    copied: 'Copié dans le presse-papiers !',
    wiseTitle: 'Virement bancaire international (Wise)',
    wiseDesc: 'Pour les virements depuis les États-Unis (ACH/Wire) ou depuis le monde entier via SWIFT :',
    name: 'Titulaire du compte',
    type: 'Type de compte',
    routing: 'Routing Number (ACH / Wire)',
    account: 'Numéro de compte',
    bank: 'Nom et adresse de la banque',
    swift: 'Code SWIFT / BIC',
    note: 'Le Routing Number est utilisé aux États-Unis. Ailleurs, utilisez le code SWIFT/BIC.',
    close: 'Fermer'
  }
};

export default function DonationModal({ visible, onHide }: DonationModalProps) {
  const { i18n } = useTranslation();
  const currentLang = i18n.resolvedLanguage || i18n.language || 'pt-BR';
  const text = TEXTS[currentLang] || TEXTS['pt-BR'];
  const [copiedField, setCopiedField] = useState<string | null>(null);

  const copyToClipboard = (val: string, field: string) => {
    navigator.clipboard.writeText(val);
    setCopiedField(field);
    setTimeout(() => setCopiedField(null), 3000);
  };

  const footer = (
    <div style={{ display: 'flex', justifyContent: 'flex-end', paddingTop: 8 }}>
      <Button label={text.close} icon="pi pi-check" onClick={onHide} className="p-button-outlined" />
    </div>
  );

  return (
    <Dialog
      header={
        <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
          <i className="pi pi-heart-fill" style={{ color: '#e53e3e', fontSize: '1.4rem' }} />
          <span style={{ fontSize: '1.2rem', fontWeight: 600, color: '#f8fafc' }}>{text.title}</span>
        </div>
      }
      visible={visible}
      style={{ width: '92%', maxWidth: '640px', backgroundColor: '#0f172a' }}
      onHide={onHide}
      footer={footer}
      className="astral-donation-dialog"
    >
      <div style={{ color: '#94a3b8', fontSize: '0.92rem', lineHeight: 1.5, marginBottom: 20 }}>
        {text.subtitle}
      </div>

      {/* PIX SECTION */}
      <div style={{
        background: 'rgba(30, 41, 59, 0.7)',
        border: '1px solid #334155',
        borderRadius: 8,
        padding: 16,
        marginBottom: 16
      }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 8 }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
            <span style={{ fontSize: '1.2rem' }}>🇧🇷</span>
            <span style={{ fontWeight: 600, color: '#38bdf8' }}>{text.pixTitle}</span>
          </div>
          {copiedField === 'pix' && (
            <span style={{ fontSize: '0.8rem', color: '#4ade80', fontWeight: 500 }}>
              {text.copied}
            </span>
          )}
        </div>
        <p style={{ margin: '0 0 10px 0', fontSize: '0.85rem', color: '#cbd5e1' }}>{text.pixDesc}</p>
        <div style={{
          display: 'flex',
          alignItems: 'center',
          gap: 8,
          background: '#090d16',
          padding: '8px 12px',
          borderRadius: 6,
          border: '1px solid #1e293b'
        }}>
          <code style={{ flex: 1, color: '#38ef7d', fontSize: '0.88rem', wordBreak: 'break-all', fontFamily: 'monospace' }}>
            {DONATION_DATA.pix}
          </code>
          <Button
            icon={copiedField === 'pix' ? 'pi pi-check' : 'pi pi-copy'}
            label={copiedField === 'pix' ? '' : text.copyPix}
            className={copiedField === 'pix' ? 'p-button-success p-button-sm' : 'p-button-sm p-button-info'}
            onClick={() => copyToClipboard(DONATION_DATA.pix, 'pix')}
          />
        </div>
      </div>

      {/* WISE SECTION */}
      <div style={{
        background: 'rgba(30, 41, 59, 0.7)',
        border: '1px solid #334155',
        borderRadius: 8,
        padding: 16
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8 }}>
          <span style={{ fontSize: '1.2rem' }}>🌐</span>
          <span style={{ fontWeight: 600, color: '#a78bfa' }}>{text.wiseTitle}</span>
        </div>
        <p style={{ margin: '0 0 12px 0', fontSize: '0.85rem', color: '#cbd5e1' }}>{text.wiseDesc}</p>

        <div style={{ overflowX: 'auto' }}>
          <table style={{ width: '100%', fontSize: '0.84rem', borderCollapse: 'collapse', color: '#e2e8f0' }}>
            <tbody>
              <tr style={{ borderBottom: '1px solid #1e293b' }}>
                <td style={{ padding: '6px 8px', color: '#94a3b8' }}>{text.name}</td>
                <td style={{ padding: '6px 8px', fontWeight: 500 }}>{DONATION_DATA.wise.name}</td>
                <td style={{ padding: '6px 8px', textAlign: 'right' }}>
                  <Button icon="pi pi-copy" text className="p-button-sm" onClick={() => copyToClipboard(DONATION_DATA.wise.name, 'name')} />
                </td>
              </tr>
              <tr style={{ borderBottom: '1px solid #1e293b' }}>
                <td style={{ padding: '6px 8px', color: '#94a3b8' }}>{text.type}</td>
                <td style={{ padding: '6px 8px' }}>{DONATION_DATA.wise.type}</td>
                <td></td>
              </tr>
              <tr style={{ borderBottom: '1px solid #1e293b' }}>
                <td style={{ padding: '6px 8px', color: '#94a3b8' }}>{text.routing}</td>
                <td style={{ padding: '6px 8px', fontFamily: 'monospace', color: '#38ef7d' }}>{DONATION_DATA.wise.routing}</td>
                <td style={{ padding: '6px 8px', textAlign: 'right' }}>
                  <Button icon="pi pi-copy" text className="p-button-sm" onClick={() => copyToClipboard(DONATION_DATA.wise.routing, 'routing')} />
                </td>
              </tr>
              <tr style={{ borderBottom: '1px solid #1e293b' }}>
                <td style={{ padding: '6px 8px', color: '#94a3b8' }}>{text.account}</td>
                <td style={{ padding: '6px 8px', fontFamily: 'monospace', color: '#38ef7d' }}>{DONATION_DATA.wise.account}</td>
                <td style={{ padding: '6px 8px', textAlign: 'right' }}>
                  <Button icon="pi pi-copy" text className="p-button-sm" onClick={() => copyToClipboard(DONATION_DATA.wise.account, 'account')} />
                </td>
              </tr>
              <tr style={{ borderBottom: '1px solid #1e293b' }}>
                <td style={{ padding: '6px 8px', color: '#94a3b8' }}>{text.swift}</td>
                <td style={{ padding: '6px 8px', fontFamily: 'monospace', color: '#38bdf8' }}>{DONATION_DATA.wise.swift}</td>
                <td style={{ padding: '6px 8px', textAlign: 'right' }}>
                  <Button icon="pi pi-copy" text className="p-button-sm" onClick={() => copyToClipboard(DONATION_DATA.wise.swift, 'swift')} />
                </td>
              </tr>
              <tr>
                <td style={{ padding: '6px 8px', color: '#94a3b8' }}>{text.bank}</td>
                <td style={{ padding: '6px 8px', fontSize: '0.8rem' }} colSpan={2}>{DONATION_DATA.wise.bank}</td>
              </tr>
            </tbody>
          </table>
        </div>

        <div style={{ marginTop: 10, fontSize: '0.78rem', color: '#64748b', fontStyle: 'italic' }}>
          ℹ️ {text.note}
        </div>
      </div>
    </Dialog>
  );
}
