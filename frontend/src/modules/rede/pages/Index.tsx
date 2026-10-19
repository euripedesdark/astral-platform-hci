import React, {useEffect, useState} from 'react';
import {useTranslation} from 'react-i18next';
import {Card} from 'primereact/card';
import {Conf} from '../../../shared/components/Conf';
import {api} from '../../../shared/services/api';
export default function RedePage(){
 const{t}=useTranslation();
 const[fw,setFw]=useState(null);
 useEffect(()=>{api('/api/firewall/status').then(setFw).catch(()=>{})},[]);
 return <div className="grid"><Card title={t('firewall.dashboard.activeRules')}><div className="metric">{fw?.rulesActive??'?'}</div><small className="muted">{t('network.synced')}</small></Card><Conf rows={[['Modelo',t('network.model')],['Tecido',t('network.fabric')]]}/></div>;
}