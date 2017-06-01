# frame 1
x = 'x'
y = 'y'

while True:
    if 'cond': 
        # frame 2, subframe 2.1
        if 'cond2': 
            # frame 3, subframe 3.1
            x = 9
            break
    else:
        # frame 2,subframe 2
        if 'cond4': 
            # frame 4, subframe 4.1
            y = 7
            break

    z = x + y

z + 9