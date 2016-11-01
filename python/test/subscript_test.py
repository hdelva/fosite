import sys
import os
sys.path.append(os.path.join(os.path.dirname(__file__), '..'))

from gts import *
from scan import Scan

code = """
a[1]
a[x]
a[1:2] = [6,7]
a[::-1]
"""

parseprint(code)

print('')

scanner = Scan()
tree = scanner.to_general_form(code)
print(tree.str())

