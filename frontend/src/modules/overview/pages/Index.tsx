import React, {useEffect, useState} from 'react';
import {useTranslation} from 'react-i18next';
import {Card} from 'primereact/card';
import {api} from '../../../shared/services/api';
export const MODS=[
 {id:'overview',did:'overview',img:'/images/firewall.png',descKey:'overview.mods.firewall.desc',titleKey:'overview.mods.firewall.title'},
 {id:'firewall',did:'firewall',img:'/images/firewall.png',descKey:'overview.mods.firewall.desc',titleKey:'overview.mods.firewall.title'},
 {id:'acl',did:'acl',img:'/images/proxy_system.png',descKey:'overview.mods.acl.desc',titleKey:'overview.mods.acl.title'},
 {id:'trafficServer',did:'proxy',img:'/images/proxy_system.png',descKey:'overview.mods.proxy.desc',titleKey:'overview.mods.proxy.title'},
 {id:'activeDirectory',did:'domain',img:'/images/domain_controllers.png',descKey:'overview.mods.ad.desc',titleKey:'overview.mods.ad.title'},
 {id:'postgres',did:'postgres',img:'/images/postgresql_admin.png',descKey:'overview.mods.postgres.desc',titleKey:'overview.mods.postgres.title'},
 {id:'observability',did:'web',img:'/images/web_server_admin.png',descKey:'overview.mods.observability.desc',titleKey:'overview.mods.observability.title'},
 {id:'compute',did:'vm',img:'/images/virtual_machines.png',descKey:'overview.mods.compute.desc',titleKey:'overview.mods.compute.title'},
 {id:'network',did:'network',img:'/images/network_config_vlan.png',descKey:'overview.mods.network.desc',titleKey:'overview.mods.network.title'},
 {id:'addressing',did:'enderecamento',img:'/images/network_config_vlan.png',descKey:'overview.mods.addressing.desc',titleKey:'overview.mods.addressing.title'},
 {id:'certs',did:'postgres',img:'/images/login.png',descKey:'overview.mods.certs.desc',titleKey:'overview.mods.certs.title'},
 {id:'terminal',did:'terminal',img:'/images/terminal.jpeg',descKey:'overview.mods.terminal.desc',titleKey:'overview.mods.terminal.title'},
];
export default function OverviewGo({setSection}){
 const{t}=useTranslation();
 const[fw,setFw]=useState(null);
 useEffect(()=>{api('/api/firewall/status').then(setFw).catch(()=>{})},[]);
 return <div><div className="grid">{MODS.map(m=><Card key={m.id} title={t(m.titleKey)} className="modcard" data-id={m.did} onClick={()=>setSection(m.id)}><img src={m.img} alt={t(m.titleKey)} className="modimg" onError={(e:React.SyntheticEvent<HTMLImageElement>)=>{e.currentTarget.style.display='none'}}/><p className="muted">{t(m.descKey)}</p>{m.id==='firewall'&&<small className="muted">{fw?.active?t('overview.firewallActive'):'—'} · {fw?.rulesActive??'?'} {t('overview.rules')}</small>}</Card>)}</div></div>;
}