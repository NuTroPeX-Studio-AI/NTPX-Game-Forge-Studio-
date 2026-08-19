#!/usr/bin/env python3
from __future__ import annotations
import argparse, fnmatch, hashlib, json, os, re, subprocess, sys
from dataclasses import dataclass, asdict
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
CONFIG=ROOT/'config'/'fortress-bootstrap.json'
OUT=ROOT/'.ntpx'/'fortress'; REPORT=OUT/'bootstrap-report.json'; MANIFEST=OUT/'bootstrap-manifest.sha256'
TEXT_LIMIT=2*1024*1024
SECRET_PATTERNS={
 'github_token':re.compile(r'\bgh[pousr]_[A-Za-z0-9_]{30,}\b'),
 'aws_access_key':re.compile(r'\bAKIA[0-9A-Z]{16}\b'),
 'generic_bearer':re.compile(r'(?i)\bBearer\s+[A-Za-z0-9._~+/=-]{24,}'),
 'private_key_block':re.compile(r'-----BEGIN (?:RSA |EC |OPENSSH |DSA )?PRIVATE KEY-----')}
ASSIGNMENT=re.compile(r'(?i)\b(api[_-]?key|secret|token|password|passwd|client[_-]?secret)\b\s*[:=]\s*[\'\"]([^\'\"\s]{20,})[\'\"]')
PLACEHOLDERS=('example','sample','placeholder','changeme','your_','dummy','test-only')
@dataclass
class Finding: severity:str; rule:str; path:str; detail:str

def cfg(): return json.loads(CONFIG.read_text())
def head():
 try:return subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True,stderr=subprocess.DEVNULL).strip()
 except Exception:return 'unknown'
def files(c):
 excluded=set(c.get('excluded_directories',[]))
 for base,dirs,names in os.walk(ROOT):
  rel=Path(base).relative_to(ROOT); dirs[:]=[d for d in dirs if d not in excluded]
  if rel.parts and rel.parts[0]=='.ntpx': continue
  for n in names: yield Path(base)/n
def read_text(p):
 try:
  if p.stat().st_size>TEXT_LIMIT:return None
  raw=p.read_bytes()
  if b'\x00' in raw[:4096]:return None
  return raw.decode('utf-8',errors='replace')
 except OSError:return None
def placeholder(v): return any(x in v.lower() for x in PLACEHOLDERS)
def scan(c):
 findings=[]; manifest=[]; policy=set(c.get('policy_files',[])); allowed=set(c.get('allowed_sensitive_examples',[]))
 for p in files(c):
  rel=p.relative_to(ROOT).as_posix()
  try: manifest.append((hashlib.sha256(p.read_bytes()).hexdigest(),rel))
  except OSError as e: findings.append(Finding('high','read_failure',rel,str(e))); continue
  name=p.name
  if name not in allowed and any(fnmatch.fnmatch(name,pat) for pat in c.get('sensitive_filename_globs',[])):
   findings.append(Finding('critical','sensitive_filename',rel,'Sensitive credential/key file is not allowed in Git.'))
  text=read_text(p)
  if text is None: continue
  if rel not in policy:
   for marker in c.get('crown_jewel_markers',[]):
    if marker in text: findings.append(Finding('critical','classification_boundary',rel,f'L2-max repository contains marker {marker}.'))
  for rule,pat in SECRET_PATTERNS.items():
   if pat.search(text): findings.append(Finding('critical',rule,rel,'High-confidence secret/private-key pattern detected.'))
  for m in ASSIGNMENT.finditer(text):
   if not placeholder(m.group(2)): findings.append(Finding('high','secret_assignment',rel,f'Potential hard-coded credential assigned to {m.group(1)}.'))
 manifest.sort(key=lambda x:x[1]); return findings,manifest
def write(findings,manifest,mode,c):
 OUT.mkdir(parents=True,exist_ok=True); MANIFEST.write_text(''.join(f'{s}  {p}\n' for s,p in manifest))
 critical=sum(f.severity=='critical' for f in findings); high=sum(f.severity=='high' for f in findings); blocked=critical>0 or high>0 or mode=='release'
 result={'schema':'ntpx-fortress-bootstrap-report/v1','project':c.get('project'),'git_head':head(),'mode':mode,'files_hashed':len(manifest),'critical_findings':critical,'high_findings':high,'release_gate':'BLOCK' if blocked else 'OPEN','release_gate_reason':'Full Fortress external trust-chain activation is required before release mode can open.' if mode=='release' and critical==0 and high==0 else 'Security findings require remediation.' if blocked else 'Bootstrap monitor checks passed.','findings':[asdict(f) for f in findings]}
 REPORT.write_text(json.dumps(result,indent=2,sort_keys=True)+'\n'); return result
def main():
 ap=argparse.ArgumentParser(); ap.add_argument('--mode',choices=('monitor','release'),default='monitor'); a=ap.parse_args(); c=cfg(); f,m=scan(c); r=write(f,m,a.mode,c); print(json.dumps(r,indent=2)); return 30 if r['release_gate']=='BLOCK' else 0
if __name__=='__main__': sys.exit(main())
