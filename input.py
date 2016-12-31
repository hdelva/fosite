import sys
import os
import json
sys.path.append(os.path.join(os.path.dirname(__file__), 'python'))

from gts import *
from scan import Scan
from gast import GastEncoder


code = """
y = 42

if 'cond1':
	y.positive = 4
	y.negative = 4

if 'cond2':
	y.positive = 9
else:
	y.negative = 9

y.positive
y.negative
"""

scanner = Scan()
tree = scanner.to_general_form(code)
print(json.dumps(tree, sort_keys=True,
                       indent=4, separators=(',', ': '), cls=GastEncoder))

