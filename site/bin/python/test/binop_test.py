import sys
import os
sys.path.append(os.path.join(os.path.dirname(__file__), '..'))

from gts import *
from scan import Scan

code = """
1 + 2
1 - 2
1 * 2
1 / 2
1 // 2
1 % 2
1 ** 2
1 << 2
1 >> 2
1 | 2
1 ^ 2
1 & 2
1 @ 2
"""

parseprint(code)

print('')

scanner = Scan()
tree = scanner.to_general_form(code)
print(tree.str())

