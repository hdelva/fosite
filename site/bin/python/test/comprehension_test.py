import sys
import os
sys.path.append(os.path.join(os.path.dirname(__file__), '..'))

from gts import *
from scan import Scan

code = """
[x ** 2 for x in range(1,3)]
{x for x in range(2, 101) if all(x%y for y in range(2, min(x, 11)))}
(x ** 2 for x in range(4,7))
[ord(c) for line in file for c in line]
"""

parseprint(code)

print('')

scanner = Scan()
tree = scanner.to_general_form(code)
print(tree.str())

