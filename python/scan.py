import copy
import gast 
import json
import constants

from ast import *

class ScopedVariable:
  def __init__(self, namespace, name):
    self.namespace = namespace
    self.name = name
    self.generation = 0
    self.attributes = {}

  def merge(self, other):
    self.generation = max(self.generation, other.generation)

    for name in other.attributes:
      attribute = other.attributes[name]
      if name in self.attributes:
        self.attributes[name].merge(attribute)
      else:
        self.attributes[name] = attribute

  def assign(self):
    self.generation += 1
    return self

  def assign_attribute(self, attribute):
    target = self.reference_attribute(attribute)
    target.assign()
    return target

  def reference_attribute(self, attribute):
    if attribute in self.attributes:
      target = self.attributes[attribute]
      return target
    else:
      target_ns = gast.Namespace(self.name, self.namespace)
      target = ScopedVariable(target_ns, attribute)
      self.attributes[attribute] = target
      return target

class Scope:
  def __init__(self, name, parent=None):
    self.name = name
    self.namespace = gast.Namespace(name, parent)
    self.extern = gast.Namespace(constants.EXTERN, None)
    self.parent = parent
    self.variables = {}

  def set_name(self, name):
    self.name = name
    self.namespace = gast.Namespace(name, self.parent)

  def set_parent(self, parent):
    self.parent = parent

  def merge(self, other):
    for name in other.variables:
      variable = other.variables[name]
      if name in self.variables:
        self.variables[name].merge(variable)
      else:
        self.variables[name] = variable

  def get_variables(self):
    return self.variables

  def get_parent(self):
    return self.parent

  def get_name(self):
    return self.name

  def reference(self, name):
    if name in self.variables:
      target = self.variables[name]
      return target
    else:
      target = ScopedVariable(self.extern, name)
      self.variables[name] = target
      return target

  def assign(self, name):
    target = self.reference(name)
    target.namespace = self.namespace
    target.generation += 1
    return target

class Scan:
  def __init__(self):
    self.count = 0

  def to_general_form(self, code):
    t = parse(code)
    scope = Scope(constants.ROOT)
    general_form = self.block(t, scope)
    #print(json.dumps(general_form, sort_keys=True,
    #                 indent=2, separators=(',', ': ')))
    return general_form

  def block(self, code, scope):
    body = []
    namespace = scope.namespace

    if type(code) is list:
      for t in code:
        if type(t) is Pass:
          continue

        self.count += 1
        content = self.statement(t, scope)
        body.append(content)
    else:
      for _, b in iter_fields(code):
        for t in b:
          content = self.statement(t, scope)
          body.append(content)

    result = gast.Block(namespace, body)

    return result

  def imports(self, code, scope):
    names = []

    for mod in code.names:
      original = gast.String(mod.name)

      if mod.asname is None:
        alias = original
      else:
        alias = gast.String(mod.asname)

      names.append(gast.Pair(original, alias))

    return gast.Import(constants.MODULE_NAME, names)


  def import_from(self, code, scope):
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

  def statement(self, code, scope):
    if type(code) is Expr:
      return self.expression(code.value, scope)
    elif type(code) is FunctionDef:
      return self.function(code, scope)
    elif type(code) is AugAssign or type(code) is Assign:
      return self.assign(code, scope)
    elif type(code) is Return:
      return self.ret(code, scope)
    elif type(code) is Yield:
      return self.ret_yield(code, scope)
    elif type(code) is Raise:
      return self.ret_raise(code, scope)
    elif type(code) is If:
      return self.conditional(code, scope)
    elif type(code) is For:
      return self.for_loop(code, scope)
    elif type(code) is While:
      return self.while_loop(code, scope)
    elif type(code) is Try:
      return self.try_except(code, scope)
    elif type(code) is With:
      return self.with_block(code, scope)
    elif type(code) is Import:
      return self.imports(code, scope)
    elif type(code) is ImportFrom:
      return self.import_from(code, scope)
    elif type(code) is Lambda:
      return self.anonymous_function(code, scope)

    raise Exception('Unsupported node:', code)

  def expression(self, code, scope):
    if type(code) is Call:
      return self.call(code, scope)
    elif type(code) is IfExp:
      return self.conditional(code, scope)
    elif type(code) is Attribute:
      return self.attribute(code, scope)
    elif type(code) is ListComp:
      return self.list_comprehension(code, scope)
    elif type(code) is SetComp:
      return self.set_comprehension(code, scope)
    elif type(code) is GeneratorExp:
      return self.generator_expression(code, scope)
    elif type(code) is comprehension:
      return self.comp(code, scope) 
    elif type(code) is BinOp:
      return self.binary_operator(code, scope)
    elif type(code) is UnaryOp:
      return self.unary_operator(code, scope)
    elif type(code) is BoolOp:
      return self.bool_operator(code, scope)
    elif type(code) is Compare:
      return self.compare(code, scope)
    elif type(code) is Subscript:
      return self.subscript(code, scope)
    elif type(code) is Name or type(code) is Starred:
      return self.variable(code, scope)
    else:
      return self.literal(code, scope)

  def ret(self, code, scope):
    value = self.expression(code.value, scope)
    return gast.Return(value)

  def ret_yield(self, code, scope):
    value = self.expression(code.value, scope)
    return gast.Yield(value)

  def ret_raise(self, code, scope):
    value = self.expression(code.exc, scope)
    return gast.Raise(value)

  def _assert(self, code, scope):
    test = self.expression(code.test, scope)
    msg = self.expression(code.msg, scope)
    return gast.Assert(test, msg)

  def bool_operator(self, code, scope):
    values = [self.expression(value, scope) for value in code.values]
    operator = code.op
    
    acc = values[0]

    for i in range(1, len(values)):
      acc = self._bool_operator(acc, operator, values[i])

    return acc

  def compare(self, code, scope):
    parts = []
    left = self.expression(code.left, scope)

    for i in range(0, len(code.comparators)):
      right = self.expression(code.comparators[i], scope)
      part = self._bool_operator(left, code.ops[i], right)
      parts.append(part)
      left = right

    acc = parts[0]

    for i in range(1, len(parts)):
      acc = gast.BoolOp(acc, 'and', parts[i])

    return acc

  def _bool_operator(self, left, op, right):
    if type(op) is And:
      return gast.BoolOp(left, 'and', right, reverse='and')
    elif type(op) is Or:
      return gast.BoolOp(left, 'or', right, reverse='or')
    elif type(op) is Eq:
      return gast.BoolOp(left, '==', right, negate='!=', reverse='==')
    elif type(op) is NotEq:
      return gast.BoolOp(left, '!=', right, negate='==', reverse='!=')
    elif type(op) is Lt:
      return gast.BoolOp(left, '<', right, negate='>=', reverse='>')
    elif type(op) is LtE:
      return gast.BoolOp(left, '<=', right, negate='>', reverse='>=')
    elif type(op) is Gt:
      return gast.BoolOp(left, '>', right, negate='<=', reverse='<')
    elif type(op) is GtE:
      return gast.BoolOp(left, '>=', right, negate='<', reverse='<=')
    elif type(op) is Is:
      return gast.BoolOp(left, 'is', right, negate='is not')
    elif type(op) is IsNot:
      return gast.BoolOp(left, 'is not', right, negate='is')
    elif type(op) is In:
      return gast.BoolOp(left, 'in', right, negate='not in')
    elif type(op) is NotIn:
      return gast.BoolOp(left, 'not in', right, negate='in')

  def unary_operator(self, code, scope):
    operand = self.expression(code.operand, scope)
    operation = code.op
    return self._unary_operator(operand, operation, scope)

  def _unary_operator(self, operand, operation, scope):
    if type(operation) is UAdd:
      return gast.UnOp('+', operand)
    elif type(operation) is USub:
      return gast.UnOp('-', operand)
    elif type(operation) is Not:
      return gast.Negate(operand)
    elif type(operation) is Invert:
      return gast.UnOp('~', operand)
    
    raise Exception('Unsupported node')

  def binary_operator(self, code, scope):
    left = self.expression(code.left, scope)
    right = self.expression(code.right, scope)
    op = code.op
    return self._binary_operator(left, op, right)

  def _binary_operator(self, left, op, right):
    if type(op) is Add:
      return gast.BinOp(left, '+', right, associative=True) 
    elif type(op) is Sub:
      return gast.BinOp(left, '-', right) 
    elif type(op) is Mult:
      return gast.BinOp(left, '*', right, associative=True)  
    elif type(op) is Div:
      return gast.BinOp(left, '/', right) 
    elif type(op) is FloorDiv:
      return gast.BinOp(left, '//', right) 
    elif type(op) is Mod:
      return gast.BinOp(left, '%', right) 
    elif type(op) is Pow:
      return gast.BinOp(left, '**', right)    
    elif type(op) is LShift:
      return gast.BinOp(left, '<<', right) 
    elif type(op) is RShift:
      return gast.BinOp(left, '>>', right) 
    elif type(op) is BitOr:
      return gast.BinOp(left, '|', right, associative=True)
    elif type(op) is BitXor:
      return gast.BinOp(left, '^', right, associative=True)
    elif type(op) is BitAnd:
      return gast.BinOp(left, '&', right, associative=True)
    elif type(op) is MatMult:
      return gast.BinOp(left, '@', right) 

    raise Exception('Unsupported node:')

  def assign(self, code, scope):
    # change to normal assignment
    if type(code) is AugAssign:
      # extract the value from the augmented assign
      value = self.expression(code.value, scope)

      # copy the target for referencing
      variable = copy.deepcopy(code.target)
      # change the context to load
      variable.ctx = Load()

      # extract the variable/target from the augmented assign
      variable = self.expression(variable, scope)

      # compose the expression using the variable and the value  
      right = self._binary_operator(variable, code.op, value)

      # augmented assigns have only one target
      # but GAST assigns support multiple targets
      left = [self.expression(code.target, scope)]
    else:
      right = self.expression(code.value, scope)

      # convert every target, store into a tuple 
      # call self.expression, not scope.assign 
      # kind of a hack to support starred variables
      left = [self.expression(target, scope) for target in code.targets]

    return gast.Assign(left, right)
    
  def variable(self, code, scope):
    if type(code) is Name:
      temp = scope.assign(code.id) if type(code.ctx) is Store else scope.reference(code.id)
      result = gast.Identifier(temp.namespace, temp.name, temp.generation)
    elif type(code) is Starred:
      intermediate = self.variable(code.value, scope, assign=assign)
      result = self.starred(intermediate, scope)
    return result

  def attribute(self, code, scope):
    temp = code.value

    if type(temp) == Name:
      target = scope.reference(temp.id)

      if type(code.ctx) is Load:
        attribute = target.reference_attribute(code.attr)
      else:
        attribute = target.assign_attribute(code.attr)

      target = gast.Identifier(target.namespace, target.name, target.generation)
      attribute = gast.Identifier(attribute.namespace, attribute.name, attribute.generation)
    else:
      target = self.expression(code.value, scope)
      temp_space = gast.Namespace(constants.TEMPORARY, None)

      attribute = gast.Identifier(temp_space, code.attr, 0)

    return gast.Attribute(target, attribute)

  def subscript(self, code, scope):
    target = self.expression(code.value, scope)

    op = code.slice

    if type(op) is Index:
      index = self.expression(op.value, scope)
      return gast.Index(target, index)
    elif type(op) is Slice:
      lower = self.expression(op.lower, scope)
      upper = self.expression(op.upper, scope)
      if op.step is None:
        step = gast.Number(1)
      else:
        step = self.expression(op.step, scope)
      return gast.Slice(target, lower, upper, step)
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
            step = gast.Number(1)
          else:
            step = self.expression(op.step)
          temp = gast.Sequence(lower, upper, step)
          dims.append(temp)
      return gast.ExtSlice(target, dims)

  def starred(self, code, scope):
    return gast.Starred(code)

  def literal(self, code, scope):
    if type(code) is Num:
      return gast.Number(code.n)
    elif type(code) is Str:
      return gast.String(code.s)
    elif type(code) is Bytes:
      return gast.Byte(code.s)
    elif type(code) is List:
      values = [self.expression(element, scope) for element in code.elts]
      return gast.List(values) 
    elif type(code) is Tuple:
      values = [self.expression(element, scope) for element in code.elts]
      return gast.Sequence(values)
    elif type(code) is Set:
      values = [self.expression(element, scope) for element in code.elts]
      return gast.Set(values)
    elif type(code) is Dict:
      # dictionaries are stored as a set of pairs
      values = [gast.Pair(self.expression(key, scope), self.expression(value, scope)) \
                for key, value in zip(code.keys, code.values)]
      return gast.Dictionary(values)
    elif type(code) is NameConstant:
      if code.value is True:
        return gast.Boolean(True)
      elif code.value is False:
        return gast.Boolean(False)
      else:
        return gast.Nil()
    elif code is None:
      return gast.Nil()
    
    raise Exception('Unsupported node:', code)

  def call(self, code, scope):
    name = self.expression(code.func, scope)
    args = [self.expression(arg, scope) for arg in code.args]
    kwargs = []

    for arg in code.keywords:
      # keyword is _not_ a variable in the current scope
      keyword = arg.arg
      value = self.expression(arg.value, scope)
      kwargs.append(self.argument(keyword, value))

    return gast.Call(name, args, kwargs)    

  def argument(self, name, value):
    return gast.Argument(name, value)

  def comp(self, code, scope):
    target = self.expression(code.target, scope)
    source = self.expression(code.iter, scope) 
    conditions = [self.expression(condition, scope) for condition in code.ifs]

    generator = gast.Generator(source, target)

    if len(conditions) > 0:
      acc = conditions[0]

      for i in range(1, len(conditions)):
        acc = gast.BoolOp(acc, 'and', conditions[i])

      return gast.Filter(generator, acc)

    return generator

  def list_comprehension(self, code, scope):
    inner_scope = copy.deepcopy(scope)
    inner_scope.set_name(constants.GENERATOR_SCOPE)
    inner_scope.set_parent(scope.name)

    generators = [self.expression(generator, inner_scope) for generator in code.generators]

    acc = generators[0]

    for i in range(1, len(generators)):
      acc = gast.AndThen(acc, generators[i])

    fun = self.expression(code.elt, inner_scope)

    mapped = gast.Map(acc, fun)
    return gast.List(mapped)

  def set_comprehension(self, code, scope):
    inner_scope = copy.deepcopy(scope)
    inner_scope.set_name(constants.GENERATOR_SCOPE)
    inner_scope.set_parent(scope.name)

    generators = [self.expression(generator, inner_scope) for generator in code.generators]

    acc = generators[0]

    for i in range(1, len(generators)):
      acc = gast.AndThen(acc, generators[i])

    fun = self.expression(code.elt, inner_scope)

    mapped = gast.Map(acc, fun)
    return gast.Set(mapped)

  def generator_expression(self, code, scope):
    inner_scope = copy.deepcopy(scope)
    inner_scope.set_name(constants.ANONYMOUS_SCOPE)
    inner_scope.set_parent(scope.name)

    generators = [self.expression(generator, inner_scope) for generator in code.generators]

    acc = generators[0]

    for i in range(1, len(generators)):
      acc = gast.AndThen(acc, generators[i])

    fun = self.expression(code.elt, inner_scope)

    mapped = gast.Map(acc, fun)
    return mapped

  def conditional(self, code, scope):
    test = self.expression(code.test, scope)

    # todo add node id
    bodyScope = copy.deepcopy(scope)
    body = self.block(code.body, bodyScope)

    elseScope = copy.deepcopy(scope)
    orElse = self.block(code.orelse, elseScope)

    scope.merge(bodyScope)
    scope.merge(elseScope)

    return gast.If(test, body, orElse)

  def for_loop(self, code, scope):
    source = self.expression(code.iter, scope)
    target = self.expression(code.target, scope)

    generator = gast.Generator(source, target)

    # todo add node id
    body = self.block(code.body, scope)

    orElse = self.block(code.orelse, scope)

    return gast.ForEach(generator, body, orElse)

  def while_loop(self, code, scope):
    test = self.expression(code.test, scope)

    # todo add node id
    body = self.block(code.body, scope)

    orElse = self.block(code.orelse, scope)

    return gast.While(test, body, orElse)

  def try_except(self, code, scope):
    # todo add node id
    body = self.block(code.body, scope)

    handleScopes = []
    handlers = []

    for handler in code.handlers:
      handleScope = copy.deepcopy(scope)
      handleBody = self.handler(handler, handleScope)
      handleScopes.append(handleScope)
      handlers.append(handleBody)

    for handleScope in handleScopes:
      scope.merge(handleScope)

    orElse = self.block(code.orelse, scope)
    
    final = self.block(code.finalbody, scope)

    return gast.Try(handlers, body, orElse, final)

  # todo, messy
  def with_block(self, code, scope):
    before_items = []
    after_items = []

    for item in code.items:
      context = self.expression(item.context_expr, scope)
      
      if item.optional_vars is None:
        enter = gast.Identifier(scope.namespace, constants.ENTER, 0)
        enter = gast.Attribute(context, enter)
        enter = gast.Call(enter, [])

        exit = gast.Identifier(scope.namespace, constants.EXIT, 0)
        exit = gast.Attribute(context, exit)
        exit = gast.Call(exit, [])

        before_items.append(enter)
        after_items.append(exit)
      else:
        name = self.expression(item.optional_vars, scope)

        assignment = gast.Assign(name, context)
        before_items.append(assignment)

        enter = gast.Identifier(scope.namespace, constants.ENTER, 0)
        enter = gast.Attribute(name, enter)
        enter = gast.Call(enter, [])

        before_items.append(enter)

        exit = gast.Identifier(scope.namespace, constants.EXIT, 0)
        exit = gast.Attribute(name, exit)
        exit = gast.Call(exit, [])

        after_items.append(exit)

    body = self.block(code.body, scope)
    return gast.With(before_items, body, after_items)
    

  def handler(self, code, scope):
    body = self.block(code.body, scope)
    return gast.Case(code.type, code.name, body)

  def anonymous_function(self, code, scope):
    # function bodies have their own scopes
    scopeName = constants.ANONYMOUS_SCOPE
    # make a deep copy of the current scope
    newScope = copy.deepcopy(scope)
    newScope.set_name(scopeName)
    newScope.set_parent(scope.name)

    args = []

    for arg in code.args.args:
      identifier = arg.arg

      scope.assign(identifier)
      default = None
      args.append(gast.Argument(identifier, default))

    body = self.block(code.body, newScope)
    return gast.AnonymousFunction(args, body)

  def class_def(self, code, scope):
    # extract the name and declare a variable of the same name
    name = code.name
    identifier = scope.assign(name)
    identifier = gast.Identifier(identifier.namespace, identifier.name, identifier.generation)

    bases = []

    for base in code.bases:
      bases.append(self.expression(base, scope))

    # function bodies have their own scopes
    scopeName = name + 'Body'
    # make a deep copy of the current scope
    newScope = copy.deepcopy(scope)
    newScope.set_name(scopeName)
    newScope.set_parent(scope.name)

    body = self.block(code.body, newScope)

    return gast.ClassDef(name, bases, body)
    
  def function(self, function, scope):
    # Helper function for positional arguments in the signature
    def positional_args(args, defaults, scope):
        result = []

        # function without positional arguments 
        if args is None:
            return result

        count = 0
        for i, arg in enumerate(args):
            # name of the identifier
            # _not_ an actual identifier, conform to the argument syntax used when calling
            identifier = arg.arg
            
            # make sure an identifier with this name now exists in the scope
            scope.assign(identifier)

            default = None

            if i >= len(args) - len(defaults):
                offset = len(args) - i - 1
                default = defaults[offset]

                default = self.literal(default, scope)
        
            argument = self.argument(identifier, default)
            result.append(argument)
            count += 1

        return result

    # Helper function for keyword arguments
    def keyword_args(args, defaults, scope):
        result = []

        # function without positional arguments 
        if args is None:
            return result

        count = 0
        for arg, default in zip(args, defaults):
            # name of the identifier
            # _not_ an actual identifier, conform to the argument syntax used when calling
            identifier = arg.arg
            
            # make sure an identifier with this name now exists in the scope
            scope.assign(identifier)

            if default is None:
                default = None
            else:
                default = self.literal(default, scope)
        
            argument = self.argument(identifier, default)
            result.append(argument)
            count += 1

        return result

    # Helper function for vararg arguments
    def vararg(arg, scope):
        # name of the identifier
        # _not_ an actual identifier, conform to the argument syntax used when calling
        identifier = arg.arg
        
        # make sure an identifier with this name now exists in the scope
        scope.assign(identifier)

        # varargs are starred
        return self.starred(identifier, scope)

    # Helper function for kwarg arguments
    def kwarg(arg, scope):
        # name of the identifier
        # _not_ an actual identifier, conform to the argument syntax used when calling
        identifier = arg.arg
        
        # make sure an identifier with this name now exists in the scope
        scope.assign(identifier)

        # kwargs are double starred
        return self.starred(self.starred(identifier, scope), scope)

    # extract the name and declare a variable of the same name
    name = function.name
    identifier = scope.assign(name)
    identifier = gast.Identifier(identifier.namespace, identifier.name, identifier.generation)

    # function bodies have their own scopes
    scopeName = name + 'Body'
    # make a deep copy of the current scope
    newScope = copy.deepcopy(scope)
    newScope.set_name(scopeName)
    newScope.set_parent(scope.name)

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
    positional_args = positional_args(_positional_args, _positional_defaults, newScope)

    if _vararg is not None:
      positional_args.append(vararg(_vararg, newScope))

    keyword_args = keyword_args(_kw_only_args, _kw_defaults, newScope)
    if _kwarg is not None:
      keyword_args.append(kwarg(_kwarg, newScope))

    body = self.block(function.body, newScope)

    return gast.Function(identifier, positional_args, keyword_args, body)