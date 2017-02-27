import sys
import os
import json
sys.path.append(os.path.join(os.path.dirname(__file__), '..'))

from gts import *
from scan import Scan
from gast import GastEncoder


code = """
v = 5
x.attribute = 5

"""

scanner = Scan()
tree = scanner.to_general_form(code)
print(json.dumps(tree, cls=GastEncoder))

