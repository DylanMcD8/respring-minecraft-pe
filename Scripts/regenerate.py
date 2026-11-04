"""Regenerate the checked-in terrain translations with the bundled 0.9.0 executable."""
import hashlib
import subprocess
import sys
import tempfile
import zipfile
from pathlib import Path

scripts = Path(__file__).resolve().parent
with tempfile.TemporaryDirectory() as work:
    temporary = Path(work)
    with zipfile.ZipFile(scripts.parent / 'Original/Minecraft.ipa') as archive:
        executable = archive.read('Payload/minecraftpe.app/minecraftpe')
    if hashlib.sha256(executable).hexdigest() != '6c23c1edba77d256250c45a7b82c52cc7f1db06b9ffd24645d6d56fc3608adc5':
        raise ValueError('Unexpected Minecraft executable')
    binary = temporary / 'minecraftpe'
    binary.write_bytes(executable)
    with (temporary / 'disassembly-armv7.txt').open('wb') as listing:
        subprocess.run(['xcrun', 'otool', '-arch', 'armv7', '-tvV', '-j', binary], stdout=listing, check=True)
    for script in ('generate-loading-blocks.py', 'generate-noise-block.py'):
        subprocess.run([sys.executable, scripts / script, temporary], check=True)
