'''
Created on 24 okt. 2016

@author: Axelle
'''
reeks = input()
ontbrekend = None

for i in range(1, len(reeks)):
    # eerste getal bepalen
    eerste_getal = int(reeks[:i])
    # rest van de reeks bepalen
    rest = reeks[i:]
    
    verwacht = str(eerste_getal + 1)
    
    # zolang er geen getal ontbreekt ga je verder met de verkorte rest
    while True:
        if rest[:len(verwacht)] == verwacht:
            rest = rest[len(verwacht):]
        elif ontbrekend is None:
            ontbrekend = verwacht
        else:
            ontbrekend = None
            break
        
        verwacht = str(int(verwacht) + 1)
        
        if len(rest) < len(verwacht):
            if len(rest) != 0:
                ontbrekend = None
            break
        
    if ontbrekend is not None:
        print(ontbrekend)
        break

        
    
    
if ontbrekend is None:
    print('geen ontbrekend getal')