import React from 'react';
import {useTranslation} from 'react-i18next';
import {Card} from 'primereact/card';
import {Conf} from '../../../shared/components/Conf';
export default function AdPage({user}){const{t}=useTranslation();return <div className="grid"><Card title={t('ad.session')}><div className="metric">{user?.username||'—'}</div><p className="muted">{(user?.authorities||[]).map(a=>a.authority).join(', ')}</p></Card><Conf rows={[['Protocolo',t('ad.protocol')],['Base',t('ad.base')],['Grupo admin',t('ad.adminGroup')],['Fallback',t('ad.fallback')]]}/></div>;}