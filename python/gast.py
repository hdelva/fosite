import constants as constants

from json import JSONEncoder
from abc import ABCMeta, abstractmethod
import constants

class GastEncoder(JSONEncoder):
    def default(self, o):
        temp =  {k:v for k, v in o.items()}
        temp['kind'] = o.kind()
        return temp

class GastNode(object, metaclass=ABCMeta):
  @abstractmethod
  def kind(self):
    pass 

  def items(self):
    return self.__dict__.items()

  def str(self):
    result = 'Kind: ' + self.kind() + '\n'

    for attr, value in self.items():
      if type(value) is list:
        subresult = []
        for count, nested_value in enumerate(value):
          sub = nested_value.str().strip()
        
          indented = []
          for line in sub.splitlines():
            if len(line) < 1:
              continue
            indented.append('  ' + line)

          indented = '\n'.join(indented)

          subresult.append('-' + indented[1:])

        subresult = '\n'.join(subresult)
        result += '{}: \n{}\n'.format(attr, subresult)

      elif isinstance(value, GastNode):
        sub = value.str().strip()
      
        indented = ''
        split = sub.splitlines()
        for line in split:
          if len(line) < 1:
            continue
          if len(split) == 1:
            indented += line + '\n'
          else:
            indented += '  ' + line + '\n'

        if len(split) > 1:
          result += '{}: \n{}\n'.format(attr, indented)
        else:
          result += '{}: {}\n'.format(attr, indented)
      else:
        result += '{}: {}\n'.format(attr, str(value))

    return result.strip()

global count
count = 0

class Named(GastNode, object):
  def __init__(self, name: 'str'):
    self.name = name
    global count
    self.id = count
    count += 1

class Control(GastNode):
  def __init__(self, before: 'Block', test: 'BoolOp', body: 'Block', orElse: 'Block', after: 'Block'):
    self.before = before
    self.test = test
    self.body = body
    self.orElse = orElse
    self.after = after
    global count
    self.id = count
    count += 1

class If(Control):
  def __init__(self, test: 'BoolOp', body:'Block', orElse:'Block', line, col):
    super().__init__(None, test, body, orElse, None)
    self.line = line
    self.col = col

  def kind(self):
    return constants.IF

class ForEach(Control):
  def __init__(self, before: 'Generator', body: 'Block', orElse: 'Block', line, col):
    super().__init__(before, None, body, orElse, None)
    self.line = line
    self.col = col

  def kind(self):
    return constants.FOREACH

class While(Control):
  def __init__(self, test: 'BoolOp', body: 'Block', orElse: 'Block', line, col):
    super().__init__(None, test, body, orElse, None)
    self.line = line
    self.col = col

  def kind(self):
    return constants.WHILE

class Try(Control):
  def __init__(self, test: '[Case]', body: 'Block', orElse: 'Block', after: 'Block', line, col):
    super().__init__(None, test, body, orElse, after)
    self.line = line
    self.col = col

  def kind(self):
    return constants.TRY
    
class Case(GastNode):
  def __init__(self, type: '?', name: 'Identifier', body: 'Block', line, col):
    self.line = line
    self.col = col
    self.type = type
    self.name = name
    self.body = body
    global count
    self.id = count
    count += 1

  def kind(self):
    return constants.CASE

class With(Control):
  def __init__(self, before: '[Expression]', body: 'Block', after: '[Expression]', line, col):
    super().__init__(before, None, body, None, after)
    self.line = line
    self.col = col

  def kind(self):
    return constants.WITH

class Block(GastNode):
  def __init__(self, content: '[GastNode]'):
    self.content = content
    global count
    self.id = count
    count += 1

  def kind(self):
    return constants.BLOCK

class Index(GastNode):
  def __init__(self, target: 'Expression', index: 'Number', line, col):
    self.line = line
    self.col = col
    self.target = target
    self.index = index
    global count
    self.id = count
    count += 1

  def kind(self):
    return constants.INDEX

class Attribute(GastNode):
  def __init__(self, target: 'Expression', attribute: 'String', line, col):
    self.of = target
    self.attribute = attribute
    global count
    self.id = count
    count += 1
    self.line = line
    self.col = col

  def kind(self):
    return constants.ATTRIBUTE

class Identifier(Named):
  def __init__(self, name: 'str', line, col):
    super().__init__(name)
    global count
    self.id = count
    count += 1
    self.line = line
    self.col = col

  def kind(self):
    return constants.IDENTIFIER

  def str(self):
    return '{}'.format(self.name)

class Int(GastNode):
  def __init__(self, value, line, col):
    self.line = line
    self.col = col
    self.value = value
    global count
    self.id = count
    count += 1

  def kind(self):
    return constants.INT
 
  def str(self):
    return str(self.value)

class Float(GastNode):
  def __init__(self, value, line, col):
    self.line = line
    self.col = col
    self.value = value
    global count
    self.id = count
    count += 1

  def kind(self):
    return constants.FLOAT
 
  def str(self):
    return str(self.value)

class String(GastNode):
  def __init__(self, value, line, col):
    self.line = line
    self.col = col
    self.value = value
    global count
    self.id = count
    count += 1

  def kind(self):
    return constants.STRING

  def str(self):
    return "'" + self.value + "'"

class Byte(GastNode):
  def __init__(self, value, line, col):
    self.value = value
    global count
    self.id = count
    count += 1
    self.line = line
    self.col = col

  def kind(self):
    return constants.BYTE

# fixed length
class Sequence(GastNode):
  def __init__(self, content: 'iterable', line, col):
    self.content = content
    global count
    self.id = count
    count += 1
    self.line = line
    self.col = col

  def kind(self):
    return constants.SEQUENCE

# variable length
class List(GastNode):
  def __init__(self, content: 'iterable', line, col):
    self.line = line
    self.col = col
    self.content = content
    global count
    self.id = count
    count += 1

  def kind(self):
    return constants.LIST

# special kind of sequence
class Pair(GastNode):
  def __init__(self, first, second, line, col):
    self.line = line
    self.col = col
    self.first = first
    self.second = second
    global count
    self.id = count
    count += 1

  def kind(self):
    return constants.PAIR

class Dictionary(GastNode):
  def __init__(self, content: 'list[Pair]', line, col):
    self.content = content
    global count
    self.id = count
    count += 1
    self.line = line
    self.col = col
 
  def kind(self):
    return constants.DICTIONARY

class Set(GastNode):
  def __init__(self, content: 'iterable', line, col):
    self.line = line
    self.col = col
    self.content = content
    global count
    self.id = count
    count += 1

  def kind(self):
    return constants.SET

class Boolean(GastNode):
  def __init__(self, value:'boolean', line, col):
    self.line = line
    self.col = col
    self.value = value
    global count
    self.id = count
    count += 1

  def kind(self):
    return constants.BOOLEAN

class Nil(GastNode):
  def __init__(self, line, col):
    self.line = line
    self.col = col
    global count
    self.id = count
    count += 1

  def kind(self):
    return constants.NIL

class Argument(Named):
  def __init__(self, name: 'Identifier', value: 'literal'):
    super().__init__(name)
    self.value = value

  def kind(self):
    return constants.ARGUMENT

class Function(Named):
  def __init__(self, name: 'Identifier', pos_args: 'list[Argument]', kw_args, body: 'Block', line, col):
    super().__init__(name)
    self.line = line
    self.col = col
    self.positional_args = pos_args
    self.keyword_args = kw_args
    self.body = body

  def kind(self):
    return constants.FUNCTION

class ClassDef(Named):
  def __init__(self, name: 'Identifier', bases: '[Identifier]', body: 'Block', line, col):
    super().__init__(name)
    self.bases = bases
    self.body = body
    self.line = line
    self.col = col

  def kind(self):
    return constants.CLASS

class Call(Named):
  def __init__(self, name: 'Name', pos_args: 'list[Expression]', line, col, kw_args: 'list[Argument]'=[]):
    super().__init__(name)
    self.positional_args = pos_args
    self.keyword_args = kw_args
    self.line = line
    self.col = col

  def kind(self):
    return constants.CALL

class Assign(GastNode):
  def __init__(self, targets: 'list', value: 'Expression', line, col):
    self.targets = targets
    self.value = value
    global count
    self.id = count
    count += 1
    self.line = line
    self.col = col

  def kind(self):
    return constants.ASSIGN

class Negate(GastNode):
  def __init__(self, value: 'Expression', line, col):
    self.line = line
    self.col = col
    self.value = value
    global count
    self.id = count
    count += 1
  
  def kind(self):
    return constants.NEGATE

class BinOp(GastNode):
  def __init__(self, left: 'Expression', op: 'str', right, line, col, associative=False):
    self.left = left
    self.right = right
    self.op = op
    self.associative = associative
    global count
    self.id = count
    count += 1
    self.line = line
    self.col = col
    
  def items(self):
    return [('left', self.left), ('op', self.op), ('right', self.right), ('line', self.line), ('col', self.col)]

  def kind(self):
    return constants.BINOP

class BoolOp(GastNode):
  def __init__(self, left: 'Expression', op: 'str', right, line, col, reverse: 'str'=None, negate=None):
    self.left = left
    self.op = op
    self.right = right
    self.reverse = reverse
    self.negate = negate
    global count
    self.id = count
    count += 1
    self.line = line
    self.col = col

  def kind(self):
    return constants.BOOLOP

  def items(self):
    return [('left', self.left), ('op', self.op), ('right', self.right), ('line', self.line), ('col', self.col)]

class UnOp(GastNode):
  def __init__(self, operation: 'str', value: 'Expression', line, col):
    self.operation = operation
    self.value = value
    global count
    self.id = count
    count += 1
    self.line = line
    self.col = col
   
  def kind(self):
    return constants.UNOP

class Return(GastNode):
  def __init__(self, value: 'Expression', line, col):
    self.value = value
    global count
    self.id = count
    count += 1
    self.line = line
    self.col = col

  def kind(self):
    return constants.RETURN

class Yield(GastNode):
  def __init__(self, value: 'Expression', line, col):
    self.value = value
    global count
    self.id = count
    count += 1
    self.line = line
    self.col = col

  def kind(self):
    return constants.RETURN

class Raise(GastNode):
  def __init__(self, value: 'Expression'):
    self.value = value
    global count
    self.id = count
    count += 1

  def kind(self):
    return constants.RAISE

class Assert(GastNode):
  def __init__(self, test: 'Expression', message: 'Expression', line, col):
    self.test = test
    self.message = message
    global count
    self.id = count
    count += 1
    self.line = line
    self.col = col

  def kind(self):
    return constants.ASSERT

class Import(GastNode):
  def __init__(self, module: 'str', aliases: '[Pair]', line, col):
    self.module = module
    self.aliases = aliases
    global count
    self.id = count
    count += 1
    self.line = line
    self.col = col

  def kind(self):
    return constants.Import

class AnonymousFunction(GastNode):
  def __init__(self, args: '[Argument]', body: 'Block', line, col):
    self.args = args
    self.body = body
    global count
    self.id = count
    count += 1
    self.line = line
    self.col = col

  def kind(self):
    return constants.ANONYMOUS_FUNCTION

def Starred(value: 'Expression', line, col):
  return UnOp('*', value, line, col)

def Slice(target: 'Expression', lower, upper, step, line, col):
  return Call(constants.SLICE, [lower, upper, step], line, col)

def ExtSlice(target: 'Expression', dims, line, col):
  return Call(constants.EXTSLICE, dims, line, col)

class Stream(GastNode):
  # wat do here
  pass

class Generator(Stream):
  def __init__(self, source: 'Iterable', target: 'Named', line=None, col=None):
    self.source = source
    self.target = target
    global count
    self.id = count
    count += 1
    self.line = line
    self.col = col


  def kind(self):
    return constants.GENERATOR

class Filter(Stream):
  def __init__(self, source: 'Iterable', condition: 'BoolOp', line=None, col=None):
    self.source = source
    self.condition = condition
    global count
    self.id = count
    count += 1
    self.line = line
    self.col = col

  def kind(self):
    return constants.FILTER

class Map(Stream):
  def __init__(self, source: 'Iterable', op: 'Expression', line, col):
    self.source = source
    self.op = op
    global count
    self.id = count
    count += 1
    self.line = line
    self.col = col

  def kind(self):
    return constants.MAP

class AndThen(Stream):
  def __init__(self, first: 'Stream', second: 'Stream', line, col):
    self.first = first
    self.second = second
    global count
    self.id = count
    count += 1
    self.line = line
    self.col = col
    
  def kind(self):
    return constants.ANDTHEN 
