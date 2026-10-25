"""Collect actual downloaded dependency license/notice texts for native distribution."""
from pathlib import Path
import json,subprocess
ROOT=Path(__file__).resolve().parent.parent
def main():
    host=next(line.split(': ',1)[1] for line in subprocess.check_output(['rustc','-vV'],text=True).splitlines() if line.startswith('host: '))
    metadata=json.loads(subprocess.check_output(['cargo','metadata','--locked','--format-version','1','--filter-platform',host],cwd=ROOT/'source_code',text=True,encoding='utf-8'))
    sections=['# Bundled dependency notices\n\nGenerated from downloaded locked dependencies for '+host+'.\n\nProject MIT license and bundled Noto SIL OFL are distributed separately.\n']
    for package in sorted(metadata['packages'],key=lambda p:(p['name'],p['version'])):
        folder=Path(package['manifest_path']).parent
        if folder.is_relative_to(ROOT):continue
        sections.append(f"\n## Rust: {package['name']} {package['version']}\n\nDeclared license: {package.get('license') or 'See package license file'}\nSource: {package.get('repository') or package.get('source') or ''}\n")
        files=[p for p in folder.iterdir() if p.is_file() and p.name.lower().startswith(('license','licence','copying','notice','copyright'))]
        if package.get('license_file'):
            item=folder/package['license_file']
            if item.is_file() and item not in files:files.append(item)
        for file in sorted(files):
            sections.append('\n### '+file.name+'\n\n```text\n'+file.read_text(encoding='utf-8',errors='replace')+'\n```\n')
    npm=ROOT/'source_code/node_modules'
    for name in ['@tauri-apps/api','@tauri-apps/plugin-dialog']:
        folder=npm/name;package=json.loads((folder/'package.json').read_text(encoding='utf-8'));sections.append(f"\n## JavaScript: {name} {package['version']}\n\nDeclared license: {package.get('license','')}\n")
        for file in sorted(folder.iterdir()):
            if file.is_file() and file.name.lower().startswith(('license','licence','notice','copyright')):sections.append('\n### '+file.name+'\n\n```text\n'+file.read_text(encoding='utf-8',errors='replace')+'\n```\n')
    output=ROOT/'local-work/native/DEPENDENCY-LICENSES.md';output.parent.mkdir(parents=True,exist_ok=True);output.write_text('\n'.join(sections),encoding='utf-8');print(f'Collected notices for {len(metadata["packages"])} resolved Rust packages and bundled JavaScript APIs.')
if __name__=='__main__':main()
