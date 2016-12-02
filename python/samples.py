class A:
    def bar(self):
        return 1

class B:
    def bar(self):
        return 's'

def foo(x, y):
    a = x.bar()
    b = y.bar()

    return a + b








def foo(x, y):
    x += 1
    y += 1

    x = bar(x)
    y = bar(y)

    return x + y


def fun(a):
    a += 1
    return bar(a)


def foo(x, y):
    r




if x < y:
    z = x
else:
    print('y is bigger')
    z = y

z = min(x,y)


if condition:
    x = input()
else:
    x = 'default value'


if condition:
    x = A()
else:
    x = B()

x.foo()




class A:
   def foo(self):
     return 2

class B:
   def bar(self):
     return 'n'

if cond1:
   x = A()
else:
   x = B()

if cond2:
   z = x.foo()
else:
   z = x.bar()


while len(x) < len(y):
    x = '0' + x


def foo(x, y):
    fun = x.__add__
    return fun(y)

a = b + c
d = e + f
d += 1



while cond:
    if volgend in ducci:
        ducci.append(volgend)
        break
    else:
        ducci.append(volgend)
        volgend = volgende(volgend)
else:
    return 'success', ducci
