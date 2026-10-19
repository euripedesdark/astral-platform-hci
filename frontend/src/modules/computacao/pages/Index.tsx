import React from 'react';
import {useTranslation} from 'react-i18next';
import {Conf} from '../../../shared/components/Conf';
export default function CompPage(){const{t}=useTranslation();return <div className="grid"><Conf rows={[['Virtualização',t('compute.virtualization')],['Contêineres',t('compute.containers')],['Regra',t('compute.rule')]]}/></div>;}