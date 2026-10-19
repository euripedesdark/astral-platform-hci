import React, {useCallback, useEffect, useState} from 'react';
import {useTranslation} from 'react-i18next';
import {Card} from 'primereact/card';
import {Button} from 'primereact/button';
import {InputText} from 'primereact/inputtext';
import {Dropdown} from 'primereact/dropdown';
import {DataTable} from 'primereact/datatable';
import {Column} from 'primereact/column';
import {TabView, TabPanel} from 'primereact/tabview';
import {Tag} from 'primereact/tag';
import {Message} from 'primereact/message';
import {api} from '../../../shared/services/api';
import {operacoes, type OperacaoOpenApi} from '../../../shared/services/openapi';

interface Categoria { id: number; code: string; name: string; description: string; blockedByDefault: boolean; source: string }
interface Dominio { id: number; pattern: string; categoryId: number; categoryCode: string; categoryName: string; source: string }
interface Excecao { id: number; pattern: string; adGroup: string; action: string; reason: string; enabled: boolean; createdBy: string }
interface Regra { id: number; categoryId: number; categoryCode: string; categoryName: string; adGroup: string; action: string; priority: number; enabled: boolean; justification: string }
interface Stats { [k: string]: string | number }
interface Tentativa { status?: number; origem?: string; motivo?: string; categoria?: string; grupo?: string; host?: string }

const ACAO_DROP = () => [{label: 'ALLOW', value: 'ALLOW'}, {label: 'DENY', value: 'DENY'}];

export default function AclPage() {
    const{t}=useTranslation();
    const EXCECAO_DROP = [
        {label: `${t('acl.default.allowed')} (libera sempre)`, value: 'ALLOW'},
        {label: `${t('acl.default.byCategory')} (bloqueia sempre)`, value: 'DENY'},
        {label: 'BYPASS (pula o motor e o cache)', value: 'BYPASS'}
    ];
    const [stats, setStats] = useState<Stats>({});
    const [categorias, setCategorias] = useState<Categoria[]>([]);
    const [dominios, setDominios] = useState<Dominio[]>([]);
    const [excecoes, setExcecoes] = useState<Excecao[]>([]);
    const [regras, setRegras] = useState<Regra[]>([]);
    const [erro, setErro] = useState('');
    const [aviso, setAviso] = useState('');
    const [busy, setBusy] = useState(false);

    const [novoDominio, setNovoDominio] = useState('');
    const [catDominio, setCatDominio] = useState<number | null>(null);
    const [novaExcecao, setNovaExcecao] = useState({pattern: '', adGroup: '', action: 'ALLOW', reason: ''});
    const [novaRegra, setNovaRegra] = useState({categoryId: 0, adGroup: '', action: 'DENY', priority: 100, justification: ''});

    const [testeHost, setTesteHost] = useState('');
    const [tentativa, setTentativa] = useState<Tentativa | null>(null);

    const [contrato, setContrato] = useState<OperacaoOpenApi[]>([]);
    const [contratoErro, setContratoErro] = useState('');

    const carrega = useCallback(async () => {
        try {
            const [s, c, d, e, r] = await Promise.all([
                api<Stats>('/api/v1/acl/stats'),
                api<Categoria[]>('/api/v1/acl/categories'),
                api<Dominio[]>('/api/v1/acl/domains'),
                api<Excecao[]>('/api/v1/acl/overrides'),
                api<Regra[]>('/api/v1/acl/rules')
            ]);
            setStats(s); setCategorias(c); setDominios(d); setExcecoes(e); setRegras(r);
            setErro('');
        } catch (ex) {
            setErro(ex instanceof Error ? ex.message : String(ex));
        }
    }, []);

    useEffect(() => { void carrega(); }, [carrega]);

    useEffect(() => {
        operacoes('/api/v1/acl')
            .then(setContrato)
            .catch(ex => setContratoErro(ex instanceof Error ? ex.message : String(ex)));
    }, []);

    function notifica(msg: string) {
        setAviso(msg);
        window.setTimeout(() => setAviso(''), 6000);
    }

    async function executa(acao: () => Promise<string>) {
        setBusy(true); setErro('');
        try { notifica(await acao()); await carrega(); }
        catch (ex) { setErro(ex instanceof Error ? ex.message : String(ex)); }
        finally { setBusy(false); }
    }

    const criaDominio = () => executa(async () => {
        if (!novoDominio || !catDominio) throw new Error(t('acl.errors.patternAndCategory'));
        await api('/api/v1/acl/domains', {method: 'POST', body: JSON.stringify({pattern: novoDominio, categoryId: catDominio, source: 'manual'})});
        setNovoDominio('');
        return `${t('acl.domainRegistered')} ${novoDominio}`;
    });

    const criaExcecao = () => executa(async () => {
        if (!novaExcecao.pattern) throw new Error(t('acl.errors.exceptionPattern'));
        await api('/api/v1/acl/overrides', {method: 'POST', body: JSON.stringify(novaExcecao)});
        setNovaExcecao({pattern: '', adGroup: '', action: 'ALLOW', reason: ''});
        return t('acl.exceptionCreated');
    });

    const criaRegra = () => executa(async () => {
        if (!novaRegra.categoryId || !novaRegra.adGroup) throw new Error(t('acl.errors.categoryAndGroup'));
        await api('/api/v1/acl/rules', {method: 'POST', body: JSON.stringify(novaRegra)});
        setNovaRegra({categoryId: 0, adGroup: '', action: 'DENY', priority: 100, justification: ''});
        return t('acl.ruleCreated');
    });

    const remove = (rota: string, id: number, oque: string) => executa(async () => {
        await api(`/api/v1/acl/${rota}/${id}`, {method: 'DELETE'});
        return `${oque} ${t('acl.removed')}`;
    });

    const invalidaCache = () => executa(async () => {
        await api('/api/v1/acl/cache/invalidate', {method: 'POST'});
        return t('acl.cacheInvalidated');
    });

    const testa = async () => {
        if (!testeHost) return;
        setBusy(true);
        try {
            const r = await fetch('/api/v1/acl/check', {headers: {'X-Original-Host': testeHost, 'X-Original-URI': '/'}});
            setTentativa(await r.json() as Tentativa);
        } catch { setTentativa({status: 0, motivo: t('acl.noCategory')}); }
        finally { setBusy(false); }
    };

    const acaoTag = (v: string) =>
        <Tag value={v} severity={v === 'ALLOW' ? 'success' : 'danger'} />;

    return (
        <div className="grid">
            <Card title={t('acl.engineState')} className="col-12">
                <div className="flex flex-wrap gap-2 align-items-center">
                    <Tag value={`${t('acl.allowed')} ${Number(stats.permitidas ?? 0)}`} severity="success" />
                    <Tag value={`${t('acl.denied')} ${Number(stats.negadas ?? 0)}`} severity="danger" />
                    <Tag value={`${t('acl.noIdentity')} ${Number(stats.semIdentidade ?? 0)}`} severity="warning" />
                    <Tag value={`${t('acl.infraFailures')} ${Number(stats.falhasInfra ?? 0)}`} severity={Number(stats.falhasInfra ?? 0) > 0 ? 'danger' : 'info'} />
                    <span className="muted" style={{marginLeft: 8}}>{String(stats.cache ?? '')}</span>
                    <Button label={t('acl.invalidateCache')} icon="pi pi-refresh" size="small" outlined onClick={invalidaCache} disabled={busy} />
                    <Button label={t('common.reload')} icon="pi pi-sync" size="small" text onClick={carrega} disabled={busy} />
                </div>
                {erro && <Message severity="error" text={erro} style={{display: 'block', marginTop: 10}} />}
                {aviso && <Message severity="success" text={aviso} style={{display: 'block', marginTop: 10}} />}
            </Card>

            <Card title={t('acl.testDecision')} className="col-12">
                <div className="flex flex-wrap gap-2 align-items-center">
                    <InputText value={testeHost} placeholder={t('acl.placeholder.host')} onChange={e => setTesteHost(e.target.value)}
                               onKeyDown={e => e.key === 'Enter' && void testa()} />
                    <Button label={t('acl.queryEngine')} icon="pi pi-search" onClick={testa} disabled={busy} />
                    {tentativa && (
                        <span className="flex align-items-center gap-2">
                            <Tag value={`${tentativa.status ?? '?'}`} severity={(tentativa.status === 200 ? 'success' : 'danger')} />
                            <b>{tentativa.origem}</b>
                            <span className="muted">{tentativa.categoria ?? t('acl.noCategory')} · {tentativa.grupo ?? '-'}</span>
                            <span className="muted">{tentativa.motivo}</span>
                        </span>
                    )}
                </div>
                <p className="muted" style={{marginBottom: 0}}>
                    {t('acl.decision')}
                </p>
            </Card>

            <Card title={t('acl.openapiContract')} className="col-12"
                  subTitle={<a href="/swagger-ui.html" target="_blank" rel="noreferrer">{t('acl.swaggerUi')}</a>}>
                {contratoErro
                    ? <Message severity="warn" text={contratoErro} />
                    : <div className="flex flex-wrap gap-2">
                        {contrato.map(o => (
                            <Tag key={`${o.metodo} ${o.path}`}
                                 value={`${o.metodo} ${o.path.replace('/api/v1/acl', '') || '/'}`}
                                 severity={o.metodo === 'GET' ? 'info' : o.metodo === 'DELETE' ? 'danger' : 'success'}
                                 title={o.summary} />
                        ))}
                      </div>}
                <p className="muted" style={{marginBottom: 0}}>
                    {t('acl.openapiNote')}
                </p>
            </Card>

            <div className="col-12">
                <TabView>
                    <TabPanel header={`${t('acl.domains')} (${dominios.length})`}>
                        <div className="flex flex-wrap gap-2 align-items-center" style={{marginBottom: 10}}>
                            <InputText value={novoDominio} placeholder={t('acl.placeholder.domain')}
                                       onChange={e => setNovoDominio(e.target.value)} />
                            <Dropdown value={catDominio} options={categorias.map(c => ({label: `${c.code} — ${c.name}`, value: c.id}))}
                                      placeholder={t('acl.placeholder.category')} onChange={e => setCatDominio(e.value as number)} style={{minWidth: 260}} />
                            <Button label={t('acl.register')} icon="pi pi-plus" onClick={criaDominio} disabled={busy} />
                        </div>
                        <DataTable value={dominios} size="small" stripedRows rows={10} emptyMessage={t('acl.noDomains')}>
                            <Column field="pattern" header={t('acl.pattern')} />
                            <Column field="categoryCode" header={t('acl.category')} />
                            <Column field="source" header={t('acl.originCol')} />
                            <Column header="" body={(r: Dominio) =>
                                <Button icon="pi pi-trash" size="small" severity="danger" text
                                        onClick={() => remove('domains', r.id, t('acl.domains'))} />} />
                        </DataTable>
                    </TabPanel>

                    <TabPanel header={`${t('acl.exceptions')} (${excecoes.length})`}>
                        <div className="flex flex-wrap gap-2 align-items-center" style={{marginBottom: 10}}>
                            <InputText value={novaExcecao.pattern} placeholder={t('acl.placeholder.exceptionPattern')}
                                       onChange={e => setNovaExcecao({...novaExcecao, pattern: e.target.value})} />
                            <InputText value={novaExcecao.adGroup} placeholder={t('acl.placeholder.adGroupEmpty')}
                                       onChange={e => setNovaExcecao({...novaExcecao, adGroup: e.target.value})} />
                            <Dropdown value={novaExcecao.action} options={EXCECAO_DROP}
                                      onChange={e => setNovaExcecao({...novaExcecao, action: e.value})} />
                            <InputText value={novaExcecao.reason} placeholder={t('acl.placeholder.reason')}
                                       onChange={e => setNovaExcecao({...novaExcecao, reason: e.target.value})} />
                            <Button label={t('acl.createException')} icon="pi pi-plus" onClick={criaExcecao} disabled={busy} />
                        </div>
                        <DataTable value={excecoes} size="small" stripedRows rows={10} emptyMessage={t('acl.noExceptions')}>
                            <Column field="pattern" header={t('acl.pattern')} />
                            <Column field="adGroup" header={t('acl.group')} />
                            <Column header={t('acl.action')} body={(r: Excecao) => acaoTag(r.action)} />
                            <Column field="reason" header={t('acl.reason')} />
                            <Column field="createdBy" header={t('acl.who')} />
                            <Column header="" body={(r: Excecao) =>
                                <Button icon="pi pi-trash" size="small" severity="danger" text
                                        onClick={() => remove('overrides', r.id, t('acl.exceptions'))} />} />
                        </DataTable>
                        <p className="muted">
                            {t('acl.exceptionRunsBefore')}
                        </p>
                    </TabPanel>

                    <TabPanel header={`${t('acl.rules')} (${regras.length})`}>
                        <div className="flex flex-wrap gap-2 align-items-center" style={{marginBottom: 10}}>
                            <Dropdown value={novaRegra.categoryId || null}
                                      options={categorias.map(c => ({label: `${c.code} — ${c.name}`, value: c.id}))}
                                      placeholder={t('acl.placeholder.category')} onChange={e => setNovaRegra({...novaRegra, categoryId: e.value as number})}
                                      style={{minWidth: 260}} />
                            <InputText value={novaRegra.adGroup} placeholder={t('acl.placeholder.adGroupOrStar')}
                                       onChange={e => setNovaRegra({...novaRegra, adGroup: e.target.value})} />
                            <Dropdown value={novaRegra.action} options={ACAO_DROP()}
                                      onChange={e => setNovaRegra({...novaRegra, action: e.value})} />
                            <InputText value={String(novaRegra.priority)} style={{width: 90}}
                                       onChange={e => setNovaRegra({...novaRegra, priority: Number(e.target.value) || 0})} />
                            <InputText value={novaRegra.justification} placeholder={t('acl.placeholder.reason')}
                                       onChange={e => setNovaRegra({...novaRegra, justification: e.target.value})} />
                            <Button label={t('acl.createRule')} icon="pi pi-plus" onClick={criaRegra} disabled={busy} />
                        </div>
                        <DataTable value={regras} size="small" stripedRows rows={10} emptyMessage={t('acl.noRules')}>
                            <Column field="categoryCode" header={t('acl.category')} />
                            <Column field="adGroup" header={t('acl.group')} />
                            <Column header={t('acl.action')} body={(r: Regra) => acaoTag(r.action)} />
                            <Column field="priority" header={t('acl.priority')} />
                            <Column field="justification" header={t('acl.justification')} />
                            <Column header="" body={(r: Regra) =>
                                <Button icon="pi pi-trash" size="small" severity="danger" text
                                        onClick={() => remove('rules', r.id, t('acl.rules'))} />} />
                        </DataTable>
                        <p className="muted">
                            {t('acl.ruleOrder')}
                        </p>
                    </TabPanel>

                    <TabPanel header={`${t('acl.categories')} (${categorias.length})`}>
                        <DataTable value={categorias} size="small" stripedRows rows={10}>
                            <Column field="code" header={t('acl.default')==='DEFAULT'?'CODIGO':'CODIGO'} />
                            <Column field="name" header={t('acl.default')==='DEFAULT'?'NOME':'NOME'} />
                            <Column header={t('acl.default')} body={(c: Categoria) =>
                                <Tag value={c.blockedByDefault ? t('acl.default.byCategory') : t('acl.default.allowed')}
                                     severity={c.blockedByDefault ? 'danger' : 'success'} />} />
                            <Column field="source" header={t('acl.originCol')} />
                        </DataTable>
                        <p className="muted">
                            {t('acl.categoryDefault')}
                        </p>
                    </TabPanel>
                </TabView>
            </div>
        </div>
    );
}