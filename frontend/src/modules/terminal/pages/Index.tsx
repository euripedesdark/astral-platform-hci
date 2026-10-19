import React from 'react';
import {useTranslation} from 'react-i18next';
import {Card} from 'primereact/card';
export default function TermPage(){const{t}=useTranslation();return <div className="grid"><Card title={t('terminal.title')}><p className="muted">{t('terminal.noWebBackend')}</p><code>{t('terminal.ssh')}</code></Card></div>;}