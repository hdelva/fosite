import data.constants as constants

class Deserializer:
  def __init__(self, id_to_name):
    self.id_to_name = id_to_name
    self.count = 1

  def identifier(self, tree):
    id = tree['id']
    if id in self.id_to_name:
      return self.id_to_name[id]
    else:
      name = 'var' + str(self.count)
      self.id_to_name[id] = name
      self.count += 1
      return name

  def deserialize(self, trees):
    result = []
    for tree in trees:
      result.append(self.delegate(tree))
    return  '\n'.join(result)

  def block(self, trees):
    result = []
    for tree in trees:
      result.append('  ' + str(self.delegate(tree)))
    return result

  def argument(self, tree):
    name = self.delegate(tree['variable'])

    value = None
    if tree['value'] is not None:
      value = self.delegate(tree['value'])

    if value is None:
      return str(name)
    else:
      return str(name) + '=' + str(value)

  def assign(self, tree):
    parts = []

    for target in tree['targets']:
      target = str(self.delegate(target))

      if target[0] is '(':
        target = target[1:-1]
      
      parts.append(target)

    value = str(self.delegate(tree['value']))

    parts.append(value)

    return ' = '.join(parts)

  def function(self, tree):
    raw_args = tree['positional_args']
    args = [self.delegate(a) for a in raw_args if a['type'] is 'identifier']
    vararg = [self.delegate(a) for a in raw_args if a['type'] is not 'identifier']

    raw_args = tree['keyword_args']
    kwonly = [self.delegate(arg) for arg in raw_args if arg['type'] is 'identifier']
    kwarg = [self.delegate(arg) for arg in raw_args if arg['type'] is not 'identifier']

    args.extend(vararg)
    args.extend(kwonly)
    args.extend(kwarg)

    arg_string = ', '.join(args)

    name = self.delegate(tree['name'])

    signature = 'def ' + str(name) + '(' + str(arg_string) + '):'

    body = self.block(tree['body'])

    full = [signature]
    full.extend(body)

    return '\n'.join(full) + '\n'

  def delegate(self, tree):
    type = tree['type']

    if type is constants.IDENTIFIER:
      return self.identifier(tree)
    elif type is constants.ARGUMENT:
      return self.argument(tree)
    elif type is constants.SEQUENCE:
      return self.sequence(tree)
    elif type is constants.ASSIGN:
      return self.assign(tree)
    elif type is constants.FUNCTION:
      return self.function(tree)
    elif type is constants.CALL:
      return self.call(tree)
    elif type is constants.NUMBER:
      return self.number(tree)
    elif type is constants.STRING:
      return self.string(tree)
    elif type is constants.RETFUN:
      return self.ret(tree)

  def sequence(self, trees):
    values = [self.delegate(tree) for tree in trees['content']]
    return '(' + ', '.join(values) + ')'

  def call(self, tree):
    id = tree['name']['id']
    if id < 0:
      return self.special_call(tree)

    raw_args = tree['positional_args']
    args = [self.delegate(arg) for arg in raw_args]

    raw_kwargs = tree['keyword_args']
    kwargs = [self.delegate(arg) for arg in raw_kwargs]

    args.extend(kwargs)

    name = self.delegate(tree['name'])

    return str(name) + '(' + ', '.join(args) + ')'

  def number(self, tree):
    return str(tree['value'])

  def string(self, tree):
    return tree['value']

  def ret(self, tree):
    value = str(self.delegate(tree['positional_args'][0]))

    if value[0] is '(':
      value = value[1:-1]

    return 'return ' + value

  def star(self, tree):
    value = self.delegate(tree['positional_args'][0])
    return '*' + value

  def special_call(self, tree):
    id = tree['name']['id']

    if id is constants.ADD:
      return self.binary_op(tree, '+')
    elif id is constants.SUB:
      return self.binary_op(tree, '-')
    elif id is constants.MULT:
      return self.binary_op(tree, '*')
    elif id is constants.DIV:
      return rself.binary_op(tree, '/')
    elif id is constants.FLOORDIV:
      return self.binary_op(tree, '//')
    elif id is constants.MOD:
      return self.binary_op(tree, '%')
    elif id is constants.POW:
      return self.binary_op(tree, '**')
    elif id is constants.LSHIFT:
      return self.binary_op(tree, '<<')
    elif id is constants.RSHIFT:
      return self.binary_op(tree, '>>')
    elif id is constants.BITOR:
      return self.binary_op(tree, '|')
    elif id is constants.BITXOR:
      return self.binary_op(tree, '^')
    elif id is constants.BITAND:
      return self.binary_op(tree, '&')
    elif id is constants.MATMULT:
      return self.binary_op(tree, '@')
    elif id is constants.RETFUN:
      return self.ret(tree)
    elif id is constants.STAR:
      return self.star(tree)

  def last_operator(self, string):
    result = None
    split = string.split()
    ops = '+-**//%<>|^&@'
    for i in range(0, len(split)):
      token = string[i]

      if token in ops:
        if i > 0:
          previous_token = c[i-1]
          if previous_token not in ops:
            result = token

    return result
    
  def first_operator(self, string):
    result = None
    split = string.split()
    ops = '+-**//%<>|^&@'
    for i in range(0, len(split))[::-1]:
      token = string[i]

      if token in ops:
        if i > 0:
          previous_token = c[i-1]
          if previous_token not in ops:
            result = token

    return result

  def first_precedes_second(self, first, second):
    mapping = {}
    mapping['**'] = 0
    mapping['*'] = 1
    mapping['/'] = 1
    mapping['%'] = 1
    mapping['//'] = 1
    mapping['@'] = 1
    mapping['+'] = 2
    mapping['-'] = 2
    mapping['>>'] = 3
    mapping['<<'] = 3
    mapping['&'] = 4
    mapping['^'] = 5
    mapping['|'] = 5

    if mapping[first] < mapping[second]:
      return True
    else:
      return False

  def binary_op(self, tree, op):
    args = [self.delegate(arg) for arg in tree['positional_args']]

    left = str(args[0])
    right = str(args[1])

    last_left = self.last_operator(left)
    first_right = self.first_operator(right)

    if last_left is not None and first_precedes_second(op, last_left):
      left_partial = '(' + left + ')'
    else:
      left_partial = left

    if first_right is not None and first_precedes_second(op, first_right):
      right_partial = '(' + right + ')'
    else:
      right_partial = right

    return left_partial + ' ' + str(op) + ' ' + right_partial