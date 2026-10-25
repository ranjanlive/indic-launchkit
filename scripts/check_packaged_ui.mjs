// Smoke test an owned packaged WebView with a temporary, explicitly enabled CDP port.
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..');
const port=process.argv[2]||9432;
const targets=await(await fetch(`http://127.0.0.1:${port}/json/list`)).json();const target=targets.find(t=>t.type==='page');if(!target)throw Error('No owned packaged UI target');
const socket=new WebSocket(target.webSocketDebuggerUrl);await new Promise((resolve,reject)=>{socket.onopen=resolve;socket.onerror=reject});let id=0;const requests=new Map();
socket.onmessage=e=>{const r=JSON.parse(e.data),pending=requests.get(r.id);if(pending){requests.delete(r.id);r.error?pending.reject(Error(JSON.stringify(r.error))):pending.resolve(r.result)}};
function call(method,params={}){return new Promise((resolve,reject)=>{const number=++id;requests.set(number,{resolve,reject});socket.send(JSON.stringify({id:number,method,params}));})}
async function evaluate(expression){const r=await call('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true});if(r.exceptionDetails)throw Error(r.exceptionDetails.exception?.description||r.exceptionDetails.text);return r.result.value;}
const assert=(v,msg)=>{if(!v)throw Error(msg)};
try{
 assert(target.url.startsWith('http://tauri.localhost')||target.url.startsWith('tauri://'),'Expected packaged local origin');
 await evaluate('document.fonts.ready');const initial=await evaluate("({heading:document.querySelector('h1')?.textContent,buttons:document.querySelectorAll('button').length,fonts:[...document.fonts].map(f=>({family:f.family,status:f.status}))})");assert(initial.heading==='From app to launch.','Packaged UI did not load');assert(initial.fonts.some(f=>f.family==='Noto'&&f.status==='loaded'),'Bundled UI font did not load');
 await evaluate("document.querySelector('[data-page=agent]').click()");assert(await evaluate("!document.querySelector('#agent-page').hidden"),'Agent page failed');await evaluate("document.querySelector('[data-page=export]').click()");assert(await evaluate("!document.querySelector('#export-page').hidden"),'Export page failed');await evaluate("document.querySelector('[data-page=project]').click()");
 const apk=path.join(root,'local-work/packaged-smoke.apk');const result=await evaluate(`window.__TAURI_INTERNALS__.invoke('operation',{request:{action:'inspect',source:${JSON.stringify(apk)}}})`);assert(result.package_name==='com.example.launchkit','Packaged Rust IPC failed');
 const report={schema_version:1,passed:true,checks:['Packaged HTML and JavaScript loaded without a development server','Bundled Noto UI font loaded','Project, export and CLI navigation work','Real packaged Rust IPC parses a local APK fixture'],origin:target.url,limitations:'Native file picker and installer installation were not automated; no Linux/macOS interactive GUI claim.'};fs.writeFileSync(path.join(root,'docs/verification/packaged-windows-smoke.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report,null,2));
}finally{socket.close()}
