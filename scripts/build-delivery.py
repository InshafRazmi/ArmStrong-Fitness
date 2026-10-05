#!/usr/bin/env python3
"""Build and verify credential-free source delivery from this checkout."""
import hashlib
import json
import re
from pathlib import Path
import subprocess
import tempfile
import zipfile

ROOT = Path(__file__).resolve().parents[1]
EXTENSIONS = {'.rs', '.sql', '.json', '.toml', '.lock', '.yaml', '.yml', '.mjs',
              '.js', '.ts', '.tsx', '.mts', '.cts', '.css', '.md', '.py', '.html', '.desktop',
              '.xml', '.svg', '.png', '.ico', '.example'}
SPECIAL = {'.gitignore', 'Procfile', 'PKGBUILD', 'LICENSE'}
FORBIDDEN = {'node_modules', 'target', 'dist', 'dist-linux', 'certs', '.temp',
             '.git', '.agents', '.codex', '.aws'}


def run(args, cwd=ROOT):
    return subprocess.run(args, cwd=cwd, check=True, capture_output=True).stdout


def allowed(name):
    p = Path(name)
    return (name == '.gitignore' or p.parts[0] in {'Client', 'server', '.github', 'scripts'}) and not any(
        part in FORBIDDEN for part in p.parts) and not any(
        part.startswith('.env') and part != '.env.example' for part in p.parts) and (
        p.suffix in EXTENSIONS or p.name in SPECIAL) and p.name != 'desktop-auth.json'


def digest(data):
    return hashlib.sha256(data).hexdigest()


def archive(path, entries):
    path.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(path, 'w', zipfile.ZIP_DEFLATED) as z:
        for name, data in sorted(entries.items()):
            z.writestr(name, data)
    with zipfile.ZipFile(path) as z:
        if z.testzip() is not None or set(z.namelist()) != set(entries):
            raise RuntimeError('Delivery archive verification failed')
        for name, data in entries.items():
            if digest(z.read(name)) != digest(data):
                raise RuntimeError('Delivery archive checksum mismatch')
    path.with_suffix(path.suffix + '.sha256').write_text(f'{digest(path.read_bytes())}  {path.name}\n')


baseline = run(['git', 'rev-parse', 'HEAD']).decode().strip()
tracked = set(run(['git', 'ls-files', '-z']).decode().rstrip('\0').split('\0'))
all_names = set(run(['git', 'ls-files', '--cached', '--others', '--exclude-standard', '-z']).decode().rstrip('\0').split('\0'))
names = sorted(name for name in all_names if name and allowed(name))
current = {}
for name in names:
    path = ROOT / name
    if path.is_symlink():
        raise RuntimeError('Source delivery refuses symlinks')
    if path.is_file():
        data = path.read_bytes()
        if re.search(rb'-----BEGIN (?:RSA )?PRIVATE KEY-----\r?\n[A-Za-z0-9+/=\r\n]{64,}-----END (?:RSA )?PRIVATE KEY-----', data):
            raise RuntimeError('Source delivery refuses private keys')
        current[name] = data

with tempfile.TemporaryDirectory(prefix='armstrong-delivery-') as tmp:
    staging = Path(tmp) / 'staging'
    staging.mkdir()
    run(['git', 'init', '-q'], staging)
    base = {}
    for name in sorted(tracked):
        if allowed(name):
            base[name] = run(['git', 'show', f'{baseline}:{name}'])
            path = staging / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(base[name])
    run(['git', 'add', '--all'], staging)
    run(['git', '-c', 'user.name=ArmStrong delivery', '-c', 'user.email=delivery@example.invalid',
         'commit', '-q', '-m', 'Verified source baseline'], staging)
    for name in base.keys() - current.keys():
        (staging / name).unlink()
    for name, data in current.items():
        path = staging / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
    run(['git', 'add', '--all'], staging)
    changed = run(['git', 'diff', '--cached', '--name-only', '-z'], staging).decode().rstrip('\0').split('\0')
    changed = [name for name in changed if name]
    patch = run(['git', 'diff', '--cached', '--no-ext-diff', '--binary', '--full-index'], staging)
    patch_file = Path(tmp) / 'changes.patch'
    patch_file.write_bytes(patch)
    check = Path(tmp) / 'application-check'
    check.mkdir()
    run(['git', 'init', '-q'], check)
    for name, data in base.items():
        path = check / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
    run(['git', 'apply', '--check', str(patch_file)], check)
    run(['git', 'apply', str(patch_file)], check)
    for name in changed:
        path = check / name
        if name in current and path.read_bytes() != current[name]:
            raise RuntimeError('Applied source does not match delivery')
        if name not in current and path.exists():
            raise RuntimeError('Deleted source survived patch application')

manifest = {'baseline': baseline, 'files': {name: digest(current[name]) if name in current else None for name in changed}}
guide = f'''ArmStrong source update
Baseline: {baseline}

Extract this ZIP outside your writable repository checkout. Preserve other work.
Run git apply --check /path/to/changes.patch, then git apply /path/to/changes.patch.
Review and commit the resulting source, then push through your normal workflow.
The included source files and manifest provide an independent review copy.
Private env files, credentials, certificates, databases and build outputs are excluded.
See Client/docs/DELIVERY.md and Client/STATUS.md for deployment and remaining gates.
'''.encode()
entries = {'changes.patch': patch, 'MANIFEST.json': json.dumps(manifest, indent=2).encode(), 'README.txt': guide}
entries.update({f'source/{name}': current[name] for name in changed if name in current})
archive(ROOT / 'Client/armstrong-desktop-changes.zip', entries)

server_entries = {name: data for name, data in current.items() if name.startswith('server/')}
server_entries['MANIFEST.json'] = json.dumps({'baseline': baseline, 'files': {name: digest(data) for name, data in server_entries.items()}}, indent=2).encode()
archive(ROOT / 'server/armstrong-render-source.zip', server_entries)
print(f'PASS source delivery: {len(changed)} changes; patch checked/applied against {baseline}; ZIP CRC and all SHA-256s verified')
print(f'PASS standalone API source: {len(server_entries)-1} allowlisted files; no private env, certificate, database or build output')
