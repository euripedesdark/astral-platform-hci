import React, {useEffect, useState} from 'react';
import {api} from '../../../shared/services/api';

export default function ObsPage() {
 const [url,setUrl]=useState('');
 const [error,setError]=useState('');
 useEffect(()=>{
  let active=true;
  api<{url:string}>('/api/observability/dashboard').then(data=>{
   if(!active)return;
   if(!data.url){setError('Dashboard do Astral ainda não configurado.');return;}
   const target=new URL(data.url);
   target.searchParams.set('kiosk','');
   setUrl(target.toString());
  }).catch(()=>{if(active)setError('Não foi possível carregar o dashboard do Astral.');});
  return ()=>{active=false;};
 },[]);
 return <section aria-label="Dashboard de observabilidade do Astral" style={{width:'100%'}}>
  {error&&<p role="alert">{error}</p>}
  {!error&&!url&&<p>Carregando observabilidade…</p>}
  {url&&<iframe title="Astral — Grafana" src={url}
   style={{width:'100%',height:'calc(100vh - 180px)',minHeight:600,border:0,borderRadius:8}}
   allow="fullscreen" />}
 </section>;
}
