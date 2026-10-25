#!/usr/bin/env python3
"""Compile and execute generated decoders in both layouts; no npm install here."""
import argparse, pathlib, subprocess, tempfile
p=argparse.ArgumentParser(); p.add_argument('--generator', required=True); p.add_argument('--tsc',required=True); a=p.parse_args()
root=pathlib.Path(__file__).resolve().parent
def run(*args): subprocess.run(list(map(str,args)),check=True)
for single in [False, True]:
    with tempfile.TemporaryDirectory() as temp:
        out=pathlib.Path(temp)
        run(a.generator,'ts','--inputs',root,'--output',out/'src','--runtime-decoders',*(['--single-file'] if single else []))
        run(a.tsc,'--target','ES2022','--module','commonjs','--strict','--noUnusedLocals','--noUnusedParameters','--skipLibCheck','--rootDir',out/'src','--outDir',out/'dist',*sorted((out/'src').rglob('*.ts')))
        run('node',root/'check.cjs',out/'dist/fixture/models/decoders.js')
