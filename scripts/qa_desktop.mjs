// Test only an owned Tauri app with an explicitly enabled temporary localhost CDP port.
// Native picker responses are stubbed; all application engine IPC remains real.
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {deflateSync,crc32} from 'node:zlib';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const argument=(name,fallback)=>{const i=process.argv.indexOf(name);return i<0?fallback:process.argv[i+1]};
const port=Number(argument('--port','9431')),project=path.resolve(argument('--project',path.join(root,'local-work/gui-project')));
const evidence=path.join(root,'assets/screenshots/desktop'),fixtures=path.join(root,'local-work/gui-fixtures');fs.mkdirSync(fixtures,{recursive:true});fs.mkdirSync(evidence,{recursive:true});
function png(file,w,h,color){const chunk=(kind,data)=>{const k=Buffer.from(kind),size=Buffer.alloc(4),crc=Buffer.alloc(4);size.writeUInt32BE(data.length);crc.writeUInt32BE(crc32(Buffer.concat([k,data])));return Buffer.concat([size,k,data,crc])};const header=Buffer.alloc(13);header.writeUInt32BE(w);header.writeUInt32BE(h,4);header[8]=8;header[9]=2;const row=Buffer.concat([Buffer.from([0]),Buffer.from(Array.from({length:w},()=>color).flat())]);fs.writeFileSync(file,Buffer.concat([Buffer.from([137,80,78,71,13,10,26,10]),chunk('IHDR',header),chunk('IDAT',deflateSync(Buffer.concat(Array.from({length:h},()=>row)))),chunk('IEND',Buffer.alloc(0))]));}
png(path.join(fixtures,'icon.png'),128,128,[101,83,216]);png(path.join(fixtures,'screen.png'),240,426,[242,237,252]);
if(argument('--icon'))fs.copyFileSync(path.resolve(argument('--icon')),path.join(fixtures,'icon.png'));
if(argument('--screenshot'))fs.copyFileSync(path.resolve(argument('--screenshot')),path.join(fixtures,'screen.png'));
const targets=await(await fetch(`http://127.0.0.1:${port}/json/list`)).json();const target=targets.find(t=>t.type==='page');if(!target)throw Error('Owned app has no page target');
const socket=new WebSocket(target.webSocketDebuggerUrl);await new Promise((resolve,reject)=>{socket.onopen=resolve;socket.onerror=reject});
let counter=0;const pending=new Map(),errors=[];
socket.onmessage=e=>{const data=JSON.parse(e.data);if(data.method==='Runtime.exceptionThrown')errors.push(data.params.exceptionDetails.text);if(data.id){const handler=pending.get(data.id);if(handler){pending.delete(data.id);data.error?handler.reject(Error(JSON.stringify(data.error))):handler.resolve(data.result)}}};
function call(method,params={}){return new Promise((resolve,reject)=>{const id=++counter;const timer=setTimeout(()=>{pending.delete(id);reject(Error(`CDP timeout: ${method}`))},120000);pending.set(id,{resolve:r=>{clearTimeout(timer);resolve(r)},reject:e=>{clearTimeout(timer);reject(e)}});socket.send(JSON.stringify({id,method,params}));})}
async function evaluate(expression){const result=await call('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true});if(result.exceptionDetails)throw Error(result.exceptionDetails.exception?.description||result.exceptionDetails.text);return result.result.value;}
const wait=async(expression,timeout=120000)=>{const end=Date.now()+timeout;while(Date.now()<end){if(await evaluate(expression))return;await new Promise(r=>setTimeout(r,150))}throw Error(`Condition timed out: ${expression}; status=${await evaluate("document.querySelector('#status')?.textContent")}`)};
const click=async(selector)=>{await evaluate(`document.querySelector(${JSON.stringify(selector)}).click()`);await new Promise(r=>setTimeout(r,100));};
const fill=async(selector,value)=>evaluate(`(()=>{const el=document.querySelector(${JSON.stringify(selector)});el.value=${JSON.stringify(value)};el.dispatchEvent(new Event('input',{bubbles:true}));})()`);
const choose=value=>evaluate(`window.__launchKitDialogSelections.push(${JSON.stringify(value)})`);
const idle=()=>wait("!document.body.classList.contains('busy')");
async function screenshot(name){const {cssContentSize:box}=await call('Page.getLayoutMetrics');const result=await call('Page.captureScreenshot',{format:'png',captureBeyondViewport:true,clip:{x:0,y:0,width:box.width,height:box.height,scale:1}});fs.writeFileSync(path.join(evidence,name),Buffer.from(result.data,'base64'));}
const checks=[];
try{
 await call('Runtime.enable');await call('Page.reload');await wait("!!document.querySelector('#new')");
 await evaluate('window.__launchKitDialogSelections=[]');
 await click('#new');await fill('#new-title',argument('--title','Little Steps'));await choose(project);await evaluate("document.querySelector('#new-form').requestSubmit()");await wait("document.querySelector('#save-state').textContent==='SAVED'");
 await fill('#tagline','A calmer way to organize your day');await fill('#package','com.example.steps');await click('#save');await idle();checks.push('Create project and save fields through real Rust IPC');
 await choose(path.join(fixtures,'icon.png'));await click('#icon');await idle();await wait("document.querySelector('#icon-state').textContent==='Image imported'");
 await choose(path.join(fixtures,'screen.png'));await click('#screenshot');await idle();await wait("!document.querySelector('#preview').hidden");checks.push('Import actual local PNG assets and render preview');
 await fill('.shot-row input','Make room for progress');await click('#save');await idle();
 for(const theme of ['paper','midnight','aurora']){await click(`[data-theme=${theme}]`);await click('#save');await idle();if(await evaluate("document.querySelector('#status').classList.contains('error')"))throw Error(await evaluate("document.querySelector('#status').textContent"));}checks.push('Edit captions and render all three themes');
 for(const kind of ['feature','social','icon','screenshot']){await evaluate(`(()=>{const el=document.querySelector('#preview-kind');el.value='${kind}';el.dispatchEvent(new Event('change'));})()`);await idle();await wait("document.querySelector('#preview').complete&&document.querySelector('#preview').naturalWidth>0");}checks.push('All four preview formats render');await screenshot('studio.png');
 await click('[data-page=export]');const output=path.join(path.dirname(project),path.basename(project)+'-export');await choose(output);await click('#dry-run');await idle();await wait("document.querySelector('#receipt-title').textContent==='Export plan checked.'");if(fs.existsSync(output))throw Error('Dry-run wrote files');checks.push('Desktop dry-run does not write output');
 await choose(output);await click('#export');await idle();await wait("document.querySelector('#receipt-title').textContent==='Your release pack is ready.'");if(!fs.existsSync(path.join(output,'SHA256SUMS.txt')))throw Error('Missing export');await screenshot('export.png');checks.push('Desktop export creates real artwork and checksums');
 await choose(output);await click('#export');await idle();await wait("document.querySelector('#status').classList.contains('error')");checks.push('Existing-output error is shown truthfully');
 await click('[data-page=agent]');await screenshot('agents.png');await click('[data-page=project]');if(errors.length)throw Error(errors.join('\n'));
 const report={schema_version:1,passed:true,checks,page_errors:errors,method:'Real Windows Tauri WebView and Rust IPC through localhost CDP; only native file-dialog selections stubbed. No interactive Linux/macOS GUI claim.'};fs.mkdirSync(path.join(root,'docs/verification'),{recursive:true});fs.writeFileSync(path.join(root,'docs/verification/desktop-smoke.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report,null,2));
}finally{socket.close();}
