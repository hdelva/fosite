import sys
import os
sys.path.append(os.path.join(os.path.dirname(__file__), '..'))

from gts import *
from scan import Scan

"""
Tests function declarations, function calls
"""

code = """
def fun(v1, v2=4, *args, d, **kwargs): 
  v = v1
  x = 4
  v1 = x + 1
  v -= 1
  return v1
  
v = fun(v, d=v)
"""

parseprint(code)

print('')

scanner = Scan()
tree = scanner.to_general_form(code)
print(tree.str())