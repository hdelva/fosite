def square(x, n):
	if n == 0:
		return 1

	y = 1

	while n > 1:
		if n % 2 == 0:
			x = x ** 2
			#n //= 2
		else:
			y = x * y
			x = x ** 2
			n = (n-1) / 2

	return x * y

square(500, 30)