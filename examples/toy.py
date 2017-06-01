warmedag=0
hetedag=0
x=input()

while x!= 'stop':
    x=float(x)
    
    for x in range(0,5):
        while x >= 25.0:
            warmedag+=1
            x=input()

        while x>=30.0:
            hetedag+=1
            x=input()
        else:
            hetedag=0
            warmedag=0
    
if warmedag==2 and hetedag==3:
    print('hittegolf')
else:
    print('geen hittegolf')