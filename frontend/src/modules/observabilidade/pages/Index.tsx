import React from 'react';
import {Card} from 'primereact/card';
import {Conf} from '../../../shared/components/Conf';
export default function ObsPage(){return <div className="grid"><Conf rows={[['Observabilidade','Grafana :3000'],['Dados','PostgreSQL — banco astral'],['Health','/actuator/health na 8081']]}/><a href={`${window.location.protocol}//${window.location.hostname}:3000/d/astral-observability`} target="_blank" rel="noopener noreferrer">Abrir Grafana</a></div>;}
