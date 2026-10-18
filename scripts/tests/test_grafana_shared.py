import importlib.util
import os
from pathlib import Path
import unittest
from unittest.mock import patch

SCRIPT = Path(__file__).resolve().parents[1] / 'provision-grafana-astral.py'

class SharedGrafanaTest(unittest.TestCase):
    def load(self):
        with patch.dict(os.environ, {'ASTRAL_GRAFANA_URL': 'https://grafana.example/grafana',
                                    'ASTRAL_GRAFANA_SERVICE_TOKEN': 'test-token'}):
            spec = importlib.util.spec_from_file_location('provision', SCRIPT)
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
            return module

    def test_new_resources_never_change_default_or_erp(self):
        module = self.load()
        calls = []
        def api(method, path, data=None, missing=False):
            calls.append((method, path, data))
            if path == '/folders/astral-observability': return {'title': 'Astral'}
            return None
        with patch.object(module, 'api', side_effect=api), patch.dict(os.environ, {'ASTRAL_GRAFANA_READER_PASSWORD': 'test'}):
            module.main()
        writes = [(p,d) for m,p,d in calls if m != 'GET']
        self.assertEqual(['/datasources', '/dashboards/db'], [p for p,_ in writes])
        self.assertFalse(writes[0][1]['isDefault'])
        self.assertEqual('astral-postgres', writes[0][1]['uid'])
        self.assertEqual('astral-observability', writes[1][1]['dashboard']['uid'])
        self.assertEqual('astral-observability', writes[1][1]['folderUid'])

    def test_dashboard_collision_aborts_before_datasource_write(self):
        module = self.load()
        calls = []
        def api(method, path, data=None, missing=False):
            calls.append((method,path))
            if path.startswith('/folders/'): return {'title': 'Astral'}
            if path.startswith('/dashboards/'): return {'meta': {'folderUid': 'erp'}}
            return None
        with patch.object(module, 'api', side_effect=api), patch.dict(os.environ, {'ASTRAL_GRAFANA_READER_PASSWORD': 'test'}):
            with self.assertRaises(SystemExit): module.main()
        self.assertTrue(all(method == 'GET' for method,_ in calls))

if __name__ == '__main__': unittest.main()
