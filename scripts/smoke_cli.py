"""Portable, offline end-to-end CLI contract check using synthetic public fixtures."""
from pathlib import Path
import argparse, hashlib, json, struct, subprocess, tempfile, zipfile, zlib

def png(path,w,h,color):
    def chunk(kind,data):return struct.pack('>I',len(data))+kind+data+struct.pack('>I',zlib.crc32(kind+data)&0xffffffff)
    rows=(b'\x00'+bytes(color)*w)*h
    path.write_bytes(b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',w,h,8,2,0,0,0))+chunk(b'IDAT',zlib.compress(rows))+chunk(b'IEND',b''))

def manifest():
    strings=['manifest','package','com.example.launchkit','versionName','1.2.3','versionCode','uses-sdk','minSdkVersion','targetSdkVersion']
    data=b'';offsets=[]
    for text in strings:
        raw=text.encode();offsets.append(len(data));data+=bytes([len(text),len(raw)])+raw+b'\0'
    data+=b'\0'*((-len(data))%4)
    pool=struct.pack('<HHIIIIII',1,28,28+4*len(strings)+len(data),len(strings),0,0x100,28+4*len(strings),0)+b''.join(struct.pack('<I',i) for i in offsets)+data
    def attribute(name,value,kind=3):return struct.pack('<IIIHBBI',0xffffffff,strings.index(name),strings.index(value) if kind==3 else 0xffffffff,8,0,kind,strings.index(value) if kind==3 else value)
    def element(name,attributes):return struct.pack('<HHIII',0x102,16,36+len(attributes)*20,1,0xffffffff)+struct.pack('<IIHHHHHH',0xffffffff,strings.index(name),20,20,len(attributes),0,0,0)+b''.join(attributes)
    def end(name):return struct.pack('<HHIIIII',0x103,16,24,1,0xffffffff,0xffffffff,strings.index(name))
    body=pool+element('manifest',[attribute('package','com.example.launchkit'),attribute('versionName','1.2.3'),attribute('versionCode',7,0x10)])+element('uses-sdk',[attribute('minSdkVersion',24,0x10),attribute('targetSdkVersion',35,0x10)])+end('uses-sdk')+end('manifest')
    return struct.pack('<HHI',3,8,len(body)+8)+body

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--binary',type=Path,required=True);parser.add_argument('--report',type=Path);args=parser.parse_args();binary=args.binary.resolve();checks=[]
    def run(*argv,code=0):
        result=subprocess.run([str(binary),*map(str,argv),'--json'],capture_output=True,text=True,timeout=120)
        assert result.returncode==code,(argv,result.returncode,result.stderr,result.stdout)
        reply=json.loads(result.stdout);assert reply['schema_version']==1 and reply['ok']==(code==0),reply
        return reply.get('data',reply.get('error'))
    with tempfile.TemporaryDirectory(prefix='launchkit-smoke-') as folder:
        root=Path(folder);project=root/'project';output=root/'release'
        png(root/'icon.png',128,128,(101,83,216));png(root/'home.png',240,426,(240,235,254))
        with zipfile.ZipFile(root/'test.apk','w') as apk:apk.writestr('AndroidManifest.xml',manifest())
        info=run('inspect',root/'test.apk');assert info['package_name']=='com.example.launchkit' and info['version_name']=='1.2.3' and info['version_code']==7 and info['target_sdk']==35 and info['signing_status']=='not-verified';checks.append('Binary Android manifest metadata and SHA-256')
        run('init',project,'--title','Little Steps');run('add','--project',project,'--icon',root/'icon.png');run('add','--project',project,'--screenshot',root/'home.png','--caption','Make room for progress');run('add','--project',project,'--apk',root/'test.apk');run('validate','--project',project);checks.append('Portable project init/import/validate')
        plan=run('export','--project',project,'--output',output,'--dry-run','--include-apk');assert not output.exists();checks.append('Dry-run lists complete pack without writes')
        report=run('export','--project',project,'--output',output,'--include-apk');assert len(report['artifacts'])==len(plan['artifacts'])
        for item in report['artifacts']:assert hashlib.sha256((output/item['file']).read_bytes()).hexdigest()==item['sha256']
        checks.append('Exported artwork, copied APK, metadata and all receipt hashes')
        run('export','--project',project,'--output',output,code=1);checks.append('Existing export refused')
        run('preview','--project',project,'--output',root/'preview.png');assert (root/'preview.png').is_file();checks.append('Headless rendered preview')
        run('add','--project',project,code=1);run('unknown-command',code=2);run('templates');run('schema');checks.append('JSON envelopes and error exit codes')
    report={'schema_version':1,'passed':True,'checks':checks}
    if args.report:args.report.parent.mkdir(parents=True,exist_ok=True);args.report.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(report,indent=2))
if __name__=='__main__':main()
