# arguments; x ** n
# source: https://en.wikipedia.org/wiki/Exponentiation_by_squaring
x = 6
n = 980

if n == 0:
	1

y = 1

while n > 1:
	if n % 2 == 0:
		x = x ** 2
		#n //= 2
	else:
		y = x * y
		x = x ** 2
		n = (n-1) // 2

x * y