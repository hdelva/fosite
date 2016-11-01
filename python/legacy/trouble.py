def trouble1(a, b, c, d):
  c.x = 7 

trouble1(a, *x, c)

"""
Valid code assuming x has two elements
Calling trouble1 will invalidate an element of x
"""


def trouble2(a, b, *c):
  c[0][0] = 7

trouble2(a,b,x,y)

"""
Calling trouble2 will invalidate an attribute of x, even though it's not obvious from the function itself
"""

"""
We obviously need to draw the right conclusions when looking at the call

calling trouble1 or trouble2 will invalidate the first element of x, and thus x as a whole

trouble2 does not invalidate y however, so we shouldn't just say everything in the vararg gets invalidated

On the other hand, 
when reporting the invalidations of a function call, 
it has to be in terms that are transparant to the outside world.

Solutions:
  Keep track of how variables got introduced:
    For positional arguments, keep track of which position 
      Store the total number of positional arguments
    For vararg: remember it was a vararg 
    For keyword arguments, remember the argument names
    For other variables, do not nothing in particular

  When resolving invalidations in function bodies:
    Local variables live in the function's scope
    Only attributes and/or elements can cause invalidations
      This is either done by an assignment in the function body
      Or a call to a function which has these side effects (careful of recursion)

    Trace the type of the variable which gets invalidated
      In case it was a positional argument, report the argument position
      In case it came from the *arg, find which position in the vararg, 
        report the position + total number of regular positional arguments
      In case it came from a regular keyword argument, report the argument name
      In case it came from a **kwarg, report the argument name
      In other cases: if the variable namespace is '_extern', return the identifier

  Call-site interpretation of invalidations:
    positional invalidations -> find the corresponding variable name in the current scope
      if the position is larger or equal to the number of positional arguments, 
      the last positional argument is now invalidated (varargs must be last)
    keyword invalidations -> find the variable that was passed using this name
      if it's not there, it was provided as a keyword vararg
      the last keyword argument is now invalidated
    '_extern' variable, traverse the scope hierarchy upwards until the referenced variable is found
      if nothing is found, the code's broken and we can just abort
"""




