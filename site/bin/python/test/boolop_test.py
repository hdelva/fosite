import sys
import os
sys.path.append(os.path.join(os.path.dirname(__file__), '..'))

from gts import *
from scan import Scan

code = """
x and y
x or y

x and y or not x and not y

x == y
x != y 
x < y
x <= y
x > y
x >= y 
x is y 
x is not y
x in y 
x not in y
"""

parseprint(code)

print('')

scanner = Scan()
tree = scanner.to_general_form(code)
print(tree.str())

