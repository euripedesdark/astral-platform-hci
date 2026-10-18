import React from 'react';

export default function ObsPage() {
 return <section aria-label="Dashboard de observabilidade do Astral" style={{width:'100%'}}>
  <iframe title="Astral — Grafana" src="/grafana/d/astral-observability?orgId=1&kiosk"
   style={{width:'100%',height:'calc(100vh - 180px)',minHeight:600,border:0,borderRadius:8}}
   allow="fullscreen" />
 </section>;
}
