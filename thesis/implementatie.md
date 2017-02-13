### Observation

Every execution branch is either taken, or it isn't. Figuring out which is the case is well-known to be uncomputable, for the simple reason that the branch condition can be arbitrarily hard to evaluate. This implies that in some cases, we can't decide whether or not a given branch gets taken. The best we can do in these conditions is conclude that the branch _might_ get taken. In the following section, we'll denote this possibility with `Maybe`, in line with Python's `True` and `False`.

### Observation

In some cases, we do need a definitive answer. Consider the following examples

```python
if current is None:
  current = datetime.now()
```

```python
if current is not None:
  print('{}-{}'.format(current.year, current.month))
```

The above examples gives a pretty good indication that in some cases, we really need an exact example. This is particularly important for any sort of input validation. The first example is a common pattern for filling in optional arguments, while the second one is just good practice in general. Other examples include checking the length of a certain collections and type-checks. 

### Boolean Expressions with Certainty

There aren't a lot of boolean expressions which we can evaluate with certainty. Luckily enough, the ones that we can do are mostly the ones of interest. The `is` operator for example should compare the addresses of two objects and return `True` if and only if they're equal. So internally, we can mimic this behavior -- and answer with certainty and under which conditions, the objects have the same address in our own analysis. The other possibilities are harder to do, and the best we can realistically return is `Maybe`.

The `==` operator is a special case. If the `__eq__` method was implemented correctly, this should at the very least return `True` if the two objects being compared are the same -- as with `is`. The analyzer in its current state does not properly support analysis of operator overloading, so it will assume that `__eq__` does indeed have a sane implementation. The analysis of the `==` operator in this case becomes the same as the analysis of the `is` operator. Likewise for the `<=` and  `>=` operators.

The `and` and `or` operators are quite obvious. `and` will return `True` if both sides are true with certainty, `False` if either side is false with certainty, and `Maybe` in any other case. The `or` operator is analogous.

### Definition: containment

A path $A$ is contained in another path $B$ if every node of path $A$ occurs in the same way as it does in path $B$
 
## Path exclusion

When executing an execution branch, we should have information about why that specific branch is being executed. If that information includes for example that we are sure that `x` is not `None`, we should disregard any mapping that says the opposite. And even better, we can exclude any mapping that would occur under the same contradictory conditions -- even if those mappings don't have an explicit connection to `x`. For example in the following trivial example:

```python
if cond1:
  y = None
  z = None
  
if y is not None:
  print(z.attribute)
```

In the positive branch of the first condition, there's a point where both `y` and `z` become `None`. After evaluating the second branching condition, we can be absolutely sure that the positive branch of the second branch will not be taken if the positive branch of the taken has been taken. In effect, this means that the mapping for `z` where it receives the value `None`in the first branch is of no use while evaluating `z.attribute`. 

The exclusion of certain mappings is what we'll conveniently call _path exclusion_. We can give this term a more formal representation as well.

Assume that resolving an identifier $x$ resulted in a set of mappings $M$. Every mapping $m \in M$ is of the form $(p, a)$, where $a$ is the address to which $x$ can point, and $p$ is the execution path that's required to get this mapping from $x$ to $a$. 

Call $R$ the set of restrictions; the set of every impossible execution path. If there exists a path $r$ in $R$  for a given mapping $(p, a)$, for which it holds that $p$ is contained within $r$, we can exclude the mapping from the current evaluation. 