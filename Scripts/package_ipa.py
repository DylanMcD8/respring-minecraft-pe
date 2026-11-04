"""Package the unsigned Xcode device product for sideloading."""
import argparse
import hashlib
import plistlib
import subprocess
import tempfile
import zipfile
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('app', type=Path)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
info = plistlib.loads((args.app / 'Info.plist').read_bytes())
if info.get('CFBundleSupportedPlatforms') != ['iPhoneOS']:
    parser.error('Use the Release-iphoneos device product.')
binary = args.app / info['CFBundleExecutable']
if 'arm64' not in subprocess.check_output(['lipo', '-archs', binary], text=True).split():
    parser.error('Expected an ARM64 executable.')
if subprocess.run(['codesign', '-d', args.app], capture_output=True).returncode == 0:
    parser.error('Build with CODE_SIGNING_ALLOWED=NO before packaging.')
expected = Path(__file__).resolve().parents[1] / 'Original/Minecraft.ipa'
if (args.app / 'Minecraft.ipa').read_bytes() != expected.read_bytes():
    parser.error('The game payload differs from the repository copy.')
args.output.parent.mkdir(parents=True, exist_ok=True)
with tempfile.TemporaryDirectory(dir=args.output.parent) as tmp:
    archive = Path(tmp) / 'release.ipa'
    with zipfile.ZipFile(archive, 'w', zipfile.ZIP_DEFLATED) as out:
        for item in sorted(args.app.rglob('*')):
            if item.name in {'embedded.mobileprovision', '_CodeSignature', 'iTunesMetadata.plist'}:
                parser.error(f'Unexpected signing or account data in {item.name}')
            if item.is_file() and item.name != '.DS_Store' and not item.name.startswith('._'):
                out.write(item, Path('Payload') / args.app.name / item.relative_to(args.app))
    with zipfile.ZipFile(archive) as z:
        if z.testzip() is not None:
            parser.error('ZIP integrity check failed.')
    archive.replace(args.output)
print(hashlib.sha256(args.output.read_bytes()).hexdigest(), args.output.name)
