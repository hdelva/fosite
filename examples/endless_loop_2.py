a = [1, 2, 3, 4]

while 1 in a:
	if 'cond1':
		#x.attr = 'invalidated'
		if 'cond2':
			pass
		else:
			a[1] = 'invalidated'
	else:
		a[1] = 'invalidated'