% Static Analysis of Dynamic Languages
% Harm Delva

# Introduction

The university of Ghent has developed the Dodona platform which students can use to submit solutions to and receive immediate feedback. This feedback is in large part done through unit testing. This lets the students know whether or not their solution is correct but doesn't really help them forward if it's not. To remedy this, there is a visual debugger included which students can use to step through their program. This comes with a couple of caveats unfortunately, for example it's unable to process file IO. More importantly, it can only let the user scroll through execution states. If an error occurred after a couple of thousand executed statements, this becomes a very tedious process.

That's just the nature of debugging. Fortunately, there are ways to avoid this pain altogether. Some programming languages have a strict compiler to catch obvious mistakes. There are linter tools such as JSLint and Pylint which emit warnings on error prone patterns. The Dodona platform uses these tools as well, and relays the warnings to the students. 

The Pylint tool for Python has noticeably helped students avoid mistakes. Which in turn allowed them to focus more on the actual assignment and less on learning how to program. The goal of this thesis is to build on top of that, giving the students even more and even better feedback. An extensive data-flow analysis can untangle even the worst spaghetti code, which makes it a prime starting point for further analysis and feedback. 

# Literature 
## Static Analysis

NASA's source code of the Apollo 11 Guidance Computer was recently digitized and put on Github. All of it was written in an Assembly language and the result worked just fine. The C programming language was made for the Unix operating system so it could run on various CPU architectures. In essence it's a pretty light abstraction over Assembly, it doesn't take much pressure off of the programmers. Unix did just fine as well. One could argue that programmers don't need tools to aid them -- they seem to know what they're doing. 

As the field of research grew, and with it the projects, it started becoming apparent that programmers can't always rely on themselves. Even veteran developers occasionally discover a major vulnerability in their code -- like the Heartbleed vulnerability in OpenSSL. Everyone makes mistakes but not all mistakes go unnoticed for years in critical code. A first line of defense against these costly mistakes is a proof of correctness. NASA's code was reliable because they had formal proofs that the most critical parts were correct [@nasa1; @nasa2; @nasa3]. Doing this is not only a massive amount of work, a proof can still be wrong. An implementation oversight can easily carry over into a proof oversight, so extra pair of eyes is needed to check all proofs as well. 

Functional programming languages are closely related to this method of provable correctness, while also automating some of checks. Notable examples of such languages are Haskell and ML. Both have a strong theoretical foundation and provide the programmer with a strong type system to make code easier to reason about. This stands in stark contrast with languages like C. While Haskell was made to facilitate writing correct code @haskell, C was made to be _close to the metal_ and efficient @cdev. The C compiler doesn't help the programmer nearly as much as the Haskell compiler. Correctness is obviously paramount to any computer program, so C's priorities don't always align with a programmer's. 

That's where static analysers come into play. They analyze a program, either in its source code form or its compiled form, and try to find as many possible mistakes as possible. These mistakes are often very subtle and rare,  but encountering one can ruin a person's week. Code sample \ref{smp:shortset} comes from Google's Error Prone Github page and is a great example of how subtle serious bugs can be.

\begin{code}
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=java,
  minted style=autumn,
  minted options={},
  colback=bg }
public class ShortSet {
  public static void main (String[] args) {
    Set<Short> s = new HashSet<>();
    for (short i = 0; i < 100; i++) {
      s.add(i);
      s.remove(i - 1);
    }
    System.out.println(s.size());
  }
}
\end{tcblisting}
\caption{Short Set}\label{smp:shortset}
\end{code}

This code seems to be just fine at first glance, but sample \ref{smp:shortset_f} contains the analysis.

\begin{code}
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=HTML,
  minted style=autumn,
  minted options={},
  colback=bg }
error: [CollectionIncompatibleType] Argument 'i - 1' should not be 
passed to this method; its type int is not compatible with its 
collection's type argument Short
      s.remove(i - 1);
              ^
\end{tcblisting}
\caption{Analysis of code sample \ref{smp:shortset}} \label{smp:shortset_f}
\end{code}

Subtracting an `int` from a `short` resulted in an `int`. In this case it won't cause an actual bug yet because both types use the same hashing function. The JVM originally didn't support generics, and their implementation is still a bit rough. The `remove` method of a `List` instance doesn't do any type checks, its argument can be any `Object` instance. This can result in calls that never actually remove anything. If that call happens to only occur in a corner case in the $1000^{\text{th}}$ iteration of a loop, this can lead to some very confusing bugs. 

### Powerful Languages

Every commonly used programming language is Turing complete, so in theory they should all be equal. This is a misconception that's been called the Turing tar-pit @tarpit. Everything is possible in a Turing tar-pit, but nothing of interest is easy. Programming languages where things of interest are perceived to be easy can be considered powerful. These languages are often the ones with lenient compilers or runtime environments such as C, Python, Javascript, ... In other words, languages that don't get in the way of the programmer too often. 

As illustrated in the previous section this may not always be a good idea as humans tend to glance over subtle details. This makes the need for additional tools very important for any project that aims to achieve high reliability. The alternative is long and painful debugging sessions. At some point these languages no longer make it easy to do things of interest in. 

#### C


The C programming language has been one of the most popular programming languages for a couple of decades now. Depending on who you ask, the best thing about C is either its efficiency or its simplicity, and both come at a price. Its simplicity is what gives developers the power they desire. This comes at a cost however; with great power comes great responsibility.

Let's focus on the other main attraction of C, the efficiency. This comes at a cost as well and it's a one many people forget about. C's _raison d'être_ isn't making developers feel good about themselves, it's generating efficient code. It was created to be a thin abstraction over various assembly languages that were limiting software development at the time @cdev. Since not all CPU architectures handle a division by zero the way, the C specification doesn't specify what to do. 

##### Undefined Behavior

There are a lot of things the language specification doesn't specify a behavior for, which leads to undefined behavior. Some are well-known, such as using freed memory. Others catch people by surprise, dividing by zero in C for example is undefined behavior. The GNU libc website still claims that `1/0` yields `inf` @gnu, even though the C99 clearly contradicts them @C99. The C99 standard did introduce the `INF` macro, but it doesn't specify which operations should result in one. Division by zero is still as undefined as it has always been. 

Entire papers have been written on the subject of undefined behavior [@lattner; @wang; @guide]. One striking thing is how recent a lot of these papers are. Even though the language is over 40 years old, this is still an active field of research. Compilers are getting more advanced and with it the optimizations they perform. 
 
Code sample \ref{smp:undef} was part of PostgreSQL @wang.

\begin{code}
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=C,
  minted style=autumn,
  minted options={},
  colback=bg }
if (arg2 == 0)
  ereport(ERROR, 
    errcode(ERRCODE_DIVISION_BY_ZERO),
    errmsg("division by zero"));
                    
/* No overflow is possible */
PG_RETURN_INT32((int32) arg1 / arg2);
\end{tcblisting}
\caption{Undefined Behavior}\label{smp:undef}
\end{code}

The `ereport` function never returns, it does some logging before calling `exit`. In the mind of the developer this prevents the division by zero on the next line. Looking at the code on Github, the function this sample came from indicates that calling it will return a `Datum` struct. The body of the null check does not return anything, so the compiler concludes that the rest of the function will also be executed and division by `arg2` will always get occur. 

Division by zero is undefined in C, so the compiler concludes that `arg2` won't ever be zero -- it wouldn't get used in a division otherwise. As a result, the null check gets flagged as dead code, and is removed entirely. 

Code sample \ref{smp:undef_f} contains the fixed code. By adding an explicit return (in the form a macro), the compiler leaves the null check intact.

\begin{code}
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=C,
  minted style=autumn,
  minted options={},
  colback=bg }
if (arg2 == 0)
{
  ereport(ERROR,
      (errcode(ERRCODE_DIVISION_BY_ZERO),
       errmsg("division by zero")));

  /* ensure compiler realizes we mustn't reach the division 
  (gcc bug) */
  PG_RETURN_NULL();
}
\end{tcblisting}
\caption{Fixed Undefined Behavior}\label{smp:undef_f}
\end{code}

Notice how the comment blames a "gcc bug". This illustrates how even experienced developers seem to misunderstand their language of choice. 

##### Tools

Not a single other programming language comes close to having as many external tools as C (and by extension C++). Many developers heavily depend on these tools in their usual workflow. One of the most established ones are the ones in Valgrind.

Valgrind is a suite of dynamic analysis tools, the most famous of which is Memcheck. Memory related bugs are some of the hardest to track down because most of them are because of undefined behavior. For example, using a memory location after freeing might not always crash the program. There's an informal term for bugs like these: _heisenbugs_. Something might go wrong but when you try to isolate the cause everything seems to be just fine. Especially since Address Space Layout Randomization (ASLR) tends to be disabled during debugging but not during normal execution. 

This is where Memcheck comes into play. It analyses the program during execution and keeps track of what happens to the memory. This way it can notice memory related bugs such as use-after-free and report them back to the developer. Unfortunately it's not a perfect solution. There can be false positives as well as false negatives, and it is quite incompatible with some libraries such as OpenMPI @mpi. 

A lot of companies rely on analysis tools to manage their large C projects, and when there's demand in a market, the supply will follow. There's an impressive amount of commercial analysis tools available. Coverity is one of the most established ones. 

Dawson Engler is one of Coverity's co-founders is one of the leading researchers in the field of static analysis. He also co-authored a great paper in which he describes how difficult static analysis is in the real world @coverity. One particularly interesting part of the paper explains that there's a fundamental misunderstanding of what programming languages and compilers are. A programming language can exist as an abstract idea or a piece of paper. While the language a program is written in is whatever the compiler accepts. In other words, compilation is not certification. A good first step for a static analysis tool is to make sure that the input adheres to the specification. They go on to pose the hypothetical question: "Guess how receptive they are to fixing code the “official” compiler accepted but the tool rejected with a parse error?"

Even with a plethora of tools available, C remains an untamed beast. Some tools like Valgrind are great at what they do but are still limited. Other tools like Coverity seem to fight stubborn developers as often as they fight bugs. 

#### Dynamic Languages

Moving on to the topic of this thesis, dynamic languages. According to Guido Van Rossum, he made Python because he wanted to make a descendant of ABC that would appeal to Unix/C hackers @lutz. After reading the previous section some spidey senses should start tingling. 

Javascript's situation isn't great either. There are no namespaces, no modularization system, no classes, no interfaces, no encapsulation, ... Eric Lippert, who was on the ECMA committee during the early days has a Stackoverflow post where he discusses why Javascript is so ill-fit for static analysis and programming in the large @lippert. Even though Stackoverflow is a dubious source for any academic work, the author's experience should make up for it. 

Both languages are known as productive languages. Developers don't spend a lot of time writing boiler plate code or declaring types, they can get right to the good part. The dynamic nature of the languages even makes it easy to write generic functions, which makes code reuse easier and should make the code easier to maintain. 

As the following anecdotal examples illustrate, this isn't always the case. Most people who have used dynamic languages should be able to confirm that they can misbehave in the spectacular ways. The examples are supposed to be relatable and bring back memories of frustrating debugging sessions.

##### Classes

Lots of developers are familiar with class hierarchies and like using them. There is one glaring difference between how static languages and dynamic languages handle classes however. In most static languages, the class definition declares which methods and attributes are defined for objects of that class. This isn't the case with dynamic languages, adding a method to an object is as simple as assigning to a callable object to an attribute. 

This is bad news for a static analysers that try to do type inferencing on dynamic languages. Consider the following problem that occurred while working on the Dodona platform.

The Ruby builtin `String` class provides a `split` that does the same thing as in most other languages: its argument is some separator which it uses to cut the `String` object into an array of smaller `String` objects. Calling the `split` method on some object returns the same object. Using `puts` on the object just shows the same exact String, weird. The documentation says that it should work. Testing the method in a REPL environment confirms that the `split` method should split the `String`. A few confused hours later, it turns out the object wasn't a `String` but an `Array`. The Ruby documentation doesn't mention that `Array` defines a `split` method. Googling "ruby array split" refers to the Ruby on Rails documentation. A library silently declared a method on a builtin type, and `puts` prints every element of an `Array` on a separate line, so it just prints a single `String`. 

This sort of reckless approach to classes has some serious implications. Not only does it confuse newcomers, it confuses static analyzers as well. Due to the lack of type declarations, a logical alternative would be type inference. But consider the following case, `split` is being called on something that's either an `Array` or a `String` at the current stage of  inferencing. A regular Ruby analysis tool might conclude that the object must be a `String`. Analyzing Ruby on Rails code would either require the tool to analyze the entire framework's code first to learn that it adds a `split` method to `Array`, or it might even require a custom analyzer for Rails altogether.

This is why static analysis of dynamic languages is typically done using an extensive data-flow analysis @madsen. You don't have to infer the type of an object if you know where it came from.

##### Refactoring

Python's lack of structure makes it hard to work incrementally. At some point during implementation of the interpreter, line and column information had to be added to the AST structure. Python's `ast` module contains everything one could need to work on Python's AST, including `lineno` and `col_offset` for seemingly all classes. With the exception of a few classes, such as `generator`. Implementing generators didn't come until much later, long after the analyser started relying on the existence of `lineno` and `col_offset` for every node. 

Refactoring dynamic languages is a challenge. What if we want to change the name of a method called `add` to `insert`. Can we replace all occurrences of `.add` to `.insert`? That might change other classes as well. As discussed in the previous section, type inferencing is non-trivial. Even IDEs that are renowned for their ability to refactor code, such as the JetBrains IDEs, rely on manual feedback to filter out the false positives. Reporting false positives is not always an option however. That's what causes people to question your tool @coverity. 

### NaN

As an aside, the existence of NaN should be considered error prone as well. Some languages like Python and Java do a good job at preventing `NaN` from entering your program through raising exceptions, but once they do find their way in you're left at the mercy of the IEEE 754 standard. The most recent version of the standard is IEEE 754-2008 and was heavily influenced by the ISO C99 standard, which introduced `NaN` and $\inf$ to the C standard.

Since this standard, if one of the function's argument is `NaN` but the other arguments already determine the function's value, the result is no longer `NaN`. For example, this means that `pow(1, NaN)` is equal to `1`. Mathematically this is at least a bit dubious, $1^{0/0}$ shouldn't be defined if $0/0$ isn't either. The C99 standard introduced various other oddities @C99. $\texttt{pow(-1, }\pm \inf\texttt{)}$ is `1` because all large positive floating-point values are even integers. And having a `NaN` in the imaginary part of a complex number does not matter when the value is being cast to a real number, it's apparently possible for a value to be only slightly undefined. 

It has become normal for `NaN` values to somehow disappear as if nothing was wrong, leading to some very confusing bugs. Jonathan Peck ran into the consequences while implementing his own thesis (personal correspondence). The Theano framework has a function to compute the gradient of some expression. A `NaN` found its way into the input of this function but the result somehow didn't contain a single `NaN`, it became seemingly random noise instead. When implementing something algorithm intensive, any person's first instinct would be that the algorithm is wrong. 

Static analysis should be able to help alleviate this problem. If the floating-point value that's being used came from a `sqrt` function, there should be an `isNaN` check first. Alternatively, the Rust programming language tries not to hide the fact that floating-pointing values can be `NaN`. Without providing your own implementation of the `Ord` trait for the `f64` type, it's impossible to order a vector of `f64` values because comparing two `f64` values might be meaningless. It does however provide an implementation of the `PartialOrd` trait, which returns an `Option<Ordering>`, which makes it clear that the result can be `None`.
 
### Safe Languages

Now that it's firmly established that some languages gladly let their users shoot themselves in the foot, let's look at languages that protect their users. Most of these languages are functional languages but there are notable exceptions such as Ada. 

#### Haskell

Having strong ties to the lambda calculus @haskell, Haskell is the archetype of a safe language. Being a pure functional language, all state in a Haskell program is implicit and by extension there are no side-effects. One of core concepts of the language is that Haskell code should be easy to reason about. That's why this language deserves a section in a thesis about static analysis and data-flow analysis; Haskell's design makes these things pleasantly simple. 

\begin{code}
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=python,
  minted style=autumn,
  minted options={},
  colback=bg }
def reset(arg):
  arg.attr = None

x = A()
x.attr = 4
y = x

reset(y)

x.attr += 1
\end{tcblisting}
\caption{Aliasing}\label{smp:aliasing}
\end{code}

Consider the Python code in Code sample \ref{smp:aliasing}. Knowing that `x` and `y` are the same is integral to realizing that `x.attr += 1` will result in a type error, as `x.attr` is None since the call to `reset`. Haskell has no side-effects -- function calls like `reset(y)` wouldn't be able to change anything. Additionally, it has no explicit state and thus no assignments and no aliasing to begin with. This trait can be mimicked in other languages using the Static Single Assignment (SSA) form where every variable is immutable. A later section will discuss this form and how it relates to code analysis.

#### Rust

A newer language that favors safety over accessibility is Rust. While it does have explicit state, Rust also makes the aliasing problem trivially easy for static analysers. With some exceptions like the reference counting types, every piece of memory has a single owner in Rust -- which means that by default there's a single way to access any piece of data. This prevents aliasing problems like the one in sample \ref{smp:aliasing} because after executing `y = x`, `x` is no longer a valid identifier -- its data has been moved to `y`. On top of that, Rust has explicit mutability. This concept came from C/C++ where it's good practice to label all immutable things with `const`, except that Rust does it the other way around. This means that unless `y` was declared to be mutable, the assignment to `y.attr` wouldn't compile either. 

In languages like Java which heavily advocate encapsulation it's not uncommon to write something like a `List<Element> getElements()` method, so that other modules can see which data is present. Returning the original list would mean anybody can change its contents. That's why it's considered good practice to return a deep copy of the list instead. Deep copies carry a significant overhead with them, so developers end up choosing between a safe architecture or an efficient implementation. Rust lets data be borrowed, so that other modules can see its contents. The result is a reference which is very similar to a pointer in C, with the exception that borrowed references carry a lot more type information with them. For starters, it's possible to borrow something mutably or immutably. If there is a single mutable borrow to an object, there can't be any other borrows at the same time. This is a mechanism that's supposed to avoid unexpected side-effects. Another thing that's part of a borrow's type is its lifetime. If object A wants to borrow something from object B, B has to exist at least as long as A. This mechanism directly prevents all sorts of memory related bugs that occur in C.

Rust is an interesting example of the relationship between languages (more specifically the compilers) and static analysers. The Rust compiler enforces a lot of things that the C compiler doesn't but that the C community has written their own tools for. One might wonder why C is even still around if Rust seems to be a more complete package. Part of the answer is that Rust is a very restrictive language which leads to frustrated developers. Keeping in mind Coverity's paper @coverity, it's a lot easier to ignore a analyser than it is to fix a compile error. 

In fact, this has received the informal label of being an _XY problem_. If a person knows how to do something using method X, he's less likely to learn method Y -- even if method Y is clearly the better choice. Rust has received a lot of criticism from renowned developers for ridiculous reasons, such as being unable to have two mutable references to the same object. That's how they'd do it in C, so it must be the right way. This problem is relevant in this thesis for two reasons. For starters, static analysis tools run into the same mentality issues @coverity. More importantly, it shows that it's important to never pick up bad habits in the first place. The field of application of this thesis is ultimately helping students learn how to program, before they even have any bad habits. 

### Middle Ground

It's unrealistic to expect that all programs be rewritten in safe languages. That doesn't mean that we should abandon all hope of having reliable programs though. This is why analysis tools have emerged; to fill the void. 

There is a class of static analysers that's particularly interesting, the linters. What sets them apart from other tools is that they report error prone patterns rather than actual errors. Preventing bugs is better than fixing them. One of the most renowned linting tools is JSLint, its Github page contains the following wisdom @jslint:

\say{The place to express yourself in programming is in the quality of your ideas and
the efficiency of their execution. The role of style in programming is the same
as in literature: It makes for better reading. A great writer doesn't express
herself by putting the spaces before her commas instead of after, or by putting
extra spaces inside her parentheses. A great writer will slavishly conform to
some rules of style, and that in no way constrains her power to express herself
creatively.}

Few developers should find anything anything wrong in that reasoning. The main point of contention is which rules to follow. Crockford is a proponent of using certain language features sparsely @goodparts. In his book titled Javascript: The Good Parts he describes which features of Javascript he deems good (or great and even beautiful) and which parts should be avoided. The bad parts are everything that lead to error prone code in his experience, the usual example is the `switch` statement. He has given a presentation at Yahoo where he gives more examples of why JSLint discourages the patterns that it does @crockford. 

## Inspirations

Analysis tools all serve a common purpose but are ultimately still very diverse. Static and dynamic analysis tools do things very differently and even within static analysis tools there's a lot of variation. Linters work mostly syntactical, focusing on speed and immediate feedback. Other tools like Coverity do a much deeper analysis and usually run nightly. Another major difference among static analysers is soundness. Generally speaking, sound analysis tools are based on formal logic, slower, more comprehensive but more prone to false positives. Unsound methods seem to coming out on top as most practical languages are unsound themselves @unsound. Others simply say that it doesn't matter how you do it, as long as the results are good @coverity.

### Code Smells
\begin{code}
\centering
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=python,
  minted style=autumn,
  minted options={},
  colback=bg }
x += 1
y += 1
z += 1

x = sqrt(x)
y = sqrt(y)
z = sqrt(z)

u = x - 1
v = y - 1
w = z - 1
\end{tcblisting}
\caption{Duplication}\label{smp:duplication}
\end{code}

More important than how to do the analysis is perhaps what to look for. Code smells are a taxonomy of indicators that something may not be right in the code. The term was coined by Kent Beck in a book on refactoring @refactor. In that book code smells are indicators that some refactoring might be necessary. Aimed at software architects, they're informal descriptions of things that may induce technical debt. They're not actual bugs yet but error prone patterns, the sort of things Linters aim to detect. One of the code smells ins _cyclomatic complexity_, i.e. having too many branches or loops. This is something Pylint also recognizes. What Pylint lacks however is the refactoring aspect of the code smell. Code smells were meant to be refactored manually by professional software architects, but in an educational setting you have to take into account that the students may not be able to refactor it themselves. 

\begin{code}
\centering
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=python,
  minted style=autumn,
  minted options={},
  colback=bg }
def foo(x):
    x += 1
    x = sqrt(x)
    return x - 1

u = foo(x)
v = foo(y)
w = foo(z)
\end{tcblisting}
\caption{Refactored version of sample \ref{smp:duplication}}\label{smp:duplication_f}
\end{code}

There are also code smells that linters currently do not pick up because they'd require deep analysis. The most notable of which would be the _Code Duplication_ smell. JetBrains has developed some of the best refactoring IDEs such as IntelliJ and PyCharm, but started off by developing code refactoring tools such as IntelliJ Renamer for Java and Resharper for C#. Made for commercial code bases, these tools are ultimately advanced heuristics that still miss obvious duplication. PyCharm 2016 was unable to find the duplication in code sample \ref{smp:duplication}. Even it should probably be refactored to something like code sample \ref{smp:duplication_f}.



\begin{code}
\centering
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=python,
  minted style=autumn,
  minted options={},
  colback=bg }
x += 1
x = sqrt(x)
u = x - 1

y += 1
y = sqrt(y)
v = y - 1

z += 1
z = sqrt(z)
w = z - 1
\end{tcblisting}
\caption{Alternative order of sample \ref{smp:duplication}}\label{smp:duplication_2}
\end{code}

JetBrain's tools are closed source so it's unclear whether or not \ref{smp:duplication} was deemed too simple to refactor. Assuming they work in a similar fashion as competing tools however, it's the order of statements that causes it to fail. Tools like Simian, Sonar, PMD, and Dup Loc all work on a token stream in the same order as it appears in the file. Code sample \ref{smp:duplication_2} illustrates how sample \ref{smp:duplication} could be reordered so that tools can detect the duplication just fine. 

\begin{code}
\centering
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=python,
  minted style=autumn,
  minted options={},
  colback=bg }
x = []

s = int(input()) + 1
x.append(s)
s = int(input()) + 1
x.append(s)
s = int(input()) + 1
x.append(s)
s = int(input()) + 1
x.append(s)

\end{tcblisting}
\caption{Duplication}\label{smp:duplication_3}
\end{code}

When analyzing student code, this can be a serious limitation. Consider code sample \ref{smp:duplication_3}, which just reads 4 numbers from `stdin`, increments them, and stores them in `x`. Even if this duplication gets detected, most tools are targeted at professional developers who would never write code like this. The critical difference is that the refactoring shouldn't introduce a new function but a loop such as in \ref{smp:duplication_3_f}.

\begin{code}
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=python,
  minted style=autumn,
  minted options={},
  colback=bg }
x = []

for _ in range(4):
  s = int(input()) + 1
  x.append(s)
\end{tcblisting}
\caption{Refactored version of sample \ref{smp:duplication_3}} \label{smp:duplication_3_f}
\end{code}

Compilers face the same problem; their refactorings are optimizations. We'll revisit this problem in a later section about the Static Single Assignment (SSA) form. 

There are some other code smells besides code duplication that could be interesting for new programmers. The following list contains some that at first glance look like the most promising.

  * _Large class_: A class that's grown too large and probably has too many responsibilities.
  * _Long method_: Much like the previous one, this method probably has too many responsibilities.
  * _Inappropiate intimacy_: When a class heavily depends on implementation details of another class.
  * _Excessive use of literals_: Also known as magic numbers, these should probably become descriptive constants.
  * _Cyclomatic complexity_: Too many branches, also informally referred to as _spaghetti code_. Linters do a fair job at pointing them out but offer little help in fixing them.

### Compilers

Static analysers aren't the only tools that aim to make code better -- compilers do so as well. Refactoring from a software architect's point of view is aimed at making the code easier to read and maintain. Optimizations are aimed at making code more efficient. These two things sometimes do completely opposite things, unfolding the loop in \ref{smp:duplication_3_f} results in sample \ref{smp:duplication_3} for example. Both operations transform code in a conservative manner though, i.e. without changing the semantics (of valid code). Optimization is a very active area of research, with companies like Google and Apple working on the LLVM, Oracle on the JVM, Red Hat on the GCC, ... More importantly, even the most esoteric features in compilers have proven their worth as they're part of a real product, which gives confidence that a static analysis that uses the same principles will work as well.

#### SSA

Static Single Assignment (SSA) form is a program representation that's well suited to a large number of compiler optimization such as constant propagation @constant, global value numbering @equality, and code equivalance detection @equivalent. The term was first used in a paper by IBM @equality in the 80s but had little practical success initially. It wasn't until several optimizations were found [@increment; @dominance] that it started becoming popular. Since then it's found its way into most popular compilers. LLVM has used it for its virtual instruction set since its inception in 2002 @lattner, it's what powered Java 6's JIT @hotspot, and it's what's behind recent GCC optimizations [@memoryssa; @treessa].

The idea is very simple, every variable can only be assigned to once. This can be done by adding incremental numbers to each variable. For example, `x = x + 1` becomes $\texttt{x}_1 \texttt{ = x}_0 \texttt{ + 1}$. This is a trivial transformation for straight-line code but becomes a bit harder when dealing with branches. Figure \ref{fig:ssa} illustrates how to handle such code. Every branch introduces new variables as before and a _phi-node_ gets inserted at the of the branch. This node mimics a function call which can return either of its two arguments. This doesn't correspond to an actual function call, it's just a token that gets inserted to help with the analysis. 

\begin{figure}
    \centering
    \begin{subfigure}[t]{0.44\textwidth}
      \resizebox{\linewidth}{!}{\input{figures/ssa1.tikz}}
      \caption{Original form}
      \label{fig:ssa_before}
  \end{subfigure}
    \begin{subfigure}[t]{0.55\textwidth}
      \resizebox{\linewidth}{!}{\input{figures/ssa2.tikz}}
        \caption{SSA form}
        \label{fig:ssa_after}
    \end{subfigure}
    \caption{SSA transformation}
    \label{fig:ssa}
\end{figure}

##### Memory SSA

The SSA form has a lot of benefits, as cited in the beginning of this section, but is limited to scalar values and is not well suited for structures and arrays. One of the solutions to this problem is Memory SSA as implemented by GCC @memoryssa. This is a transformation that runs on a sufficiently low-level representation of the source code. It has to be low-level so that there's a notion of `LOAD` and `STORE` instructions. 

Because the actual memory locations are not known during compilation some abstractions are needed. GCC uses compiler symbols which they called tags to represent regions of memory along with two virtual operators `VDEF` and `VUSE`. For every `LOAD` a `VUSE` of some tag(s) gets inserted, and likewise for `STORE` and `VDEF`. 

There are three types of tags:

  * _Symbol Memory Tag_ (SMT): These are the result of a flow-insensitive alias analysis and is almost purely type-based. For example, all dereferences of an `int*` receive the same tag.
  * _Name Memory Tag_ (NMT): These are the result of a points-to analysis applied after the program is in SSA form and inherits its flow-sensitive properties @memoryssa. In other words, multiple SSA forms get used. 
  * _Structure Field Tags_ (SFT): These are symbolic names for the elements of structures and arrays. 

Tags get used in a similar way as variables in the regular SSA algorithm. Assigning to a tag is like assigning to all symbols that have received that tag. In the original implementation, _call clobbering_, i.e. a function call overwrites some memory of an argument, such as in code sample \ref{smp:aliasing} is handled the same way as global variables @memoryssa. Quoting the authors: \say{All call clobbered objects are grouped in a single set}. The current implementation has rewritten all call-clobber related code to use an _aliasing-oracle_ @oracle.

LLVM's usage of Memory SSA is not quite as documented. Their documentation refers to GGC's paper [@memoryssa; @memoryssallvm] and mention that \say{Like GCC’s, LLVM’s MemorySSA is intraprocedural.}. As mentioned in the previous section, this isn't entirely true for GCC anymore. It doesn't seem to be true for LLVM either, a recent publication describes _Static Value-flow_ analysis which produces an interprocedural SSA form @svf. It has been part of LLVM since 2016 and like GCC's implementation it uses the results of an external points-to analysis.

### Symbolic Execution

Rather than using concrete values, symbolic execution uses symbolic values. Those values represent possible values through a set of constraints. This technique is commonly used to find interesting corner cases for unit testing [@symb1; @symb2; @symb3] and the original SSA paper used similar principles as well to make their analysis more precise @equality.

At each branch point a _path constraint_ gets introduced, which is a symbolic expression that must be true for the current branch to be taken. If there are no values that satisfy all the constraints, the branch gets flagged as dead code. In the case of a static analysis tool this will most likely result in a warning, a compiler will just remove the dead branch. For example, consider code sample \ref{smp:symb}. `y > 0` becomes a constraint during the execution of the positive branch, as well as `x > -1`. The former constraint is pretty simple to add, the latter requires some serious bookkeeping. Symbolic execution is closely related to data-flow analysis.

\begin{code}
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=python,
  minted style=autumn,
  minted options={},
  colback=bg }
x = int(input())
y = x + 1

if y > 0:
    z = sqrt(y)
else:
    z = 0
\end{tcblisting}
\caption{Symbolic Execution} \label{smp:symb}
\end{code}

Symbolic execution is a very powerful tool but comes with a few limitations. One is the _state explosion problem_, the number of paths can increase _very_ fast. The Java Pathfinder manages this problem using state matching and backtracking @symb2. State matching will check if a similar state has already been processed, if it has it will backtrack to the next state that still has unexplored choices. 

Another limitation is that symbolic executors typically work on scalar values. Things get a lot harder when dealing with structures and collections. Consider code sample \ref{smp:symb2} for example which defines a function `foo` that prints `homogeneous` if and only if its argument is a collection and all elements in that collection are of the same type. Not only does the positive branch introduce a constraint on the length of `x`, all elements have to be of the same unspecified type as well. The most advanced symbolic executor for Python seems to be PyExZ3 but even that one doesn't handle heterogeneous collections. The relation between uniqueness and the length of a set is also non-trivial, even though it's a very _pythonic_ pattern. 

\begin{code}
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=python,
  minted style=autumn,
  minted options={},
  colback=bg }
def foo(x):
  if len(set(map(type, x))) == 1:
    print('homogeneous')
  else:
    print('heterogeneous')
\end{tcblisting}
\caption{Symbolic Execution Challenge} \label{smp:symb2}
\end{code}

# Fosite

Named after the Frisian god Fosite and foresight, Fosite is the name of the abstract interpreter developed for this thesis. A relatively unknown god, Fosite is the god of reconciliation, justice, and mediation. These are also qualities that a static analyser should aim to have to gain a user's trust @coverity.

## Area of Application

Unlike most existing tools, Fosite's focus is analysing small student submissions. This has both advantages and disadvantages. The biggest advantage is that we're free to explore precise but inefficient methods. Not entirely free however, since feedback should be fast ($< 1s$) because a lot of students will send a lot of submissions at the same time. Imagine being a stressed out student, working on your final exam and every submission takes a minute to run because submissions come in faster than they get processed. It would also be nice if the tool has little runtime dependencies, since by the Dodona design it'll have to run in Docker containers. These two requirements make the Rust programming language a good fit, since it generates fast native code without the unexpected problems C and C++ might induce.

Perhaps the most requirement of all, any warnings have to be as detailed as possible. More details should make the errors more convincing and will be met with less resistance from the user @coverity. This is important when dealing with new programmers; simply pointing out bad style is not enough if that person does not know how to fix it themselves. Fosite uses a data-flow analysis to pinpoint the source of problems and uses that information to inform the user. 

The analysis should work as close as possible to the submitted code and at the very least maintain a one-to-one relationship to it. This is important because the end goal is automated refactoring, which becomes hard when the input becomes mangled beyond recognition. 

## Approach

As a first step towards automated refactoring, Fosite is an abstract interpreter with the intention of doing a data-flow analysis. This by itself isn't entirely new; PyPy uses a similar principle to power their optimizations @pypy. PyPy's solution is intraprocedural though which isn't ideal. Others have successfully used abstract interpreters to perform a may-alias analysis on Python with the goal of optimization @interpret. Fosite is based on their conclusions and results but is ultimately made for a different cause, i.e. detailed static analysis. 

The result isn't exactly an SSA form. The Memory SSA approach by the GCC and LLVM isn't an option since that relies on a low-level representation. That's a luxury we don't have, we need to stay close to the original source code. Regular SSA is of course an option since PyPy does it @pypy, but is limited to intraprocedural analysis. 

Fosite generates _use-def_ information instead. 
A _use_ refers to any data dependency, such as resolving a variable name or retrieving an element from a collection. 
A _def_ is the inverse such as assigning something to a variable name or inserting an element into a collection. 
Within Fosite, a _use_ and a _def_ are respectively called a dependency and a change. 
As with SSA, incremental numbering can be applied to subsequent changes.
The main difference with regular SSA is that this requires a separate data structure. 
This allows a single expression or statement to have multiple changes. 
For example, a conditional statement's dependencies and changes are the sum of its branches'. 
A conditional statement is exactly that -- a statement, and it can be useful during refactoring to treat it as a single atomic thing. 
Another useful consequence of this is that a function call can induce multiple changes, for each of its call clobbered arguments.

To achieve the same level of precision as Memory SSA, Fosite uses two kinds of changes and dependencies. For starters, there's the usual identifier dependency, which is useful for to model reachability of data. Consider code sample \ref{smp:dep} in which `x` gets defined twice. In between assignments the first value of `x` gets used to call in a call to `print`. This gets modeled using an identifier dependency. Another sort of dependency is the object dependency, which is useful to model an internal state dependency. The second assignment to `x` assigns a list to it, which gets printed as well. Before printing however an element gets appended to it. Appending an element to a list doesn't change anything about the identifier and thus can't be modeled in the same way. In other words, the final call to `print` has a dependency to both the `x` identifier and to whichever object `x` points to at the same time -- and both dependencies serve their own purpose. 

\begin{code}
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=python,
  minted style=autumn,
  minted options={},
  colback=bg }
x = int(input())
y = int(input())
print(x - y)

x = []
x.append(1)
print(x)
\end{tcblisting}
\caption{Dependency Example} \label{smp:dep}
\end{code}

There are hidden dependencies in this program as well. The first two lines of code will read two lines from `stdin` and parse them to integers. The order in which this happens is important since the two values get subtracted from each other. This can be modeled by adding implicit state. The call to `input` both depends on the implicit state and changes it, which will ensure that the relative order between the two calls remains the same. Other IO functionality such as the `print` call will do the same. Implicit state can easily be modeled by designating a specific  and hardcoded object to be the internal state. 

## Modules

Since languages tend to share a lot of features, it's not unthinkable to have an abstract interpreter that can process multiple languages. The first thing this would need is a common input format. Fosite defines a _General Abstract Syntax Tree_ (GAST), which is like other AST structures with a few exceptions. First of all, it has to be able to capture all _syntactic_ features of the languages it supports; the semantics aren't important yet. There's a good chance that adding support for an additional language will require some new nodes or some extra information in existing nodes. Since only things will have to get added interpreting the existing languages can just ignore the additional node types. Another thing that's special about the GAST is that every node has its unique identifier, and the identifiers are totally ordered. If one node's identifier is less than another's, that must mean it came before that other node in the original source file. This is important to accurately report warning, but also because some optimizations rely on it. 

The GAST only provides a common syntactical framework to work on, the interpreter has to be able to add semantics to this. A different `Executor` instance may be assigned for every supported node for every supported language. Languages that share the same features can reuse existing `Executor` implementations. Common and fundamental features such as function scoping are even available inside the interpreter's implementation itself. 

## Power of Interpretation

While linters recognize error prone patterns, interpreters can recognize error prone logic as well as some outright errors. An additional benefit of an interpreter-based approach is that it approaches feedback the same way a person would: starting at the beginning, step by step. This section gives a few interesting examples of what an interpreter can do that linters (or at least PyLint) can't. 

\begin{code}
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=python,
  minted style=autumn,
  minted options={},
  colback=bg }
def foo():
    for ...:
        for ...:
            if ...:
                return ...
            elif ...:
                return ...
\end{tcblisting}
\caption{Function Returns} \label{smp:return}
\end{code}

Code sample \ref{smp:return} contains an error prone pattern of Python-like code. A student had written something like this which resulted in one out of 200 unit tests failing. Written like this, it's possible that none of the intended `return` statements get executed. If this happens the return value is going to be `None`, which makes the unit test fail in an unexpected way -- nowhere did they specify a `None` value should be returned. Fosite gives an accurate description of the cause -- a `return` statement was missing -- instead of just the result. 

\begin{code}
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=python,
  minted style=autumn,
  minted options={},
  colback=bg }
x = []
...
while tuple([0] * i) not in x:
    ...
    x += tuple([0] * i)
\end{tcblisting}
\caption{Heterogeneous Collections} \label{smp:nohomo}
\end{code}

One of the exercises required that a sequence of tuples should be generated, stopping whenever a tuple of zeroes has been added. Code sample \ref{smp:nohomo} is based on one of the submissions. Up until the adding of the tuple of zeroes, the type of `x` had been `List[Tuple[int]]` (in the notation used by Python 3.5 type hints). Instead of appending the tuple however, `+=` will concatenate the tuple's elements to `x`. This changes the type to `List[Union[int, Tuple[int]]]`. This transition to a heterogeneous collection is valid Python code but ultimately very error prone. In fact, this causes an infinite loop in this case, as the expected element never gets added.

\begin{code}
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=python,
  minted style=autumn,
  minted options={},
  colback=bg }
def change(x, d = None):
  list1 = ''
  list2 = []
  for i in range(0, len(x)):
    while x[i] != ' ':
      list2 += x[i]
    list1 += translate(list2[0], d)
    list2 = []
  return list1
\end{tcblisting}
\caption{Endless Loop} \label{smp:nostop}
\end{code}

Although deciding whether or not any given program will never stop is impossible, it is possible in some trivial cases. Those trivial cases also happen to be quite common. Code sample \ref{smp:nostop} is an excerpt from a submission. The student intended to tokenize the string `x`, building the token in `list2`. Every token should then get translated and the translated token gets stored in `list1`. There are a number of mistakes but the most important one is arguably the endless `while` loop. The student wanted index `i` to be a starting position of the token, with the `while` loop building the token from that point. That's of course not what the code does, the same character will get added over and over since none of the values in the loop condition ever change. Data-flow analysis remembers when and where variables get their values, so it can be used to recognize that the variables are still the same. This approach will also flag any `while True` loop as error prone, which is arguably a good thing. 

# Implementation

Fosite is an abstract interpreter. It uses abstract pointers, which can be used to fetch abstract objects from an abstract memory. The objects themselves have no value whatsoever, but they do have a notion of types, attributes, and elements. There is a notion of namespaces where names get mapped to objects, and a notion of ordered heterogeneous collections. In essence, the interpreter tries to get as close at it can to actual interpreter without having actual values. This not as obvious as it sounds. For example, it's tempting to cut corners when implementing the _Method Resolution Order_ (MRO), variable capturing in closure definitions, or the explicit `self` argument in Python. Simple approximations of these behaviors would suffice for well written code -- but targeting such code makes no sense for a static analyzer. We have to be able to analyze _really_ bad code as well. 

The goal of the interpreter is to perform a data-flow analysis. Every evaluated node returns a list of changes and dependencies. The error prone patterns that the interpreter finds are welcome side-effect.

## Objects 

Everything is an object in Python, even classes which become class objects upon definition. An object of a class has a reference to its class object. Among other things, this reference gets used during name resolution. Every class can also extend multiple other classes, called base classes, in a similar way. This can easily be modeled in an abstract interpreter using a list of pointers.

The type of an object is harder to model however. In most object-oriented languages, an object's class is its type. Since Python has multiple inheritance, and classes are objects as well, there's more to it than that. 

\begin{code}
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=pycon,
  minted style=autumn,
  minted options={},
  colback=bg }
>>> x = 42
>>> t1 = type(x)
>>> t1
<class 'int'>
>>> t2 = type(t1)
>>> t2
<class 'type'>
>>> t3 = type(t2)
>>> t3
<class 'type'>
\end{tcblisting}
\caption{Types in Python} \label{smp:types}
\end{code}

Code sample \ref{smp:types} illustrates the oddity in this. The `type` function returns a class object, so that `type(42)` returns the class objects of name `int`. Using the same function to get the class object's type returns a class object of name `type`. Requesting that object's type reveals something strange -- `type` is its own type. This seemingly cyclic dependency gets implemented in CPython using a type flag, if that flag is set it'll return the type class object when needed. In other words, the `type` object doesn't have a reference to itself, it'll get its own reference at runtime when needed. 

In other words, the type of a value is the same as its class object. A class's basetypes have nothing to do with its type -- they always have type `type`. These semantics are quite straight forward to model in an abstract interpreter: the list of base class references are still there, but there's also a type flag. When that flag is set, the `type` function shouldn't use the base classes but fetch the pointer to the `type` class object.
 
Besides types and base classes, the Fosite interpreter also keeps track of attributes and elements. Attributes can reuse the namespace logic that's implemented for scoping. Elements are a lot harder to model and will be covered by a later section.

## Paths and Mappings

In order to report the cause of an error accurately, we need to know the source of every value. A path should correspond to a sequence of instructions so that the user gets an idea of the execution path that lead to a problem. Every entry in the path gets called a path node. Examples of path nodes include which branch of a conditional was followed, assignments, and function calls. 

Every AST node has a unique identifier with a total ordering we can use. A first attempt at defining the path node would be to just reuse the AST identifiers. This works fine until user-defined functions come into the picture. A function call will come after the function definition, and its identifier will be larger than any of the function definition's nodes. 
This would place the function body execution before the function call itself. 
On top of that, this system does not support executing the same node more than once. 
A better solution is to define a path node to be an ordered collection of AST nodes -- the nodes that are currently being executed. 
Some nodes need some extra information, a branch node needs to contain which branch was actually taken. 
Each branch is incrementally numbered, and also contains the total number of branches for practical reasons (see method calls and namespace frames).
The actual branch numbers are of no concern, their main purpose is telling possible branches apart. 
Definition \ref{def:path_node} makes this more formal, and definitions \ref{def:path_order}, \ref{def:contain}, \ref{def:complement}, \ref{def:mergeable}, and algorithm \ref{alg:complement} introduce useful properties of paths.

A mapping is simply a pair of the form `(Path, Pointer)`. Because they usually appear in multiples, they can be implemented as a list of `(Path, Pointer)` values instead. In this case, every path in a mapping must be distinct; there are no paths that are contained by another path in the same mapping. 

\begin{definition} \label{def:path_node}
A path node is of the form $((n_1, n_2, ... , n_i), b, t)$, where the elements $n_i$ are an ordered sequence of AST node identifiers, $b$ is the number of the branch that was taken, and $t$ is the total of branches that were possible at that node.
\end{definition}

\begin{definition} \label{def:path_order}
Let $p$ and $q$ be two paths with forms respectively $(n_p, b_p, t_p)$ and $(n_q, b_q, b_t)$, $p \prec q \iff n_p \prec_{lex} n_q \vee (n_p = n_q \wedge b_p \prec b_q )$. 
\end{definition}

\begin{definition} \label{def:contain}
A path $A$ is \textit{contained} in another path $B$ if every node of path $A$ occurs in path $B$ as well.
\end{definition}

\begin{definition} \label{def:complement}
The complementary nodes of a single path node $(n_p, b_p, t_p)$ are defined as $\{\, (n_p, i, t_p) \mid 0 \leq i < t \wedge i \neq b_p \,\}$. If $t_p = 1$, an assignment node for example, there are no complementary nodes.
\end{definition}

\begin{definition} \label{def:mergeable}
A path $A$ is \textit{mergeable} with another path $B$ if a $A$ does not contain a complement of one of $B$'s nodes. \end{definition}

\begin{algorithm}
    \caption{Complementary Paths}\label{alg:complement}
    \begin{algorithmic}[1]
        \Function{complement} {path}
          \State $\texttt{result} \gets [\,]$
          \State $\texttt{current} \gets [\,]$
          \ForAll{nodes in path}
            \If{ $\texttt{node.is\_branch()}$ }
              \ForAll{complements of node}
                \State $\texttt{temp} = \texttt{current.clone()}$
                \State $\texttt{temp.add\_node(complement)}$
                \State $\texttt{result} \gets \texttt{result} \cup \texttt{temp}$
              \EndFor
            \EndIf
            \State $\texttt{current.add\_node(complement)}$
          \EndFor
          \Return result
        \EndFunction
    \end{algorithmic}
\end{algorithm}

## Boolean Expressions

A boolean expression can be arbitrarily hard to evaluate. When used in a conditional statement, we can't always decide whether or not a given branch gets taken. The best we can do in these cases is conclude that the branch _might_ get taken. Evaluating any boolean expression can thus result in `True` or `False`, as well as `Maybe`. In some cases, a `Maybe` isn't satisfactory. 

\begin{code}
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=python,
  minted style=autumn,
  minted options={xleftmargin=-8pt, linenos},
  colback=bg}

if current is not None:
  print('given {}-{}'.format(current.year, current.month))
else:
  current = datetime.now()

\end{tcblisting}
\caption{Conditions} \label{smp:conds}
\end{code}

Code sample \ref{smp:conds} shows that in some cases, we really need an accurate answer. This is a pattern that commonly occurs when dealing with optional arguments or when writing library functions. The negative branch should only get executed when `current` was not None, so that an actual argument doesn't get overwritten. On the other hand, it _must_ be executed if `current` was None, so that further evaluation doesn't result in a false type error. 

The `is` operator compares the addresses of two objects and returns `True` if and only if they're equal. We can mimic this behavior -- and answer with certainty and under which conditions the two operands' point to the same location. The resulting mapping will use the merged paths of the operands to point to the `True` object. The `==` operator should be similar. Technically it depends on the implementation of the `__eq__` method, but let's assume that it has a decent implementation. In that case it should at least return `True` if both operands point to the same object -- as with `is`. A similar reasoning can be applied to the `!=`, `<=`, and `>=` operators. 

We can also handle the `and`, `or`, and `not` operators in a similar way. If both operands already point to `True` we can merge the paths and return a mapping that points to `True` as well. The other two operators are analogous.

We combine the paths of both operands to get a new mapping. This means means that we must only consider path pairs that are mergeable, if not those operand pairs cannot actually exist at runtime. Failure to meet this requirement will lead to false positives very quickly.

## Conditionals

When executing an execution branch, we can have information about why that specific branch is being executed. If that information includes for example that we are sure that `x` is not `None`, we should disregard any mapping that says otherwise. Even better, we can exclude any mapping that would occur under the same contradictory conditions -- even if those mappings don't have an explicit connection to `x`. 

\begin{code}
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=python,
  minted style=autumn,
  minted options={xleftmargin=-8pt, linenos},
  colback=bg}
if cond1: # Condition 1
  y = None
  z = None
  
if y is not None: # Condition 2
  print(z.attribute)

\end{tcblisting}
\caption{Conditions} \label{smp:exclude}
\end{code}

In code sample \ref{smp:exclude}, there's an implicit relation between condition 1 and condition 2. Going back to the previous section, the result of the test of condition 2 will contain a mapping $(p, x)$, where $p$ contains a node indicating that the positive branch of condition 1 was taken, and where $x$ is a pointer value to `False`. This means that any mapping containing $p$ during the execution of the positive branch of condition 2 cannot occur during actual execution. We will call concept \textit{path exclusion}, and paths such as $p$ are called _restricted paths_. Observation \ref{obs:exclude} summarizes this more formally. 

\begin{observation}
\label{obs:exclude}
Assume that resolving an identifier $x$ results in a set of mappings $M$. Every mapping $m \in M$ is of the form $(p, a)$, where $a$ is a pointer value, and $p$ is the execution path that mapped $x$ to $a$. 

Let $R$ be the set of restricted paths. Given a mapping $(p_m, a) \in M$, if there exists a path $p_r$ in $R$ for which holds that $p_m$ contains $p_r$ (definition \ref{def:contain}), we can exclude the mapping from the current evaluation. 
\end{observation}

## Namespace

Namespaces are the most essential component of Fosite. We set out to give the most descriptive and helpful messages we can, by describing the conditions in which something occurs. Paths are a first step towards describing these conditions, but namespaces are where they're stored. There are a few layers of abstraction required to make this manageable, this section will introduce them incrementally.

### OptionalMapping

Mappings have already been introduced, but they contain a pointer value which is not enough to indicate an uninitialized variable. A different structure is used to this end, where the pointer value is optional. An `OptionalMapping` with a missing pointer value indicates an uninitialized variable and also describes why the variable is uninitialized.

### Branch

The `Branch` struct is the first layer of the namespaces and is where names are added, and its internal structure is of the form `HashMap<String, OptionalMapping>`. Every branching point during execution can induce several new branches in a namespace which have to be separated during execution. For example, the negative and the positive branch of a conditional statement should not influence each other. 

### StatisChamber

If we encounter a `break` statement while evaluating a loop body, the evaluation of the current execution path terminates. The changes made until that point still have to be saved though, as they'll become visible again after the loop has been evaluated. Function calls require the same to handle different return points. A `StatisChamber` contains a `HashMap<Path, Branch>`. The key is needed because the control flow can be broken at multiple points, and we have to keep them all separate.

### SubFrame

For every branching point, we'll use a `Branch` and two `StatisChamber`s -- one for loops, one for function calls. These get stored in a `SubFrame` struct.

### Frame

This is the first namespace component that contains some actual logic. Every branching point leads to the creation of a new `Frame`. This structure contains a _cause_, the path node where the branching happened. It also contains a list containing subframes and its length depends on the number of possible branches at the cause node. The cause node has the form $(n, b, t)$ where $t$ is the number of possible branches at that node, so the number of subframes course equal to $t$. There is only one subframe active at any point during execution and its index is stored. Algorithm \ref{alg:setframe} describes how a mapping gets set into a frame, the `insert` method of a `StatisChamber` will simply insert into each of its branches. 

\begin{algorithm}
    \caption{Set Mapping}\label{alg:setframe}
    \begin{algorithmic}[1]
        \Function{set\_mapping} {name, mapping}
          \If{ $\texttt{self.contains(name)}$ }
            \State $\texttt{old\_mapping} \gets \texttt{self.resolve(name)}$
          \Else
            \State $\texttt{old\_mapping} \gets \texttt{OptionalMapping}::new(\texttt{Path}::empty(), None)$
          \EndIf
        
          \State $\texttt{self.current\_branch.loop\_statis.insert(name, old\_mapping)}$
          \State $\texttt{self.current\_branch.function\_statis.insert(name, old\_mapping)}$
          \State $\texttt{self.current\_branch.branch.insert(name, mapping)}$
        \EndFunction
    \end{algorithmic}
\end{algorithm}

Code sample \ref{smp:statis} illustrates how these are used. The `break` statement will freeze the contents of its current place and place it the loop `StatisChamber` of the current subframe. The current branch is empty however -- no changes were made since line 3. At the end of the execution of line 3, the contents of the current frame get lifted to the previous one, this will be explained in the next section. Before assigning to `y` on line 4, `y` gets resolved first, there is no mapping for `y` yet so a new mapping indicating an unitialized value gets created instead. This result is then stored into each currently active statis chamber. Resolving `y` on line 6 simply return the result of the assignment on line 5 -- the statis chamber isn't used here.

The results of the loop static chamber get inserted into the regular active branch after executing the loop body on lines 2 - 6.  Trying to resolve `y` on line 8 will result in an error: `y` is still uniniatilized if the condition on line 3 was true. The statis chamber in this case was responsible for storing an unitialized value, as this is different from an non-existing mapping.

\begin{code}
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=python,
  minted style=autumn,
  minted options={xleftmargin=-8pt, linenos},
  colback=bg}
while True:
    z = x.foo()
    if x.test():
        break
    y = z.bar()
    print(y)

print('final:', y)
\end{tcblisting}
\caption{Statis Example} \label{smp:statis}
\end{code}

### Namespace

A namespace's data is simply a list of frames, one for each branch point that's being executed. Looking up an identifier is quite simple: look for the most recent frame that contains a mapping for that name. Name resolution is so simple because the other operations do all the heavy lifting. There are three other operations:

  * \textit{Grow}: Uses a path to create new frames until there's a frame for each node in the path
  * \textit{Insert}: Grows the namespace when needed, and then inserts a mapping into the last frame. 
  * \textit{Merge}: Merges the last frame's content in the next to last frame.

For the sake of data sparsity, growing is done upon insertion -- and not upon the actual branching. Bearing in mind that every object has a namespace as well, we don't want to grow each of their namespaces every time -- its namespaces probably won't even change as most objects are immutable literals. 

#### Grow

If the current namespace has $n$ frames, and the given path has $m$ nodes, we must add $m-n$ frames -- corresponding to the last $m-n$ path nodes. The correctness of this approach relies on a bit of inductive reasoning. 

The active execution path will always be at least as long as the number of frames of a namespace. All the namespaces that have changed have the same number of frames. If a change has been made in the current branch, growing has added frames until there are as many frames as there are nodes in the execution path. If a change was made in some branch that's already been executed, and is thus no longer part of the execution path, merge operations will have reduced the number of frames until the length is equal to the length of execution path. The namespaces that have not changed have strictly less frames, corresponding to the length of the execution path of their last change.

The cause nodes of the frames form a prefix of the active execution path. This more of a feature of the language than of the interpreter. Since Python does not have a `goto` statement, there's a fixed structure to the branching points.

#### Merge

The merge operation combines the results of the last frame, removes it, and puts the merged result into the last frame. An argument determines the destination of every subframe's content -- either into the regular branch or into a statis chamber. The method is specific to function scoping, but block scoping can be done in a similar way.

\begin{code}
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=python,
  minted style=autumn,
  minted options={xleftmargin=-8pt, linenos},
  colback=bg }
x = 42
y = 'string'

if cond:
  x = '42'

x + y 
\end{tcblisting}
\caption{Merging}\label{smp:merge}
\end{code}

The name resolution stops at the most recent frame that contains that name. Code sample  \ref{smp:merge} illustrates that this isn't just an easy solution -- it's also a useful one. If the negative branch of the condition at line 4 is taken, execution will fail at line 7. Variable `x` still has the value it received at line 1, but only reporting this paints an incomplete picture. It only has that value if the negative branch was taken. So if we want to accurately describe why it has that value, that information should be there as well. To this end, merging will resolve relevant identifiers and add a node about the branch to every mapping's path. 

\begin{algorithm}
    \caption{Merge}\label{alg:merge}
    \begin{algorithmic}[1]
        \Function{merge} {}
          \State $\texttt{names} \gets [\,]$

          \ForAll{ subframes \textbf{in} frames.last()}
            \State $\texttt{names} \gets \texttt{names} \cup \texttt{subframe.names}$
          \EndFor

          \State $\texttt{branch\_content} \gets \texttt{Branch}::new()$

          \State $\texttt{loop\_statis} \gets \texttt{StatisChamber}::new()$

          \State $\texttt{function\_statis} \gets \texttt{StatisChamber}::new()$

          \State $\texttt{cause} \gets \texttt{frames.last().cause}$

          \ForAll{ (i, subframes) \textbf{in} frames.pop().enumerate()}
            \State $\texttt{new\_node} \gets \texttt{PathNode}::new(\texttt{cause, i})$
            \State $\texttt{subframe.augment\_statis\_chambers(new\_node)}$
            \State $\texttt{loop\_statis.insert\_all(subframe.loop\_statis)}$
            \State $\texttt{function\_statis.insert\_all(subframe.function\_statis)}$
            \ForAll{name}
              \State $\texttt{mapping} \gets \texttt{subframe.resolve(name)}$
              \State $\texttt{mapping.augment(new\_node)}$

              \If { \texttt{break\_loop(subframe)}}
                  \State $\texttt{loop\_statis.add(mapping)}$
              \Else 
                \If { \texttt{return\_function(subframe)}}
                    \State $\texttt{function\_statis.add(mapping)}$
                \Else 
                    \State $\texttt{branch\_content.add(mapping)}$
                \EndIf
              \EndIf
            \EndFor
          \EndFor

          \State $\texttt{frames.last().insert\_all(branch\_content)}$
          \State $\texttt{frames.last().insert\_all\_loop\_statis(loop\_statis)}$
          \State $\texttt{frames.last().insert\_all\_function\_statis(function\_statis)}$
        \EndFunction
    \end{algorithmic}
\end{algorithm}

There are a few things of note here. The first step is collecting the names of all identifiers that have changed in this branch. A new path node is created for every subframe that will disappear, based on the frame's path node, and the number of the subframe. The number is necessary because the cause node's branch number is uninitialized until this point. This node is then used to _augment_ all the paths in the statis chambers. Remember that the statis chambers were of the form `HashMap<Path, Branch>`, where the `Path` was supposed to keep the different sources of broken control flow apart. The `augment` method will add a new node to every key path -- future additional key paths won't contain this node, which is how they keep the different points of broken control flow apart. 

Code sample \ref{smp:merge} showed us some behavior we want during name resolution. We can achieve this by incorporating name resolution in the merge operation. Every identifier that has been changed in any branch will get resolved in every branch, augmented, and stored in the next to last frame. This will ensure that all the relevant mappings for any given identifier can always be found in a single frame. More importantly, the `augment` method will update path mappings to reflect when an identifier didn't change in some particular branch.

This is just the behavior for merging a frame because of a conditional statement. Function calls and loops will also introduce a new namespace frame. Merging a loop won't place things into the loop statis chamber, merging a function won't place anything in any statis chamber. Instead merging those should put the contents of one or both statis chambers into the regular branch. A statis chamber is of the form `HashMap<Path, Branch>` -- every mapping of every branch gets updated with the key path before being added to the regular branch. 

## Name Resolution

Namespaces by themselves aren't enough to implement all the name resolution behavior, name resolution can use several different namespaces. It's possible that an identifier is only sometimes defined in a namespace, for the other cases resolution will have to continue to some other namespace. The unresolved paths will be carried over into the resolution in the next namespace. All returned mappings will have its paths merged with the unresolved paths to reflect the fact that name resolution continued in the next namespace. This is probably unwanted behavior as well -- it's quite hard to use a variable that only exists sometimes -- so it's something our interpreter should warn about as part of its analysis. Algorithm \ref{alg:chain} shows the logic behind the implementations, the `next_namespace` function will depend on the kind of name being resolved.

\begin{algorithm}
    \caption{Resolve}\label{alg:chain}
    \begin{algorithmic}[1]
        \Function{resolve} {name}
          \State $\texttt{result} \gets \texttt{Mapping}::new()$

          \State $\texttt{unresolved} \gets [\, \texttt{Path}::empty()\,]$

          \While{ unresolved.len() > 0 }
              \State $\texttt{new\_unresolved} \gets [\,]$

              \State $\texttt{namespace} \gets \texttt{next\_namespace}$
              \State $\texttt{mapping} \gets \texttt{namespace.resolve(name)}$

              \ForAll{$(p, x)$ \textbf{in} mapping}
                \ForAll{unresolved\_path \textbf{in} unresolved}
                    \State $\texttt{new\_path} \gets \texttt{p.merge(unresolved\_path)}$

                    \If {\texttt{x.is\_none()}}
                      \State $\texttt{new\_unresolved} \gets \texttt{new\_unresolved} \cup \texttt{new\_path}$
                    \Else 
                      \State $\texttt{result.add(new\_path, x)}$
                    \EndIf
                \EndFor
              \EndFor

              \State $\texttt{unresolved} \gets \texttt{new\_unresolved}$

          \EndWhile

          \State \Return \texttt{result}
        \EndFunction
    \end{algorithmic}
\end{algorithm}

### Identifiers

Variables are stored in up to four different namespaces:

  * The local scope, this is the function scope and every function call will create a new one. All variables made during a function call will live here, and stop existing when the function call is done.
  * The enclosing scope, which is most commonly used to capture variables in lambda definitions. Variables that occur in a function definition and that are already in scope during the definition will get "saved" to the enclosing scope of the function. 
  * The global scope, where most class and function definitions will end up going, and where students first write their first programs in. 
  * The builtin scope, which is not to be confused with the standard libraries, contains essential Python features such as the `int` type and the `sorted` function. 

At any point during execution there will be either two or four active scopes, depending on whether or not a function is being executed. Every function call will create a new empty local scope, but reuse the existing enclosing scope asociated with its definition. 

### Attributes

An object's attributes don't neccesarily exist in its own namespace. A lot of them, especially its methods, will exist in the namespace of its class object, or maybe even in one of its base classes. Python uses its own _Method Resolution Order_ (MRO) to define the order of the base classes in case of multiple inheritance. The first base class in the definition will get used first, where MRO can be applied again, ... This means that the second base class of the definition will only get used if the first base class did not contain an attribute of that name, and neither did any of its extended base classes. 

### Methods

Functions are just callable objects in Python, and a method seems to be a function that's part of an object's namespace. There is one obvious difference however, the explicit self of a method definition isn't part of the method call. This is because Python hides an implicit creation of a `method` object, which isn't just a callable object. In fact, it encapsulates a callable object -- the callable attribute. It also contains a reference to object it's being called on however. The method object itself is also callable, and calling it will call the embedded function with the embedded parent object prepended to the arguments. 

## Collections

Collections are a hard component to implement, especially for dynamic programming languages. When an element of a collection is used in an operation, we have to know which element that was to interpret the result. Static programming languages can just return a value of the defined element type, but dynamic programming languages typically have heterogeneous collections. Features like multiple return values rely in this feature. These are actually tuples, which can be indexed or unpacked just like any other collection. Remembering length and the order of the elements of this tuple is paramount to avoiding false positives. 

The solution is similar to namespaces, but with some additional abstractions to reflect the additional uncertainty. We don't have to store every element explicitly. If a list contains only objects of type `A`, it doesn't really matter which objects those are. Any time an element of the collection gets used, it will be in terms of an abstract, general, instance of `A`. Non-empty collection literals such as multiple return values are exceptions, in this case every element does get stored explicitly. This general instance is called the representant. The most important things to know of a representant are its type, and how many elements of the collection it represents. Chaining representants together can describe both ordered and unordered collections. Unordered collections do have an order after all, just not a reliable which programmers should rely on. 

### Components

  * The core components of a collection are the representants, which are basically an alias for an object pointer. 
  * Chunks represent contiguous regions within a collection. Their size is defined as a range whose bounds may be empty. Iteratively adding an element to a collection will create a chunk with no upper bound for example, unless we knew with certainty how many iterations took place. Chunks also contain mappings to its possible representants. If `x` is either an `int` or a `list` for example, adding it to a collection will add a chunk with two representant mappings. 
  * A sequence of chunks forms a chain. A notion of length is added here as well, which isn't just the sum of the ranges of its chunks, insertion will invalidate the relationship between the chunk length and the chunk lengths. Most of the collection logic is implemented here, such as indexing and slicing.
  * The next layer are branches, which are similar to the ones that are part of namespaces. A branch in the context of a namespace get initialized as an empty `HashMap<String, Mapping>`, and only stores changes. This sparse structure is possible because with namespaces you know exactly which elements have been changed -- they have a unique name. Collections don't have this luxury. Unless we absolutely know which element was changed, we have to assume the entire structure has been changed. This is why a collection's `Branch` contains a list of `(Path, Chain)` tuples, where every `Chain` is an entire "collection" in its own right.
  * A collection also has frames, which are similar to the ones in namespaces as well. The main difference is that collections currently don't have a notion of statis chambers. These can be added to improve accuracy in some cases, but didn't seem very important yet. A `Frame` contains a list of branches, the path node that caused the branching, and the index of the active branch. 
  * Collections are stacks of frames, just like namespaces. They also have a grow and a merge operation, but the implementations are slightly different. Growing still creates a new frame for every branch point, but it will copy the current branch's content into every branch of the new frame. Merging is currently done naively. The last frame gets popped, the paths of its content get augmented with the frame's cause node and then replace the contents of the new active branch. A different approach could try to merge branches to the chunk level so avoid unnecessary duplication, which is a considerable challenge to implement. This might be necessary for large and complex files, but the current solution seems to be suffice for now.

### Operations

There are a few different ways a new element can get added to a collection. The most fundamental way is simply by definition, as a list of mappings. In this case, a single chunk gets added for each mapping -- every given element is its own representant in this case for optimal accuracy, and every chunk has a known size of exactly 1. Inserting an element is the most complex operation. Any chunk that has only a single representant of the same type as the new element can have its upper bound incremented. All other chunks have it minimum size decremented -- to reflect that one of its elements may have been replaced. A new chunk of size $[\,0, 1\,]$ gets wedged in between chunks that haven't had their upper bounds incremented. The branch itself does not have its length incremented. The most common way to add something to a `list` in Python is through the `append` method however, which is a much easier case. We can repeat the previous process, but only applied to the last chunk. We can either increment the upper bound of the existing chunk, or add a new chunk of size exactly 1. 

\begin{algorithm}
    \caption{Linearize Collection}\label{alg:lin}
    \begin{algorithmic}[1]
      \Function{linearize} {n, chunks}
        \If {chunks.is\_empty()}
            \State \Return $[\,]$
        \EndIf
        \State $\texttt{chunk} \gets \texttt{chunks[0]}$
        \State $\texttt{result} \gets [\,]$
        \State
        \State \textbf{outer:} 
        \For {\texttt{(path, repr)} \textbf{in} \texttt{chunk}}

          \State $\texttt{acc} \gets [\,]$

          \For{\texttt{\_} \textbf{in} \texttt{0\,..\,chunk.min}}
            \State $\texttt{acc.append(Mapping(path, repr))}$

            \If {\texttt{acc.len() >= n}}
              \State $\texttt{result.append(acc)}$
              \State \textbf{continue outer}
            \EndIf
          \EndFor

          \State

          \State $\texttt{intermediate} \gets \texttt{linearize(n - acc.len(), chunks[1..])}$

          \For {\texttt{sequence} \textbf{in} \texttt{intermediate}}
            \State $\texttt{result.append(acc ++ intermediate)}$
          \EndFor 

          \State

          \For{\texttt{\_} \textbf{in} \texttt{chunk.min\,..\,chunk.max}}
            \State $\texttt{acc.append($\texttt{Mapping}::new$(path, repr))}$

            \If {\texttt{acc.len() >= n}}
              \State $\texttt{result.append(acc)}$
              \State \textbf{continue outer}
            \EndIf

            \State

            \State $\texttt{intermediate} \gets \texttt{linearize(n - acc.len(), chunks[1..])}$

            \For {\texttt{sequence} \textbf{in} \texttt{intermediate}}
              \State $\texttt{result.append(acc ++ intermediate)}$
            \EndFor 
          \EndFor


        \EndFor

        \State \Return \texttt{result}
        
      \EndFunction
    \end{algorithmic}
\end{algorithm}

There are a few different ways to retrieve an element from a collection. If done through a dynamically calculated index, a mapping for every representant is returned. We can do better for static indices however. The collections contents get linearized first -- replacing the chain of chunks with a list of mappings, this process is described in algorithm \ref{alg:lin}. This process reveals which possible elements there can be at that position. This is a powerful feature for several reasons. The first (or last) element of a collection can be given some special meaning, even if it's generally a bad idea, in which case retrieving the correct element is a good thing. More importanely, students often don't realize Python has multiple return values, and instead return a tuple which they then index explicitely. A static analyzer might want to recommend using multiple return values instead, but it will only be able to do so if its analysis didn't stumble over the indices. The linearized representation can also be used to create slices of collections, which is a very _pythonic_ operation so being able to interpret it accurately is important. 

All indexing operations may return duplicates, because the merge operation isn't very thorough at this point. This can be alleviated by only returning a single mapping for every unique representant. The different ways an element may have been added to a collection are irrelevant, all that matters is that it has been added, and showing a single execution path should be enough to get the point across. 

## Function Definitions

A function definition creates a callable object and assigns the result to the given variable name. The object contains a closure, which in turn contains the function body as a block of AST nodes and the required argument definitions. Upon calling the closure the defined arguments and the given mappings get mapped to each other, as shown in the next section. Once the arguments are in place it executes the function body. Return values get treated as assignments to an `___result` identifier, so the existing namespace logic can be reused. 

Python's own functions and modules are harder to implement. Depending on the interpreter used they may not even be in Python, but in C for example. Although the Fosite interpreter is designed to accomodate multiple languages, it's made with dynamic languages in mind -- C is quite far out of scope. A solution is to implement function behavior in closures, which then get injected into the interpreter as callable objects. Unlike the user defined functions these internal functions don't contain a function body AST, but manipulate the interpreter state directly. 
This is a laborsome endeavor, but it does have a few upsides. Builtin functions such as `sorted` contain hundreds of lines of code -- and none of them are relevant to our analysis. Including implementation details in the paths can only confuse users, as these usually have no knowledge of the language internals. Giving a summarized, hard coded, result is both more efficient and more useful. 
Modules are implemented as closures that can inject callable objects into the interpreter at runtime. This means that with some time and dedication, third party libraries can be easily added in the same way. 

## Function Calls

A few things have to be evaluated before a function call, the call target has to be evaluated first, then the arguments get evaluated from left to right. Evaluating the target will return a mapping which can contain a variable amount of pointers. Well written code can make extensive use of function objects to reduce code duplication for example. Target evaluation can result in several different function objects, and we have to consider every possible case. These all have to be evaluated independently, which is why a frame can have a variable amount of subframes -- and why path nodes contain information about how many branches there are. 

\begin{algorithm}
    \caption{Assign Arguments}\label{alg:arg}
    \begin{algorithmic}[1]
        \Function{assign\_pos} {gpos, gkw, arg, kwonly, vararg, kwarg}
          \If {\texttt{gpos.len() > 0 \&\& arg.len() > 0}}
            \State $\texttt{assign(arg[0].name, gpos[0])}$
            \State $\texttt{assign\_pos(gpos[1..], gkw, arg[1..], kwonly, vararg, kwarg)}$
          \Else
            \State $\texttt{assign\_vararg(gpos, gkw, arg, kwonly, vararg, kwarg)}$
          \EndIf
        \EndFunction

        \Function{assign\_vararg} {gpos, gkw, arg, kwonly, vararg, kwarg}
          \If {\texttt{vararg.is\_some()}}
            \State $\texttt{arg\_list} \gets \texttt{abstract\_list(gpos)}$
            \State $\texttt{assign(vararg.name, arg\_list)}$
          \EndIf
          \State $\texttt{assign\_kw(gkw, arg ++ kwonly, kwarg)}$
        \EndFunction

        \Function{assign\_kw} {gkw, arg, kwarg}
          \If {\texttt{gkw.len() > 0 \&\& arg.len() > 0}}
            \State $\texttt{name} \gets (\texttt{arg.names} \cap \texttt{gkw.names})\texttt{.pick\_one()}$
            \State $\texttt{assign(name, gkw[name])}$
            \State $\texttt{assign\_kw(gkw $\setminus$ name, arg $\setminus$ name, kwarg)}$
          \Else
            \State $\texttt{assign\_kwarg(gkw, arg, kwarg)}$
          \EndIf
        \EndFunction

        \Function{assign\_kwarg} {gkw, arg, kwarg}
          \If {\texttt{kwarg.is\_some() }}
            \State $\texttt{arg\_dict} \gets \texttt{abstract\_dict(gkw)}$
            \State $\texttt{assign(kwarg.name, arg\_dict)}$
          \EndIf

          \State $\texttt{assign\_arg\_default(gkw, arg, kwarg)}$
        \EndFunction

        \Function{assign\_arg\_default} {arg}
          \If {\texttt{arg.len() > 0 \&\& arg[0].has\_default()}}
            \State $\texttt{assign(arg[0].name, arg[0].default)}$
            \State $\texttt{assign\_arg\_default(arg[1..])}$
          \EndIf
        \EndFunction
    \end{algorithmic}
\end{algorithm}

The call arguments get stored in two collections of mappings: a list for the positional arguments, and a map for the keyword arguments. All these arguments get augmented with a path node of the current function call. These then get mapped to arguments in the definition. Python has four  kinds of arguments in a function definition: `args`, `kwonlyargs`, `vararg`, and `kwarg`. Both the `args` and `kwonlyargs` can be given default values, and all arguments after the `varargs` will be placed in `kwonlyargs`. The underlying semantics are less trivial than one might expect, the principle is illustrated in algorithm \ref{alg:arg}. The algorithm assumes every argument in only provided once -- the Python runtime already gives a detailed error when this isn't the case. 


\begin{algorithm}
    \caption{Function Calls}\label{alg:call}
    \begin{algorithmic}[1]
        \Function{call} {target, args, kwargs}
          \State $\texttt{b} \gets \texttt{scopes.len() > 2}$

          \If {\texttt{b}}
            \State $\texttt{shadow\_scopes.push(scopes.pop())}$

            \State $\texttt{shadow\_scopes.push(scopes.pop())}$
          \EndIf

          \State

          \State $\texttt{enclosing} \gets \texttt{get\_closure(target)}$

          \State $\texttt{scopes.push(enclosing)}$

          \State $\texttt{scopes.push(Scope}::new())$

          \State

          \State $\texttt{fun} \gets \texttt{get\_callable(target)}$

          \State $\texttt{fun(args, kwargs)}$

          \State

          \State $\texttt{result} \gets \texttt{scopes.pop().extract\_result()}$

          \State $\texttt{scopes.pop()}$

          \If {\texttt{b}}
            \State $\texttt{scopes.push(shadow\_scopes.pop())}$

            \State $\texttt{scopes.push(shadow\_scopes.pop())}$
          \EndIf

          \State \Return \texttt{result}

        \EndFunction
    \end{algorithmic}
\end{algorithm}

The calling mechanism is best explained in pseudocode as well, as in algorithm \ref{alg:call}. Lines 2 to 5 move existing enclosing and local scopes to a different stack. This makes name resolution easier, there are always at most four active scopes. Lines 7-9 retrieve the the existing enclosing scope, a local scope is created, and both get placed on the stack of active scopes. Lines 11 and 12 contain the function call. A callable object in the interpreter gets retrieved, and executed using the provided arguments. The callable will do the argument assignments as in algorithm \ref{alg:arg} before executing whatever function logic it contains. Line 14 pops the local scope, and retrieves the entry that holds the return value. Lines 15-18 restore the scopes to the state before the function call. 
Algorithm \ref{alg:call} does not contain some necessary bookkeeping operations. The local scope gets discarded after handling each call target, but the namespaces of the changed objects have to be merged after handling all call targets, and the resulting mapping needs to have its paths augmented with the path to get to the target.

Interpreting recursive functions requires a way to _tie the knot_ -- so that analysis doesn't get stuck in an endless loop. The interpreter maintains a call stack, and only permits the same function to be called a set amount of times. A branch that would perform a recursive call that exceeds the recursion depth is simply terminated, so that only the branches that result in a base case get executed at this depth. A call at a higher recursion depth will then use this value in its analysis of the entire function body. This method's accuracy is sufficient for most use cases, even for low recursion depth limits @interpret. 

## Warnings and Errors

The interpreter itself can do some useful static analysis itself. If it encounters it cannot handle, the actual runtimes probably won't be able to either. In this case the interpreter will emit an error, but with more information about which execution paths lead to that error.

It might also encounter error prone patterns during execution. If execution does fail, the warnings's information will add to the errors's explanation to paint an even more accurate picture. Even if execution doesn't fail, some of the warnings might indicate a deeper logical flaw. 

### Errors

Python doesn't always validate the input to the builtin functions. The `sum` function will just error with `TypeError: unsupported operand type(s) for +` if the argument wasn't an iterable containing things that can be summed together. We can be more helpful in our errors by describing which elements can't be added together and how they got there, perhaps even hinting towards implement `__add__` and `__radd__`. Similar errors can be emitted when trying to incorrectly apply an operator objects, such as adding an `int` and a `str` or trying to insert an element into a `str`. 

Uninitialized variables are some of the most common bugs in many programming languages. Badly written code can make these hard to track down. The way our interpreter is structured gives us all the needed information to provide helpful errors. 

Every branch of a collection knows how large it can be. Indexing using a static value can be checked to make sure there are at least enough elements in the collection. This can be quite useful as students often try to get the first or last element out of a empty list.

### Warnings

A variable can point to objects of different types. With the exception of the numeric types, this makes the variable very unwieldly to use. This can be checked after merging a namespace by simply looking at the contents, and looking up their types. 

Heterogeneous collections are hard to deal with; unless there's a fixed structure to the collection, operations on its elements have to support all of the possible types. A collection keeps track of its elements's types. Adding something new to a collection will compare the types before and after adding, and will emit a warning if something's changed. 

Changing a collection while iterating over its contents is generally a bad idea. Python will not throw an error when this happens, which can lead to weird behavior such as skipping elements, endless loops, and collections sucking up all available memory. This can be checked by introducing _watches_ to the interpreter state, which will take a snapshot of all the mappings used to evaluate the loop generator. The watch will also save all changes to the values it's watching. These changes can then be used to emit warnings at the end of evaluating the loop. 

A while loop whose condition does change after an iteration is prone to endless loops. This is the case if all the variables in the condition still point to the same object, and all those objects haven't been changed either. Finding the paths in which this is the case isn't easy as our analysis only returns changes -- the opposite of what we want. The invariants can be found by taking all the complementary paths of all the changes, and retaining only those that are mergeable with all changes. 

A function call that did not end with an explicit return statement will return a `Ǹone` value. This can lead to confusing errors when a user forgot to return in just one of its branches. Since return values are treated as regular identifiers, we can use the existing logic provided by the namespaces to see which `OptionalMapping`s are still uninitialized at the of a function call. 

# Results

# Discussion

# Future Work

# References