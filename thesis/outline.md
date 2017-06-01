X # Introduction
X ## Initial Goals 
X ### Giving students automatic feedback
X ### Helping them learn more quickly

X # Literature
X ## Static Analysis
X   Why Static Analysis
X   Core concept is avoiding mistakes at runtime (hence static)
X ### Powerful Languages
X    Fake sense of productivity
X      C: Undefined behaviors breaking shit after updating compilers
X      C: Valgrind and commercial tools to track memory leaks
X      Ruby: split
X      Python: refactoring
X ### NaN
X    Python: Gradient becoming pure noise working on NaNs
X ### Restrictive Languages
X   Haskell pure functional
X     No side effects
X     No type coercion
X   Rust tries to find a good middle ground
X     Explicit mutability
X     (Declare something as mutable but never use it)
X     Single ownership
X ### Middle Ground
X   Avoiding Error-Prone Patterns
X   Linters

smells get introduced at the beginning: http://www.cs.wm.edu/~denys/pubs/ICSE'15-BadSmells-CRC.pdf

X ## Inspirations
X ### Code Smells
X    Automatic Refactoring
X    Code Duplication
!!   PDG = data flow + control flow, graph
X    Existing tools focus on industry
X    -> Efficient but inaccurate
X       False positives
X       Made to be validated by experienced software architects
X ## Compilers
X ### Refactoring 
X     Conservative, cannot break the semantics of valid code
X ### SSA
X     Memory SSA on low-level IR, in SSA form and after aliasing analysis. 
X ### LLVM: SSA, MemorySSA
X ### GCC: TreeSSA

# Fosite
X# Area of application
   X Main priority is analysing small code files
   X Efficient (< 1s)
   X Has to run in Docker (little runtime dependencies) -> Rust
   X @coverity claims that reporting the issue is the most important part, otherwise people will ignore the bug and blame the tool. This is even more important when new programmers are involved. Fosite has a heavy focus on reporting the analysis, much more so than currently available tools. The data-flow analysis gives accurate reports of what happens under which conditions.
   X Results have to have a one-to-one relationship with the code, i.e. the analysis shouldn't deform the input beyond restoration. This is necessary to show show potential refactorings to students.
X# Approach
   X First step towards refactoring, abstract interpreter to do a data-flow analysis. Result is similar to an SSA form.
   X Memory SSA like GCC and LLVM is impossible without the low-level representation. PyPy uses an interpreter as well but on the python Bytecode.
   X Try to do as much as possible in a single pass. A single SSA-like form that combines SSA with Memory SSA. 
X# Common modules for Python, JS, Ruby, ...
   X GAST
   X Function scoping can be shared
X# Interpreter vs Linter
   X Linters work directly on and with the syntax
   X The interpreter doesn't just reveal error prone code, but actual errors as well. The data-flow analysis can reveal why actual execution results in a run-time error. Able to recognize advanced X error prone code such as heterogeneous collections.
     Often the first step in debugging 'where does this value come from?'
     X Recognizes Type errors
      X          Forgetting to return
       X         Trivial endless loops

## Implementation

### Core Objects
    Types
      Classes are objects as well (class object)
      An object of a class has a reference to the class object
      Classes have reference to their superclasses (called base classes)
      Python has multiple inheritance, so a class can have multiple base classes
      But any object has only a single type, so what's the type of a class object? 
### Path & Mapping
### Conditionals

### Namespaces (Scopes)
    Frames

### Somewhere
* Reverse mappings
* Containment


# Future Work

   More languages

   Actual operator overloading
   
   Refactoring: https://www3.cs.stonybrook.edu/~liu/papers/Alias-DLS10.pdf comes very close and is one of the main sources of inspiration of this thesis. They run in two phases, an abstract interpreter to build a CFG and find type information which they then use to infer aliasing information. This thesis does all this during the interpretation step, using a SSA-like dependency system instead of an aliasing analysis. They aggregate results as often as possible for memory reasons, which makes accurate reporting of errors impractical.
   Context-Sensitive from path exclusion
   Trace-sensitive from path mergeability
   Precise-type-sensitive is trivially follows from path exclusion and types being modelled like any other object
   Doesn't model any values either, types seem to be sufficient. Symbolic execution could definitely improve the context sensitivity though.
   Our model applies the same principles to collections, which should yield even better results. 

   Symbolic Execution
