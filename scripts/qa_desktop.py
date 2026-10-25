"""Real Tauri WebView integration smoke. Native file pickers are stubbed; Rust IPC is real.

Run only against an owned development/test app opened with a temporary localhost CDP port.
The app stays open for visual review; the caller owns its shutdown and process audit.
"""
from pathlib import Path
import argparse,json,sys
from playwright.sync_api import sync_playwright
from smoke_cli import png
ROOT=Path(__file__).resolve().parent.parent
def main():
 p=argparse.ArgumentParser();p.add_argument('--port',type=int,default=9431);p.add_argument('--project',type=Path,required=True);a=p.parse_args();project=a.project.resolve();project.parent.mkdir(parents=True,exist_ok=True);evidence=ROOT/'assets/screenshots/desktop';evidence.mkdir(parents=True,exist_ok=True)
 fixtures=ROOT/'local-work/gui-fixtures';fixtures.mkdir(parents=True,exist_ok=True);png(fixtures/'icon.png',128,128,(101,83,216));png(fixtures/'screen.png',240,426,(242,237,252))
 checks=[];errors=[]
 with sync_playwright() as pw:
  browser=pw.chromium.connect_over_cdp(f'http://127.0.0.1:{a.port}');page=browser.contexts[0].pages[0];page.on('pageerror',lambda e:errors.append(str(e)));page.reload();page.wait_for_selector('#new')
  # Intercept only native dialog selections, never product engine results.
  page.evaluate('''() => {window.__testSelections=[];const original=window.__TAURI_INTERNALS__.invoke.bind(window.__TAURI_INTERNALS__);window.__TAURI_INTERNALS__.invoke=(command,args,options)=>command.startsWith('plugin:dialog|')?Promise.resolve(window.__testSelections.shift()??null):original(command,args,options);}''')
  def choose(path):page.evaluate('(p)=>window.__testSelections.push(p)',str(path))
  page.locator('#new').click();page.locator('#new-title').fill('Little Steps');choose(project);page.locator('#new-form button[type=submit],#new-form button:not([type])').click();page.wait_for_function("document.querySelector('#save-state').textContent==='SAVED'")
  page.locator('#tagline').fill('A calmer way to organize your day');page.locator('#package').fill('com.example.steps');page.locator('#save').click();page.wait_for_function("document.querySelector('#save-state').textContent==='SAVED'");checks.append('Create project and save edited fields through real Rust IPC')
  choose(fixtures/'icon.png');page.locator('#icon').click();page.wait_for_function("document.querySelector('#icon-state').textContent==='Image imported'")
  choose(fixtures/'screen.png');page.locator('#screenshot').click();page.wait_for_function("!document.querySelector('#preview').hidden",timeout=120000);checks.append('Import and render actual local images')
  page.locator('.shot-row input').fill('Make room for progress');page.locator('#save').click();page.wait_for_function("!document.body.classList.contains('busy')",timeout=120000)
  for theme in ['paper','midnight','aurora']:
   page.locator(f'[data-theme={theme}]').click();page.locator('#save').click();page.wait_for_function("!document.body.classList.contains('busy')",timeout=120000);assert not page.locator('#status').evaluate('(e)=>e.classList.contains("error")')
  checks.append('Edit captions and render all three themes')
  for kind in ['feature','social','icon','screenshot']:
   page.locator('#preview-kind').select_option(kind);page.wait_for_function("!document.body.classList.contains('busy')",timeout=120000);assert page.locator('#preview').evaluate('(e)=>e.complete&&e.naturalWidth>0')
  checks.append('All four preview formats produce real rendered images')
  page.screenshot(path=str(evidence/'studio.png'),full_page=True)
  page.locator('[data-page=export]').click();output=project.parent/'gui-export';choose(output);page.locator('#dry-run').click();page.wait_for_function("document.querySelector('#receipt-title').textContent==='Export plan checked.'",timeout=120000);assert not output.exists();checks.append('Desktop dry-run does not write output')
  choose(output);page.locator('#export').click();page.wait_for_function("document.querySelector('#receipt-title').textContent==='Your release pack is ready.'",timeout=120000);assert (output/'SHA256SUMS.txt').is_file();page.screenshot(path=str(evidence/'export.png'),full_page=True);checks.append('Desktop export creates artwork and checksums')
  choose(output);page.locator('#export').click();page.wait_for_function("document.querySelector('#status').classList.contains('error')");checks.append('Desktop existing-output error shown truthfully')
  page.locator('[data-page=agent]').click();page.screenshot(path=str(evidence/'agents.png'),full_page=True);page.locator('[data-page=project]').click()
  assert not errors,errors
  report={'schema_version':1,'passed':True,'checks':checks,'page_errors':errors,'method':'Real Windows Tauri WebView and Rust IPC through CDP; native file-dialog selection stubbed for repeatability; no interactive Linux/macOS GUI claim'}
  (ROOT/'docs/verification/desktop-smoke.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps(report,indent=2))
  browser.close() # disconnects CDP; app lifecycle belongs to caller
if __name__=='__main__':main()
