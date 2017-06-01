if 'cond':
	x = 'str'
else:
	x = 2

y = [1, x]
stuff = y + [3, '4']
a, *b, c = stuff 
d, e = b
d + e

b + c