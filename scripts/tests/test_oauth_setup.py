"""Configuration must preserve other OAuth clients and keep secrets on disk only."""
import importlib.util
from pathlib import Path
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT))
spec = importlib.util.spec_from_file_location('oauth_setup', ROOT / 'configure_oauth2.py')
setup = importlib.util.module_from_spec(spec)
spec.loader.exec_module(setup)


class OAuthSetupTest(unittest.TestCase):
    def test_registration_preserves_erp_and_reuses_hci_secret(self):
        original = [{'client_id': 'erp', 'client_secret': 'existing', 'redirect_uris': ['https://erp.example/callback']}]
        clients, client = setup.registration(original, 'https://192.168.2.10')
        self.assertEqual(clients[0], original[0])
        self.assertEqual(len(original), 1)
        self.assertGreaterEqual(len(client['client_secret']), 32)
        again, same = setup.registration(clients, 'https://192.168.2.10')
        self.assertEqual(again, clients)
        self.assertEqual(same['client_secret'], client['client_secret'])
        more, same = setup.registration(again, 'https://astral.example')
        self.assertEqual(len(more), 2)
        self.assertEqual(len(same['redirect_uris']), 2)
        self.assertEqual(same['client_secret'], client['client_secret'])

    def test_public_origin_rejects_plaintext_credentials_paths_and_injection(self):
        for uri in ['http://192.168.2.10', 'https://user:password@portal.example', 'https://portal.example/app/', 'https://portal.example/?next=x', 'https://portal.example/#x', 'https://portal.example\nBAD=1', 'https://portal.example:bad']:
            with self.subTest(uri=uri), self.assertRaises(ValueError):
                setup.public_url(uri)
        self.assertEqual(setup.public_url('https://192.168.2.10/'), 'https://192.168.2.10')
        self.assertEqual(setup.public_url('https://PORTAL.Example:443/'), 'https://portal.example')

    def test_env_secrets_are_private_atomic_and_roundtrip(self):
        with tempfile.TemporaryDirectory() as directory:
            env = Path(directory) / 'oauth.env'
            values = {'SECRET': 'a$`b"c\\d', 'ISSUER': 'https://192.168.2.10/auth'}
            setup.atomic_write(env, setup.env_text(values))
            self.assertEqual(env.stat().st_mode & 0o777, 0o600)
            self.assertEqual(setup.read_env(env), values)
            self.assertEqual(len(list(Path(directory).iterdir())), 1)


if __name__ == '__main__':
    unittest.main()
