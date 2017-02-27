import sys
import os
sys.path.append(os.path.join(os.path.dirname(__file__), '..'))

from gts import *
from scan import Scan

code = """
if x < y:
  x += 1
  v = x
else:
  v = y

for z in range(0, v):
  v += 1
else:
  v -= 1

while cond:
  print(cond)

with open(file) as f:
  f.read()

with f:
  f.read()

try:
  f.read()
except:
  print('shiet')
else:
  print('cool')
finally:
  print('is done')
"""

parseprint(code)

print('')

scanner = Scan()
tree = scanner.to_general_form(code)
print(tree.str())

