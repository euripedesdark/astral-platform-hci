import React, {useEffect, useState} from 'react';
import {useTranslation} from 'react-i18next';
import {api} from '../../../shared/services/api';

export default function ObsPage() {
 const{t}=useTranslation();
 const [url,setUrl]=useState('');
 const [error,setError]=useState('');
 useEffect(()=>{
  let active=true;
  api<{url:string}>('/api/observability/dashboard').then(data=>{
   if(!active)return;
   if(!data.url){setError(t('observability.notConfigured'));return;}
   const target=new URL(data.url);
   target.searchParams.set('kiosk','');
   setUrl(target.toString());
  }).catch(()=>{if(active)setError(t('observability.loadError'));});
  return ()=>{active=false;};
 },[t]);
 return <section aria-label={t('observability.iframeTitle')} style={{width:'100%'}}>
  {error&&<p role="alert">{error}</p>}
  {!error&&!url&&<p>{t('observability.loading')}</p>}
  {url&&<iframe title={t('observability.iframeTitle')} src={url}
   style={{width:'100%',height:'calc(100vh - 180px)',minHeight:600,border:0,borderRadius:8}}
   allow="fullscreen" />}
 </section>;
}