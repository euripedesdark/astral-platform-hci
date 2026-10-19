import React, {useCallback, useEffect, useState} from 'react';
import {useTranslation} from 'react-i18next';
import {Card} from 'primereact/card';
import {Button} from 'primereact/button';
import {InputText} from 'primereact/inputtext';
import {Dropdown} from 'primereact/dropdown';
import {Checkbox} from 'primereact/checkbox';
import {Message} from 'primereact/message';
import {Tag} from 'primereact/tag';
import {api} from '../../../shared/services/api';

interface InterfaceRede {
    nome: string;
    papel: 'WAN' | 'LAN' | '';
    enderecoCidr: string;
    gateway: string;
    servidorDhcp: boolean;
    dhcpInicio: string;
    dhcpFim: string;
}
interface EnderecamentoRede {
    metodo: 'manual' | 'auto';
    interfaces: InterfaceRede[];
    dns: string[];
    dominioDhcp: string;
    protegerResolvConf: boolean;
}
interface EstadoApi {
    addressing: EnderecamentoRede;
    interfaces: string[];
    observado: Record<string, unknown>;
}
interface Validacao {
    erros: string[];
    aprovado: boolean;
    plano?: string[];
    semMudanca?: boolean;
}
interface Intencao {
    intentId?: string;
    status?: string;
    plano?: string[];
    message?: string;
}

const vazia = (): InterfaceRede => ({
    nome: '', papel: 'LAN', enderecoCidr: '', gateway: '', servidorDhcp: false, dhcpInicio: '', dhcpFim: ''
});

export default function EnderecamentoPage() {
    const{t}=useTranslation();
    const [estado, setEstado] = useState<EstadoApi | null>(null);
    const [desejo, setDesejo] = useState<EnderecamentoRede | null>(null);
    const [dnsTexto, setDnsTexto] = useState('');
    const [validacao, setValidacao] = useState<Validacao | null>(null);
    const [intencao, setIntencao] = useState<Intencao | null>(null);
    const [erro, setErro] = useState('');
    const [busy, setBusy] = useState(false);

    const carrega = useCallback(async () => {
        try {
            const e = await api<EstadoApi>('/api/v1/network/addressing');
            setEstado(e);
            const a = e.addressing;
            setDesejo({
                metodo: a.metodo || 'manual',
                interfaces: (a.interfaces || []).map(i => ({...i})),
                dns: a.dns || [],
                dominioDhcp: a.dominioDhcp || '',
                protegerResolvConf: !!a.protegerResolvConf
            });
            setDnsTexto((a.dns || []).join(', '));
            setErro('');
        } catch (ex) {
            setErro(ex instanceof Error ? ex.message : String(ex));
        }
    }, []);

    useEffect(() => { void carrega(); }, [carrega]);

    function muta(fn: (d: EnderecamentoRede) => EnderecamentoRede) {
        setDesejo(d => (d ? fn(d) : d));
        setValidacao(null);
    }

    function mutaInterface(idx: number, campo: keyof InterfaceRede, valor: string | boolean) {
        muta(d => {
            const interfaces = d.interfaces.map((i, k) => (k === idx ? {...i, [campo]: valor} : i));
            return {...d, interfaces};
        });
    }

    function montaPayload(): EnderecamentoRede {
        const d = desejo as EnderecamentoRede;
        const dns = dnsTexto.split(',').map(s => s.trim()).filter(Boolean);
        return {...d, dns};
    }

    async function validar() {
        setBusy(true); setErro(''); setIntencao(null);
        try {
            const r = await api<Validacao>('/api/v1/network/addressing/validar', {
                method: 'POST', body: JSON.stringify(montaPayload())
            });
            setValidacao(r);
        } catch (ex) {
            setValidacao({erros: [ex instanceof Error ? ex.message : String(ex)], aprovado: false});
        } finally { setBusy(false); }
    }

    async function aplicar() {
        const plano = validacao?.plano ?? [];
        const confirmar = window.confirm(
            t('addressing.confirmApply', { plan: plano.length ? plano.join('\n') : t('addressing.noChangesPlan') })
        );
        if (!confirmar) return;
        setBusy(true); setErro('');
        try {
            const r = await api<Intencao>('/api/v1/network/addressing', {
                method: 'POST', body: JSON.stringify(montaPayload())
            });
            setIntencao(r);
        } catch (ex) { setErro(ex instanceof Error ? ex.message : String(ex)); }
        finally { setBusy(false); }
    }

    if (!desejo) return <Card title={t('addressing.title')}><p className="muted">{erro || t('addressing.loading')}</p></Card>;

    return (
        <div className="grid">
            <Card title={t('addressing.desired')} className="col-12">
                {erro && <Message severity="error" text={erro} style={{display: 'block', marginBottom: 10}} />}

                <div className="flex flex-wrap gap-3 align-items-center" style={{marginBottom: 12}}>
                    <label className="muted">{t('addressing.method')}</label>
                    <Dropdown value={desejo.metodo}
                              options={[{label: t('addressing.manual'), value: 'manual'}, {label: t('addressing.auto'), value: 'auto'}]}
                              onChange={e => muta(d => ({...d, metodo: e.value}))} style={{width: 200}} />
                    <label className="muted">{t('addressing.dhcpDomain')}</label>
                    <InputText value={desejo.dominioDhcp} placeholder={t('addressing.emptyInterface.domain')}
                               onChange={e => muta(d => ({...d, dominioDhcp: e.target.value}))} style={{width: 200}} />
                    <label>
                        <Checkbox checked={desejo.protegerResolvConf} inputId="prot"
                                  onChange={e => muta(d => ({...d, protegerResolvConf: !!e.checked}))} />
                        <span style={{marginLeft: 6}}>{t('addressing.protectResolv')}</span>
                    </label>
                </div>

                <div className="flex flex-wrap gap-2 align-items-center" style={{marginBottom: 12}}>
                    <label className="muted" style={{width: 40}}>{t('addressing.dns')}</label>
                    <InputText value={dnsTexto} placeholder="1.1.1.1, 8.8.8.8" style={{minWidth: 320}}
                               onChange={e => { setDnsTexto(e.target.value); setValidacao(null); }} />
                    <span className="muted">{t('addressing.dnsHint')}</span>
                </div>

                <h4 style={{margin: '14px 0 8px'}}>{t('addressing.interfaces')}</h4>
                <div className="flex flex-wrap gap-2 align-items-center" style={{marginBottom: 6}}>
                    <span className="muted" style={{width: 90}}>{t('addressing.columns.name')}</span>
                    <span className="muted" style={{width: 90}}>{t('addressing.columns.role')}</span>
                    <span className="muted" style={{width: 180}}>{t('addressing.columns.address')}</span>
                    <span className="muted" style={{width: 170}}>{t('addressing.columns.gateway')}</span>
                    <span className="muted" style={{width: 210}}>{t('addressing.columns.dhcp')}</span>
                </div>

                {desejo.interfaces.map((i, idx) => (
                    <div className="flex flex-wrap gap-2 align-items-center" key={idx} style={{marginBottom: 6}}>
                        <InputText value={i.nome} placeholder={t('addressing.emptyInterface.name')} style={{width: 90}}
                                   onChange={e => mutaInterface(idx, 'nome', e.target.value)} />
                        <Dropdown value={i.papel} options={[{label: 'WAN', value: 'WAN'}, {label: 'LAN', value: 'LAN'}]}
                                  onChange={e => mutaInterface(idx, 'papel', e.value)} style={{width: 90}} />
                        <InputText value={i.enderecoCidr} placeholder={t('addressing.emptyInterface.address')} style={{width: 180}}
                                   onChange={e => mutaInterface(idx, 'enderecoCidr', e.target.value)} />
                        <InputText value={i.gateway} placeholder={t('addressing.emptyInterface.gateway')} style={{width: 170}}
                                   onChange={e => mutaInterface(idx, 'gateway', e.target.value)} />
                        <label className="flex align-items-center" style={{width: 110}}>
                            <Checkbox checked={i.servidorDhcp}
                                      onChange={e => mutaInterface(idx, 'servidorDhcp', !!e.checked)} />
                            <span style={{marginLeft: 6}}>{t('addressing.dhcp')}</span>
                        </label>
                        <InputText value={i.dhcpInicio} placeholder={t('addressing.emptyInterface.dhcpStart')} style={{width: 130}}
                                   onChange={e => mutaInterface(idx, 'dhcpInicio', e.target.value)} />
                        <InputText value={i.dhcpFim} placeholder={t('addressing.emptyInterface.dhcpEnd')} style={{width: 130}}
                                   onChange={e => mutaInterface(idx, 'dhcpFim', e.target.value)} />
                        <Button icon="pi pi-trash" severity="danger" text size="small"
                                onClick={() => muta(d => ({...d, interfaces: d.interfaces.filter((_, k) => k !== idx)}))} />
                    </div>
                ))}

                <div className="flex gap-2" style={{marginTop: 10}}>
                    <Button label={t('addressing.addInterface')} icon="pi pi-plus" text size="small"
                            onClick={() => muta(d => ({...d, interfaces: [...d.interfaces, vazia()]}))} />
                    <Button label={t('addressing.clear')} icon="pi pi-eraser" text size="small"
                            onClick={() => { setDesejo({metodo: 'manual', interfaces: [vazia()], dns: [], dominioDhcp: '', protegerResolvConf: true}); setDnsTexto(''); setValidacao(null); }} />
                </div>
            </Card>

            <Card title="Plano" className="col-12">
                <div className="flex gap-2" style={{marginBottom: 10}}>
                    <Button label={t('addressing.validate')} icon="pi pi-check-circle" onClick={validar} disabled={busy} />
                    <Button label={t('addressing.apply')} icon="pi pi-send" severity="warning" onClick={aplicar}
                            disabled={busy || !validacao?.aprovado} />
                </div>

                {validacao?.aprovado === false && (validacao.erros || []).map((e, k) =>
                    <Message key={k} severity="error" text={e} style={{display: 'block', marginBottom: 6}} />)}

                {validacao?.aprovado && (
                    <div>
                        {validacao.semMudanca
                            ? <Tag value={t('addressing.noChanges')} severity="info" />
                            : <ul style={{margin: 0, paddingLeft: 20}}>
                                {(validacao.plano || []).map((p, k) => <li key={k}>{p}</li>)}
                              </ul>}
                        <p className="muted" style={{marginTop: 8}}>
                            {t('addressing.validateZero')}
                        </p>
                    </div>
                )}

                {intencao && (
                    <Message severity="success"
                             text={`${t('addressing.intention')} ${intencao.intentId} · ${intencao.status} — ${intencao.message ?? ''}`}
                             style={{display: 'block', marginTop: 8}} />
                )}
            </Card>

            <Card title={t('addressing.observed')} className="col-12">
                <pre className="muted" style={{whiteSpace: 'pre-wrap', margin: 0, fontSize: 12}}>
                    {JSON.stringify(estado?.observado ?? {}, null, 2)}
                </pre>
            </Card>
        </div>
    );
}