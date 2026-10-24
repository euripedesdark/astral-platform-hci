"""Exercise release activation and recovery without changing host services."""
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('deploy', Path(__file__).resolve().parents[2] / 'deploy_platform.py')
deploy = importlib.util.module_from_spec(spec)
spec.loader.exec_module(deploy)


class DeploymentTest(unittest.TestCase):
    def scenario(self, failure=None):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        root = Path(temporary.name)
        source = root / 'source'
        app = root / 'app'
        units = root / 'systemd'
        config = root / 'etc'
        for directory in [source / 'frontend', units, config, app / 'releases/old']:
            directory.mkdir(parents=True)
        for name in ['Cargo.lock', 'frontend/package-lock.json']:
            (source / name).write_text('lock')
        shared = config / 'astral.env'
        shared.write_text('KEEP_EXISTING_AUTH=yes\n')
        (units / deploy.LEGACY).write_text('old Java unit')
        (app / 'current').symlink_to(app / 'releases/old')
        runtime = config / 'runtime.env'
        runtime.write_text('SERVER_PORT=8081\n')
        commands = []
        active = {deploy.LEGACY}

        def systemctl(*args, check=True):
            commands.append(args)
            code = 0
            if args[0] == 'is-active':
                code = 0 if args[1] in active else 3
            elif args[0] == 'stop':
                active.discard(args[1])
            elif args[0] == 'start':
                active.add(args[1])
            elif args[0] == 'enable':
                wants = units / 'multi-user.target.wants'
                wants.mkdir(exist_ok=True)
                (wants / deploy.UNIT).symlink_to(units / deploy.UNIT)
            result = subprocess.CompletedProcess(args, code, '', '')
            if check and code:
                raise subprocess.CalledProcessError(code, args)
            return result

        def run(args, **kwargs):
            commands.append(tuple(map(str, args)))
            if failure == 'build' and args[:2] == ['cargo', 'test']:
                raise subprocess.CalledProcessError(1, args)
            if args[:2] == ['cargo', 'build']:
                binary = source / 'target/release/astral-platform-hci'
                binary.parent.mkdir(parents=True)
                binary.write_bytes(b'new binary')
                binary.chmod(0o755)
            if args == ['npm', 'run', 'build']:
                index = source / 'src/main/resources/static/app/index.html'
                index.parent.mkdir(parents=True)
                index.write_text('new frontend')
            return subprocess.CompletedProcess(args, 0, 'abc123' if args[0] == 'git' else '')

        original_is_dir = Path.is_dir
        with patch.multiple(deploy, ROOT=source, APP=app, UNIT_DIR=units, RUNTIME=runtime), \
                patch.object(deploy, 'run', side_effect=run), \
                patch.object(deploy, 'systemctl', side_effect=systemctl), \
                patch.object(deploy.shutil, 'which', return_value='/bin/tool'), \
                patch.object(deploy.os, 'geteuid', return_value=0), \
                patch.object(deploy.subprocess, 'check_output', side_effect=lambda args, **kw: 'v22.0.0' if args[0] == 'node' else 'abc123'), \
                patch.object(deploy.subprocess, 'run'), \
                patch.object(deploy, 'verify', side_effect=RuntimeError('unhealthy') if failure == 'health' else None), \
                patch.object(deploy.time, 'monotonic', side_effect=[0, 61]), \
                patch.object(Path, 'is_dir', lambda path: True if str(path) == '/run/systemd/system' else original_is_dir(path)), \
                patch('sys.argv', ['deploy_platform.py']):
            if failure:
                with self.assertRaises((RuntimeError, subprocess.CalledProcessError)):
                    deploy.main()
            else:
                deploy.main()
        self.assertEqual(shared.read_text(), 'KEEP_EXISTING_AUTH=yes\n')
        if failure:
            self.assertEqual((app / 'current').resolve(), app / 'releases/old')
            self.assertEqual((units / deploy.LEGACY).read_text(), 'old Java unit')
            self.assertEqual(runtime.read_text(), 'SERVER_PORT=8081\n')
            self.assertIn(deploy.LEGACY, active)
            self.assertFalse((units / deploy.UNIT).exists())
            self.assertFalse((units / 'multi-user.target.wants' / deploy.UNIT).exists())
        else:
            self.assertTrue((units / deploy.LEGACY).is_symlink())
            self.assertIn('SERVER_PORT=8082', runtime.read_text())
            release = (app / 'current').resolve()
            self.assertEqual((release / 'static/app/index.html').read_text(), 'new frontend')
            self.assertEqual((release / 'bin/astral-platform-hci').read_bytes(), b'new binary')
            self.assertEqual(json.loads((release / 'manifest.json').read_text())['commit'], 'abc123')
            self.assertIn(deploy.UNIT, active)
        return commands

    def test_build_failure_restarts_previous_service(self):
        commands = self.scenario('build')
        self.assertNotIn(('daemon-reload',), commands)

    def test_unhealthy_release_restores_unit_config_release_and_enablement(self):
        self.scenario('health')

    def test_success_deploys_backend_and_frontend_together(self):
        commands = self.scenario()
        self.assertLess(commands.index(('stop', deploy.LEGACY)), commands.index(('cargo', 'test', '--locked')))


if __name__ == '__main__':
    unittest.main()
