import subprocess
import sys
import os

from ansi2html import Ansi2HTMLConverter

class Forsite(object):
    
    def __init__(self, code):
        with open(os.path.join('bin', 'raw.py'), 'w+') as f:
            print(code, file=f)
    
    def run(self, code=None):
        p = subprocess.Popen(os.path.join('bin', 'run.sh'), stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        result = p.wait()
        a, _ = p.communicate()
        stdout = a.decode("utf-8")
        conv = Ansi2HTMLConverter()
        stdout = stdout.replace('(B', '')
        stdout = conv.convert(stdout)
        return stdout 