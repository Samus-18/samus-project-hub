#!/usr/bin/env python3
"""Validate self-contained public GitHub Pages navigation and project assets."""
from pathlib import Path
from html.parser import HTMLParser
import json,re
ROOT=Path(__file__).resolve().parents[1]
SITE=ROOT/"docs"
class A(HTMLParser):
 def __init__(self):super().__init__();self.links=[]
 def handle_starttag(self,tag,attrs):
  d=dict(attrs)
  key="href" if tag in ("a","link") else ("src" if tag in ("img","script") else None)
  if key and d.get(key):self.links.append(d[key])
parser=A();parser.feed((SITE/"index.html").read_text(encoding="utf8"))
for src in parser.links:
 if src.startswith(("http://","https://","#","mailto:")):continue
 path=(SITE/src.split("?")[0].split("#")[0]).resolve()
 assert path.is_relative_to(SITE.resolve()) and path.is_file(),src
assert (SITE/".nojekyll").exists()
assert (SITE/"app.js").is_file() and (SITE/"styles.css").is_file()
for name in ("aurora-chat.png","horizon-home.png","thalvoryn-pack.png"):
 data=(SITE/"assets"/name).read_bytes()
 assert data[:8]==b"\x89PNG\r\n\x1a\n" and len(data)>1000,name
projects=json.loads((ROOT/"projects.json").read_text(encoding="utf8"))
assert len(projects["projects"])==3
for project in projects["projects"]:
 assert (ROOT/"projects"/project["slug"]/"README.md").is_file()
for p in ROOT.rglob("*"):
 if not p.is_file() or ".git" in p.parts:continue
 assert p.suffix.lower() not in (".exe",".dll",".pdb",".rar",".zip"),p
 assert "node_modules" not in p.parts and "private_chat_exports" not in p.parts,p
 if p.stat().st_size<=2_000_000 and p.suffix.lower() in (".js",".rs",".json",".md",".txt",".yml"):
  text=p.read_text(encoding="utf8",errors="replace")
  assert not re.search(r"gh[pousr]_[A-Za-z0-9_]{30,}|github_pat_[A-Za-z0-9_]{30,}|sk-proj-[A-Za-z0-9_-]{25,}",text),p
print("SITE_CHECK_PASS",len(parser.links),"links",len(projects["projects"]),"projects")
