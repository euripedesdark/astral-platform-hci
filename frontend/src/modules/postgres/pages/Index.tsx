import React from 'react';
import {useTranslation} from 'react-i18next';
import {Card} from 'primereact/card';
import {Conf} from '../../../shared/components/Conf';
export default function PostgresPage(){const{t}=useTranslation();return <div className="grid"><Conf rows={[['Host',t('postgres.host')],['Banco',t('postgres.database')],['Role',t('postgres.role')],['Auth',t('postgres.auth')]]}/><Card title="Nota"><p className="muted">{t('postgres.note')}</p></Card></div>;}