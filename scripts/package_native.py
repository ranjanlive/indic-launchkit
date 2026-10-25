"""Package only measured native outputs, with build identity and bundled licenses."""
from pathlib import Path
import argparse, hashlib, json, platform, shutil, subprocess, tarfile, zipfile
ROOT=Path(__file__).resolve().parent.parent
def main():
    parser=argparse.ArgumentParser();parser.add_argument('--platform',choices=['windows','linux','macos'],required=True);parser.add_argument('--version',default='0.1.0-beta.1');args=parser.parse_args()
    out=ROOT/'local-work/native';out.mkdir(parents=True,exist_ok=True);target=ROOT/'source_code/target/release';extension='.exe' if args.platform=='windows' else '';binary=target/f'launchkit{extension}'
    if not binary.is_file():raise SystemExit('CLI binary is missing')
    arch={'AMD64':'x64','x86_64':'x64','arm64':'arm64','aarch64':'arm64'}.get(platform.machine(),platform.machine())
    name=f'launchkit-{args.version}-{args.platform}-{arch}';stage=ROOT/'local-work'/name;stage.mkdir(parents=True,exist_ok=False)
    shutil.copy2(binary,stage/binary.name)
    for file in ['LICENSE','THIRD_PARTY_NOTICES.md']:shutil.copy2(ROOT/file,stage/file)
    shutil.copy2(ROOT/'assets/fonts/OFL.txt',stage/'FONT-LICENSE.txt');shutil.copy2(ROOT/'docs/CLI.md',stage/'CLI.md')
    if args.platform=='windows':
        archive=out/f'{name}.zip'
        with zipfile.ZipFile(archive,'w',zipfile.ZIP_DEFLATED) as z:
            for file in stage.iterdir():z.write(file,f'{name}/{file.name}')
    else:
        archive=out/f'{name}.tar.gz'
        with tarfile.open(archive,'w:gz') as z:z.add(stage,arcname=name)
    bundles=[]
    for pattern in ['nsis/*.exe','deb/*.deb','appimage/*.AppImage','dmg/*.dmg']:
        for file in (target/'bundle').glob(pattern):
            destination=out/file.name;shutil.copy2(file,destination);bundles.append(destination.name)
    if not bundles:raise SystemExit('No native desktop bundle was produced')
    commit=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()
    receipt={'schema_version':1,'version':args.version,'platform':args.platform,'architecture':arch,'system':platform.platform(),'commit':commit,'cli_archive':archive.name,'desktop_bundles':bundles,'signing':'unsigned; no Authenticode or Apple notarization','verification':'Core tests, CLI end-to-end smoke, frontend compilation, native build; GUI manual verification is recorded separately'}
    (out/f'build-{args.platform}-{arch}.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(receipt,indent=2))
if __name__=='__main__':main()
