def foo(x=None):
	x + [1]

	if x is None:
		x = []

	x + [1]

foo()