#!/usr/bin/env python3
"""Require Swift: verify generated constructors from a distinct module."""
import argparse, pathlib, subprocess, tempfile
p=argparse.ArgumentParser(); p.add_argument('--generator',required=True); a=p.parse_args()
root=pathlib.Path(__file__).resolve().parent
def run(*args): subprocess.run(list(map(str,args)),check=True)
with tempfile.TemporaryDirectory() as temp:
    out=pathlib.Path(temp)
    run(a.generator,'swift','--inputs',root/'common.fl',root/'models.fl','--output',out/'src','--single-file')
    run('swiftc','-module-cache-path',out/'cache','-emit-module','-module-name','Fixture','-emit-module-path',out/'Fixture.swiftmodule',*sorted((out/'src').rglob('*.swift')))
    (out/'client.swift').write_text('''import Fixture
let child = Child(count: 3)
let entry = Entry(name: "你好", count: -1, large: 42, amount: 1.5, enabled: true, optionalName: nil, nested: [[child]], byName: ["one": child], state: .ready, event: .added(child))
''')
    run('swiftc','-module-cache-path',out/'cache','-typecheck','-I',out,out/'client.swift')
    print('Separate-module Swift constructor compilation passed')
