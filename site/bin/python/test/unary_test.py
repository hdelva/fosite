import sys
import os
import json
sys.path.append(os.path.join(os.path.dirname(__file__), '..'))

from gts import *
from scan import Scan
from gast import GastEncoder

"""
Tests assignments, additions and calls
"""

code = """
v = +8 
v = -9 
v = not True
v = ~5

"""

parseprint(code)

print('')

scanner = Scan()
tree = scanner.to_general_form(code)
print(json.dumps(tree, cls=GastEncoder))
print(tree.str())

