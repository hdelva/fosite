import copy
import gast 
import json
import constants

from ast import *

class Scan:
  def __init__(self):
    self.count = 0

  def to_general_form(self, code):
    t = parse(code)
    general_form = self.block(t)
    #print(json.dumps(general_form, sort_keys=True,
    #                 indent=2, separators=(',', ': ')))
    return general_form

  def block(self, code):
    body = []

    if type(code) is list:
      for t in code:
        if type(t) is Pass:
          continue

        self.count += 1
        content = self.statement(t)
        body.append(content)
    else:
      for _, b in iter_fields(code):
        for t in b:
          content = self.statement(t)
          body.append(content)

    result = gast.Block(body)

    return result

  def imports(self, code):
    names = []

    for mod in code.names:
      original = gast.String(mod.name)

      if mod.asname is None:
        alias = original
      else:
        alias = gast.String(mod.asname)

      names.append(gast.Pair(original, alias))

    return gast.Import(constants.MODULE_NAME, names)


  def import_from(self, code):
    names = []

    for mod in code.names:
      original = gast.String(mod.name)

      if mod.asname is None:
        alias = original
      else:
        alias = gast.String(mod.asname)

      names.append(gast.Pair(original, alias))

    module = gast.String(code.module)

    return gast.Import(module, names)

  def statement(self, code):
    if type(code) is Expr:
      return self.expression(code.value)
    elif type(code) is FunctionDef:
      return self.function(code)
    elif type(code) is AugAssign or type(code) is Assign:
      return self.assign(code)
    elif type(code) is Return:
      return self.ret(code)
    elif type(code) is Yield:
      return self.ret_yield(code)
    elif type(code) is Raise:
      return self.ret_raise(code)
    elif type(code) is If:
      return self.conditional(code)
    elif type(code) is For:
      return self.for_loop(code)
    elif type(code) is While:
      return self.while_loop(code)
    elif type(code) is Try:
      return self.try_except(code)
    elif type(code) is With:
      return self.with_block(code)
    elif type(code) is Import:
      return self.imports(code)
    elif type(code) is ImportFrom:
      return self.import_from(code)
    elif type(code) is Lambda:
      return self.anonymous_function(code)

    raise Exception('Unsupported node:', code)

  def expression(self, code):
    if type(code) is Call:
      return self.call(code)
    elif type(code) is IfExp:
      return self.conditional(code)
    elif type(code) is Attribute:
      return self.attribute(code)
    elif type(code) is ListComp:
      return self.list_comprehension(code)
    elif type(code) is SetComp:
      return self.set_comprehension(code)
    elif type(code) is GeneratorExp:
      return self.generator_expression(code)
    elif type(code) is comprehension:
      return self.comp(code)
    elif type(code) is BinOp:
      return self.binary_operator(code)
    elif type(code) is UnaryOp:
      return self.unary_operator(code)
    elif type(code) is BoolOp:
      return self.bool_operator(code)
    elif type(code) is Compare:
      return self.compare(code)
    elif type(code) is Subscript:
      return self.subscript(code)
    elif type(code) is Name or type(code) is Starred:
      return self.variable(code)
    else:
      return self.literal(code)

  def ret(self, code):
    value = self.expression(code.value)
    return gast.Return(value, code.lineno, code.col_offset)

  def ret_yield(self, code):
    value = self.expression(code.value)
    return gast.Yield(value, code.lineno, code.col_offset)

  def ret_raise(self, code):
    value = self.expression(code.exc)
    return gast.Raise(value, code.lineno, code.col_offset)

  def _assert(self, code):
    test = self.expression(code.test)
    msg = self.expression(code.msg)
    return gast.Assert(test, msg, code.lineno, code.col_offset)

  def bool_operator(self, code):
    values = [self.expression(value) for value in code.values]
    operator = code.op
    
    acc = values[0]

    for i in range(1, len(values)):
      acc = self._bool_operator(acc, operator, values[i], code.lineno, code.col_offset)

    return acc

  def compare(self, code):
    parts = []
    left = self.expression(code.left)

    for i in range(0, len(code.comparators)):
      right = self.expression(code.comparators[i])
      part = self._bool_operator(left, code.ops[i], right, code.lineno, code.col_offset)
      parts.append(part)
      left = right

    acc = parts[0]

    for i in range(1, len(parts)):
      acc = gast.BoolOp(acc, 'and', parts[i], code.lineno, code.col_offset)

    return acc

  def _bool_operator(self, left, op, right, line, col):
    if type(op) is And:
      return gast.BoolOp(left, 'and', right, line, col, reverse='and')
    elif type(op) is Or:
      return gast.BoolOp(left, 'or', right, line, col, reverse='or')
    elif type(op) is Eq:
      return gast.BoolOp(left, '==', right, line, col, negate='!=', reverse='==')
    elif type(op) is NotEq:
      return gast.BoolOp(left, '!=', right, line, col, negate='==', reverse='!=')
    elif type(op) is Lt:
      return gast.BoolOp(left, '<', right, line, col, negate='>=', reverse='>')
    elif type(op) is LtE:
      return gast.BoolOp(left, '<=', right, line, col, negate='>', reverse='>=')
    elif type(op) is Gt:
      return gast.BoolOp(left, '>', right, line, col, negate='<=', reverse='<')
    elif type(op) is GtE:
      return gast.BoolOp(left, '>=', right, line, col, negate='<', reverse='<=')
    elif type(op) is Is:
      return gast.BoolOp(left, 'is', right, line, col, negate='is not')
    elif type(op) is IsNot:
      return gast.BoolOp(left, 'is not', right, line, col, negate='is')
    elif type(op) is In:
      return gast.BoolOp(left, 'in', right, line, col, negate='not in')
    elif type(op) is NotIn:
      return gast.BoolOp(left, 'not in', right, line, col, negate='in')

  def unary_operator(self, code):
    operand = self.expression(code.operand)
    operation = code.op
    return self._unary_operator(operand, operation, code.lineno, code.col_offset)

  def _unary_operator(self, operand, operation, line, col):
    if type(operation) is UAdd:
      return gast.UnOp('+', operand, line, col)
    elif type(operation) is USub:
      return gast.UnOp('-', operand, line, col)
    elif type(operation) is Not:
      return gast.Negate(operand, line, col)
    elif type(operation) is Invert:
      return gast.UnOp('~', operand, line, col)
    
    raise Exception('Unsupported node')

  def binary_operator(self, code):
    left = self.expression(code.left)
    right = self.expression(code.right)
    op = code.op
    return self._binary_operator(left, op, right, code.lineno, code.col_offset)

  def _binary_operator(self, left, op, right, line, col):
    if type(op) is Add:
      return gast.BinOp(left, '+', right, line, col, associative=True) 
    elif type(op) is Sub:
      return gast.BinOp(left, '-', right, line, col) 
    elif type(op) is Mult:
      return gast.BinOp(left, '*', right, line, col, associative=True)  
    elif type(op) is Div:
      return gast.BinOp(left, '/', right, line, col) 
    elif type(op) is FloorDiv:
      return gast.BinOp(left, '//', right, line, col) 
    elif type(op) is Mod:
      return gast.BinOp(left, '%', right, line, col) 
    elif type(op) is Pow:
      return gast.BinOp(left, '**', right, line, col)    
    elif type(op) is LShift:
      return gast.BinOp(left, '<<', right, line, col) 
    elif type(op) is RShift:
      return gast.BinOp(left, '>>', right, line, col) 
    elif type(op) is BitOr:
      return gast.BinOp(left, '|', right, line, col, associative=True)
    elif type(op) is BitXor:
      return gast.BinOp(left, '^', right, line, col, associative=True)
    elif type(op) is BitAnd:
      return gast.BinOp(left, '&', right, line, col, associative=True)
    elif type(op) is MatMult:
      return gast.BinOp(left, '@', right, line, col) 

    raise Exception('Unsupported node:')

  def assign(self, code):
    # change to normal assignment
    if type(code) is AugAssign:
      # extract the value from the augmented assign
      value = self.expression(code.value)

      # copy the target for referencing
      variable = copy.deepcopy(code.target)
      # change the context to load
      variable.ctx = Load()

      # extract the variable/target from the augmented assign
      variable = self.expression(variable)

      # compose the expression using the variable and the value  
      right = self._binary_operator(variable, code.op, value, code.lineno, code.col_offset)

      # augmented assigns have only one target
      # but GAST assigns support multiple targets
      left = [self.expression(code.target)]
    else:
      right = self.expression(code.value)

      # convert every target, store into a tuple 
      # call self.expression, not scope.assign 
      # kind of a hack to support starred variables
      left = [self.expression(target) for target in code.targets]

    return gast.Assign(left, right, code.lineno, code.col_offset)
    
  def variable(self, code):
    if type(code) is Name:
      result = gast.Identifier(code.id, code.lineno, code.col_offset)
    elif type(code) is Starred:
      intermediate = self.variable(code.value)
      result = self.starred(intermediate, code.lineno, code.col_offset)
    return result

  def attribute(self, code):
    temp = code.value

    if type(temp) == Name:
      target = gast.Identifier(temp.id, code.lineno, code.col_offset)
      attribute = code.attr
    else:
      target = self.expression(temp)
      attribute = code.attr

    return gast.Attribute(target, attribute, code.lineno, code.col_offset)

  def subscript(self, code):
    target = self.expression(code.value)

    op = code.slice

    if type(op) is Index:
      index = self.expression(op.value)
      return gast.Index(target, index, code.lineno, code.col_offset)
    elif type(op) is Slice:
      lower = self.expression(op.lower)
      upper = self.expression(op.upper)
      if op.step is None:
        step = gast.Number(1, code.lineno, code.col_offset)
      else:
        step = self.expression(op.step)
      return gast.Slice(target, lower, upper, step, code.lineno, code.col_offset)
    else:
      dims = []
      for dim in op.dims:
        if type(dim) is Index:
          index = self.expression(op.value)
          dims.append(index)
        else:
          lower = self.expression(op.lower)
          upper = self.expression(op.upper)
          if op.step is None:
            step = gast.Number(1, code.lineno, code.col_offset)
          else:
            step = self.expression(op.step)
          temp = gast.Sequence(lower, upper, step, code.lineno, code.col_offset)
          dims.append(temp)
      return gast.ExtSlice(target, dims, code.lineno, code.col_offset)

  def starred(self, code, line=None, col=None):
    return gast.Starred(code, line, col)

  def literal(self, code):
    if type(code) is Num:
      n = code.n 
      if type(n) is float:
        return gast.Float(n, code.lineno, code.col_offset)
      else:
        return gast.Int(code.n, code.lineno, code.col_offset)
    elif type(code) is Str:
      return gast.String(code.s, code.lineno, code.col_offset)
    elif type(code) is Bytes:
      return gast.Byte(code.s, code.lineno, code.col_offset)
    elif type(code) is List:
      values = [self.expression(element) for element in code.elts]
      return gast.List(values, code.lineno, code.col_offset) 
    elif type(code) is Tuple:
      values = [self.expression(element) for element in code.elts]
      return gast.Sequence(values, code.lineno, code.col_offset)
    elif type(code) is Set:
      values = [self.expression(element) for element in code.elts]
      return gast.Set(values, code.lineno, code.col_offset)
    elif type(code) is Dict:
      # dictionaries are stored as a set of pairs
      values = [gast.Pair(self.expression(key), self.expression(value), code.lineno, code.col_offset) \
                for key, value in zip(code.keys, code.values)]
      return gast.Dictionary(values, code.lineno, code.col_offset)
    elif type(code) is NameConstant:
      if code.value is True:
        return gast.Boolean(True, code.lineno, code.col_offset)
      elif code.value is False:
        return gast.Boolean(False, code.lineno, code.col_offset)
      else:
        return gast.Nil(code.lineno, code.col_offset)
    elif code is None:
      return gast.Nil(None, None)
    
    raise Exception('Unsupported node:', code)

  def call(self, code):
    name = self.expression(code.func)
    args = [self.expression(arg) for arg in code.args]
    kwargs = []

    for arg in code.keywords:
      keyword = arg.arg
      value = self.expression(arg.value)
      kwargs.append(self.argument(keyword, value))

    return gast.Call(name, args, code.lineno, code.col_offset, kwargs)    

  def argument(self, name, value):
    return gast.Argument(name, value)

  def comp(self, code):
    target = self.expression(code.target)
    source = self.expression(code.iter)
    conditions = [self.expression(condition) for condition in code.ifs]

    generator = gast.Generator(source, target)

    if len(conditions) > 0:
      acc = conditions[0]

      for i in range(1, len(conditions)):
        acc = gast.BoolOp(acc, 'and', conditions[i], code.lineno, code.col_offset)

      return gast.Filter(generator, acc)

    return generator

  def list_comprehension(self, code):
    generators = [self.expression(generator) for generator in code.generators]

    acc = generators[0]

    for i in range(1, len(generators)):
      acc = gast.AndThen(acc, generators[i], code.lineno, code.col_offset)

    fun = self.expression(code.elt)

    mapped = gast.Map(acc, fun, code.lineno, code.col_offset)
    return gast.List(mapped, code.lineno, code.col_offset)

  def set_comprehension(self, code):
    generators = [self.expression(generator) for generator in code.generators]

    acc = generators[0]

    for i in range(1, len(generators)):
      acc = gast.AndThen(acc, generators[i], code.lineno, code.col_offset)

    fun = self.expression(code.elt)

    mapped = gast.Map(acc, fun, code.lineno, code.col_offset)
    return gast.Set(mapped, code.lineno, code.col_offset)

  def generator_expression(self, code):
    generators = [self.expression(generator) for generator in code.generators]

    acc = generators[0]

    for i in range(1, len(generators)):
      acc = gast.AndThen(acc, generators[i], code.lineno, code.col_offset)

    fun = self.expression(code.elt)

    mapped = gast.Map(acc, fun, code.lineno, code.col_offset)
    return mapped

  def conditional(self, code):
    test = self.expression(code.test)

    # todo add node id
    body = self.block(code.body)
    orElse = self.block(code.orelse)

    return gast.If(test, body, orElse, code.lineno, code.col_offset)

  def for_loop(self, code):
    source = self.expression(code.iter)
    target = self.expression(code.target)

    generator = gast.Generator(source, target, code.lineno, code.col_offset)

    # todo add node id
    body = self.block(code.body)

    orElse = self.block(code.orelse)

    return gast.ForEach(generator, body, orElse, code.lineno, code.col_offset)

  def while_loop(self, code):
    test = self.expression(code.test)

    # todo add node id
    body = self.block(code.body)

    orElse = self.block(code.orelse)

    return gast.While(test, body, orElse, code.lineno, code.col_offset)

  def try_except(self, code):
    # todo add node id
    body = self.block(code.body)

    handlers = []

    for handler in code.handlers:
      handleBody = self.handler(handler)
      handlers.append(handleBody)

    orElse = self.block(code.orelse)
    
    final = self.block(code.finalbody)

    return gast.Try(handlers, body, orElse, final, code.lineno, code.col_offset)

  # todo, messy
  def with_block(self, code):
    before_items = []
    after_items = []

    for item in code.items:
      context = self.expression(item.context_expr)
      
      if item.optional_vars is None:
        enter = gast.Identifier(constants.ENTER, code.lineno, code.col_offset)
        enter = gast.Attribute(context, enter, code.lineno, code.col_offset)
        enter = gast.Call(enter, [], code.lineno, code.col_offset)

        exit = gast.Identifier(constants.EXIT, code.lineno, code.col_offset)
        exit = gast.Attribute(context, exit, code.lineno, code.col_offset)
        exit = gast.Call(exit, [], code.lineno, code.col_offset)

        before_items.append(enter)
        after_items.append(exit)
      else:
        name = self.expression(item.optional_vars)

        assignment = gast.Assign(name, context, code.lineno, code.col_offset)
        before_items.append(assignment)

        enter = gast.Identifier(constants.ENTER, code.lineno, code.col_offset)
        enter = gast.Attribute(name, enter, code.lineno, code.col_offset)
        enter = gast.Call(enter, [], code.lineno, code.col_offset)

        before_items.append(enter)

        exit = gast.Identifier(constants.EXIT, code.lineno, code.col_offset)
        exit = gast.Attribute(name, exit, code.lineno, code.col_offset)
        exit = gast.Call(exit, [], code.lineno, code.col_offset)

        after_items.append(exit)

    body = self.block(code.body)
    return gast.With(before_items, body, after_items, code.lineno, code.col_offset)
    

  def handler(self, code):
    body = self.block(code.body)
    return gast.Case(code.type, code.name, body,code.lineno, code.col_offset)

  def anonymous_function(self, code):
    args = []

    for arg in code.args.args:
      identifier = arg.arg

      default = None
      args.append(gast.Argument(identifier, default))

    body = self.block(code.body)
    return gast.AnonymousFunction(args, body, code.lineno, code.col_offset)

  def class_def(self, code):
    # extract the name and declare a variable of the same name
    name = code.name
    identifier = gast.Identifier(identifier.name, code.lineno, code.col_offset)

    bases = []

    for base in code.bases:
      bases.append(self.expression(base))

    body = self.block(code.body)

    return gast.ClassDef(name, bases, body, code.lineno, code.col_offset)
    
  def function(self, function):
    # Helper function for positional arguments in the signature
    def positional_args(args, defaults):
        result = []

        # function without positional arguments 
        if args is None:
            return result

        count = 0
        for i, arg in enumerate(args):
            # name of the identifier
            # _not_ an actual identifier, conform to the argument syntax used when calling
            identifier = arg.arg

            default = None

            if i >= len(args) - len(defaults):
                offset = len(args) - i - 1
                default = defaults[offset]

                default = self.literal(default)
        
            argument = self.argument(identifier, default)
            result.append(argument)
            count += 1

        return result

    # Helper function for keyword arguments
    def keyword_args(args, defaults):
        result = []

        # function without positional arguments 
        if args is None:
            return result

        count = 0
        for arg, default in zip(args, defaults):
            # name of the identifier
            # _not_ an actual identifier, conform to the argument syntax used when calling
            identifier = arg.arg
            
            if default is None:
                default = None
            else:
                default = self.literal(default)
        
            argument = self.argument(identifier, default)
            result.append(argument)
            count += 1

        return result

    # Helper function for vararg arguments
    def vararg(arg):
        # name of the identifier
        # _not_ an actual identifier, conform to the argument syntax used when calling
        identifier = arg.arg

        # varargs are starred
        return self.starred(identifier)

    # Helper function for kwarg arguments
    def kwarg(arg):
        # name of the identifier
        # _not_ an actual identifier, conform to the argument syntax used when calling
        identifier = arg.arg

        # kwargs are double starred
        return self.starred(self.starred(identifier))

    # extract the name and declare a variable of the same name
    name = function.name
    identifier = gast.Identifier(name, function.lineno, function.col_offset)

    # contains all kinds of arguments
    args = function.args

    # extract the four kinds of arguments
    _positional_args = args.args
    _positional_defaults = args.defaults
    _kw_only_args = args.kwonlyargs
    _kw_defaults = args.kw_defaults
    _vararg = args.vararg 
    _kwarg = args.kwarg

    # process the kinds of arguments
    positional_args = positional_args(_positional_args, _positional_defaults)

    if _vararg is not None:
      positional_args.append(vararg(_vararg))

    keyword_args = keyword_args(_kw_only_args, _kw_defaults)
    if _kwarg is not None:
      keyword_args.append(kwarg(_kwarg))

    body = self.block(function.body)

    return gast.Function(identifier, positional_args, keyword_args, body, function.lineno, function.col_offset)


