def foo(x, y, *args, z, w=None, **kwargs):
	x + y
	z + w
	x + args[0] + args[1]
	v
	#kwargs[x]

v = 7
foo(v, [])
foo('x', 'y', 1, {}, z='z', w=(), k='v')
x + y