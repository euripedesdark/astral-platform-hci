import React, {useState} from 'react';
import {useTranslation} from 'react-i18next';
import {Card} from 'primereact/card';
import {InputText} from 'primereact/inputtext';
import {Password} from 'primereact/password';
import {Button} from 'primereact/button';
export default function CertsPage(){
 const{t}=useTranslation();
 const[u,setU]=useState(''),[p,setP]=useState(''),[busy,setBusy]=useState(false),[msg,setMsg]=useState('');
 async function dl(){
  setBusy(true);setMsg('');
  try{
   const r=await fetch('/api/certs/ca/download',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({username:u,password:p})});
   if(r.status===401)throw new Error(t('certs.invalidCredentials'));
   if(r.status===503)throw new Error(t('certs.caNotIssued'));
   if(!r.ok)throw new Error(t('certs.failure')+' '+r.status);
   const blob=await r.blob();const a=document.createElement('a');
   a.href=URL.createObjectURL(blob);a.download='srvcloud-root-ca.crt';document.body.appendChild(a);a.click();a.remove();
   setMsg(t('certs.downloaded'));
  }catch(e){setMsg(e.message)}finally{setBusy(false)}
 }
 return <div className="grid"><Card title={t('certs.title')}><p className="muted">{t('certs.note')}</p>
  <div className="formrow"><InputText value={u} placeholder={t('certs.username')} onChange={e=>setU(e.target.value)}/></div>
  <div className="formrow"><Password value={p} placeholder={t('certs.password')} feedback={false} toggleMask onChange={e=>setP(e.target.value)} onKeyDown={e=>e.key==='Enter'&&dl()}/></div>
  <div className="formrow"><Button label={busy?t('certs.verifying'):t('certs.download')} icon="pi pi-download" disabled={busy||!u||!p} onClick={dl}/></div>
  {msg&&<p className="muted">{msg}</p>}</Card></div>;
}