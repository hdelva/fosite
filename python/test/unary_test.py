import sys
import os
sys.path.append(os.path.join(os.path.dirname(__file__), '..'))

from gts import *
from scan import Scan

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
print(tree.str())

