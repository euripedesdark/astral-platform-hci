import React from 'react';
import {useTranslation} from 'react-i18next';
import {Card} from 'primereact/card';
import {Conf} from '../../../shared/components/Conf';
export default function ProxyPage(){const{t}=useTranslation();return <div className="grid"><Conf rows={[['Escuta',t('proxy.listen')],['Auth',t('proxy.auth')],['Modo',t('proxy.mode')],['Remap',t('proxy.remap')]]}/><Card title="Fluxo"><p className="muted">{t('proxy.flow')}</p></Card></div>;}