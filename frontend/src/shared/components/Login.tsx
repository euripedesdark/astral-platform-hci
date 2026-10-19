import React, {useRef, useState} from 'react';
import {useTranslation} from 'react-i18next';
import {Dropdown} from 'primereact/dropdown';
import i18n,{supportedLanguages}from'../../i18n';
import {api, login, me} from '../services/api';
const HEXA=<svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg"><polygon points="50,5 95,25 95,75 50,95 5,75 5,25" fill="none" stroke="#00d2ff" strokeWidth="4"/><path d="M50,25 L75,70 L65,70 L50,40 L35,70 L25,70 Z" fill="#00d2ff"/></svg>;
const USERI=<svg className="input-icon" viewBox="0 0 24 24"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"></path><circle cx="12" cy="7" r="4"></circle></svg>;
const LOCKI=<svg className="input-icon" viewBox="0 0 24 24"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect><path d="M7 11V7a5 5 0 0 1 10 0v4"></path></svg>;
export default function Login({onAuth}){
 const{t}=useTranslation();
 const[u,setU]=useState(''),[p,setP]=useState(''),[busy,setBusy]=useState(false),[err,setErr]=useState('');
 const flying=useRef(false);
 async function go(e){
  e&&e.preventDefault();
  if(flying.current)return;flying.current=true;setBusy(true);setErr('');
  try{
   const data=await login(u,p);
   if(!(data&&(data.token||data.authenticated===true)))throw new Error('bad-response');
   if(data.token){document.cookie='astral_token='+encodeURIComponent(data.token)+'; path=/';try{localStorage.setItem('astral_token',data.token)}catch(_){}}
   await me();
   onAuth(data);
  }catch(ex){setErr(t('login.invalidCredentials'));}
  finally{flying.current=false;setBusy(false)}
 }
 return <div className="lg-page"><div className="brand-title">{t('app.name')}</div>
  <div className="login-box"><div className="center-logo">{HEXA}</div>
   <div style={{marginBottom:12}}><Dropdown value={i18n.resolvedLanguage||i18n.language} options={supportedLanguages} onChange={e=>i18n.changeLanguage(e.value)} style={{width:'100%'}} aria-label={t('login.language')}/></div>
   <button type="button" className="btn-submit" onClick={async()=>{
    try{const sso=await api<{enabled:boolean;authorizationUrl:string}>('/api/auth/sso');if(sso.enabled)window.location.assign(sso.authorizationUrl);else setErr(t('login.ssoNotConfigured'));}
    catch(_){setErr(t('login.ssoUnavailable'));}
   }}>{t('login.ssoButton')}</button>
   {new URLSearchParams(window.location.search).has('ssoError')&&<p role="alert">{t('login.ssoFailed')}</p>}
   <form id="loginForm" onSubmit={go}>
    <div className="input-group"><label htmlFor="username">{t('login.username')}</label>
     <div className="input-wrapper"><input type="text" id="username" autoComplete="off" required value={u} onChange={e=>setU(e.target.value)}/>{USERI}</div></div>
    <div className="input-group"><label htmlFor="password">{t('login.password')}</label>
     <div className="input-wrapper"><input type="password" id="password" required value={p} onChange={e=>setP(e.target.value)}/>{LOCKI}</div></div>
    <div className="divider"></div>
    <button type="submit" className="btn-submit" id="submitBtn" disabled={busy}>{busy?t('login.authenticating'):t('login.submit')}</button>
    <div className="error-msg" id="errorMsg" style={err?{display:'block'}:{}}>{err||t('login.invalidCredentials')}</div>
   </form></div></div>;
}