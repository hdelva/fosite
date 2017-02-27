import sys
import os
import json
sys.path.append(os.path.join(os.path.dirname(__file__), 'python'))

from gts import *
from scan import Scan
from gast import GastEncoder

with open(sys.argv[1]) as f:
	code = f.read()

scanner = Scan()
tree = scanner.to_general_form(code)
print(json.dumps(tree, sort_keys=True,
                       indent=4, separators=(',', ': '), 
                       cls=GastEncoder))

