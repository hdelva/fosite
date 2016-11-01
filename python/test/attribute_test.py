import sys
import os
sys.path.append(os.path.join(os.path.dirname(__file__), '..'))

from gts import *
from scan import Scan

code = """
x.y
x.y = 1
x = z
x.y = 3

foo().x
"""

parseprint(code)

print('')

scanner = Scan()
tree = scanner.to_general_form(code)
print(tree.str())

