% Static Analysis of Dynamic Languages
% Harm Delva

# Context & Product Statement

Ghent University has developed the Dodona platform (dodona.ugent.be) that students can use to submit solutions and receive immediate feedback. This feedback is in large part done through unit testing which lets the students know whether or not their solution is correct, but it doesn't really help them forward if it's not. To remedy this, a visual debugger is available which students can use to step through their program. Unfortunately it is limited in what it can do. More importantly, it can only let the user scroll through execution states. If an error occurred after a couple of thousand executed statements, this becomes a very tedious process.

There are ways to avoid the pain of debugging altogether. There are linter tools such as JSLint and Pylint which emit warnings on error prone patterns. The Dodona platform uses these tools as well, and relays the warnings to the students. 

The Pylint tool for Python has noticeably helped students avoid mistakes. This in turn allowed them to focus more on the problem solving aspect and less on learning how to program. The goal of this dissertation is to build on top of that, giving the students even more and even better feedback. An extensive data-flow analysis can untangle even the worst spaghetti code, which makes it a prime starting point for further analysis and feedback. 

# Related Work

## Static Analysis

NASA's source code of the Apollo 11 Guidance Computer was recently digitized and put on GitHub [^2]. All of it was written in an assembly language and the result worked just fine. The C programming language was developed for the Unix operating system so the latter could run on various CPU architectures @cdev. In essence it's a pretty thin abstraction over assembly, in the sense that it doesn't take much pressure off of the programmers. Unix worked just fine, just like NASA's guidance controller. One could argue that programmers don't need tools to aid them -- they seem to know what they're doing. 

[^2]: https://github.com/chrislgarry/Apollo-11

As the field of computing grew, and with it the projects, it started becoming apparent that programmers can't always rely on themselves. Even veteran developers occasionally discover a major vulnerability in their code -- like the Heartbleed vulnerability in OpenSSL [^3]. Of course everyone makes mistakes , but critical code should avoid them at all costs. A first line of defense against these costly mistakes is a proof of correctness. NASA's code was reliable because they had formal proofs that the most critical parts were correct [@nasa1; @nasa2; @nasa3]. Doing this is a massive amount of work, and a proof can still be wrong. More importantly, most verification frameworks are applied to designs and not implementations @visser.

[^3]: http://heartbleed.com/

Functional programming languages are closely related to provable correctness, while also automating some of the checks. Notable examples of such languages are Haskell and ML. Both have a strong theoretical foundation and provide the programmer with a strong type system to make it easier to reason about the code. This stands in strong contrast with languages like C. While Haskell was made to facilitate writing correct code @haskell, C was made to be close to the metal and efficient @cdev. The C compiler doesn't help the programmer nearly as much as the Haskell compiler. Developing correct and functional programs is obviously paramount to any programmer, so C's priorities don't always align with those of the developer. 

That's where static analyzers come into play. They analyze a program, either in its source code form or its compiled form, and try to find as many candidate mistakes as possible. These mistakes are often very subtle and rare, but even a single one can ruin someone's week. Code sample \ref{smp:shortset} comes from Google's Error Prone GitHub page [^4] and is a great example of how subtle serious bugs can be. The code seems to be just fine at first glance, the analysis in sample \ref{smp:shortset_f} reveals a subtle flaw.

[^4]: https://github.com/google/error-prone

\begin{code}
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=java,
  minted style=autumn,
  minted options={xleftmargin=-6pt, linenos},
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

\begin{code}
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=HTML,
  minted style=autumn,
  minted options={xleftmargin=-6pt, linenos},
  colback=bg }
error: [CollectionIncompatibleType] Argument 'i - 1' should not be 
passed to this method; its type int is not compatible with its 
collection's type argument Short
      s.remove(i - 1);
              ^
\end{tcblisting}
\caption{Analysis of code sample \ref{smp:shortset}} \label{smp:shortset_f}
\end{code}

Subtracting an `int` from a `short` results in an `int`. In this case it won't cause an actual bug yet, because both types use the same hashing function. But this isn't something a developer should rely on. The JVM originally didn't support generics, and their implementation is still a bit rough. The `remove` method of a `List` instance accepts any `Object` instance. This can result in calls that never actually remove anything. If that call happens to only occur in a corner case in the $1000^{\text{th}}$ iteration of a loop, this can lead to some very confusing bugs. 

### Popular Languages

Every practical programming language is Turing complete, so in theory they should all be equal. This is a misconception that has been called the Turing tar-pit @tarpit. Everything is possible in a Turing tar-pit, but nothing of interest is easy. Programming languages where things of interest are perceived to be easy can be considered powerful. These languages are often the ones with lenient compilers or runtime environments such as C, Python, Javascript, ... In other words, languages that don't get in the way of the programmer too often. These are also by far the most popular languages.

As illustrated in the previous section, this may not always be a good idea as humans tend to glance over subtle details. This makes the need for additional tools very important for any project that aims to achieve high reliability. The alternative is long and painful debugging sessions. At some point these languages no longer make it easy to do things of interest. 

#### C

The C programming language has been one of the most popular programming languages for a couple of decades now. Depending on who you ask, the best thing about C is either its efficiency or its simplicity. The latter is what gives developers the power they desire. This comes at a cost however; with great power comes great responsibility.

Let's focus on the other main attraction of C, its efficiency. This comes at a cost as well and it's one many people forget about. C's _raison d'être_ isn't making developers feel good about themselves, it's generating efficient code. It was created to be a thin abstraction layer over various assembly languages that were limiting software development at the time @cdev. This has left some holes in the C specification; for example, not all CPU architectures handle a division by zero, so the C specification doesn't specify what to do in this case. 

##### Undefined Behavior

There are a lot of things the C specification doesn't specify a behavior for, which leads to undefined behavior. Some are well-known, such as using freed memory. Others catch people by surprise. For example, the GNU libc website claims that `1/0` yields `inf` [^5], even though the C99 specification clearly contradicts them @C99. The C99 standard introduced the `INF` macro, but it doesn't specify which operations should result in one. Division by zero is still as undefined as it has always been. 

[^5]: http://www.gnu.org/software/libc/manual/html_node/Infinity-and-NaN.html

Entire papers have been written on the subject of undefined behavior [@lattner; @wang; @guide]. One striking thing is how recent a lot of these papers are. Even though the language is over 40 years old, this is still an active field of research. Compilers are getting more advanced and with it the optimizations they perform. Some of those optimizations rely on the fact that the compiler is under no particular obligation when optimizing code containing undefined behavior. 

\begin{code}
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=C,
  minted style=autumn,
  minted options={xleftmargin=-6pt, linenos},
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

Code sample \ref{smp:undef} was part of PostgreSQL @wang. The call to `ereport` on line 2 never returns. It does some logging before calling `exit`. In the mind of the developer this prevents the division by zero on line 7. Looking at the code on GitHub [^6], the function this sample came from indicates that calling it will return a `Datum` struct. According to the language specification, this function _must_ return a value each time it's called. The body of the null check does not return anything, so the compiler concludes that the rest of the function will also be executed and division by `arg2` will always occur. 

[^6]: https://github.com/postgres/postgres/blob/master/src/backend/utils/adt/int.c#L847

Division by zero is also undefined behavior in C, so the compiler concludes that `arg2` won't ever be zero -- it wouldn't get used in a division otherwise. As a result, the null check gets flagged as dead code, and is removed entirely. 

\begin{code}
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=C,
  minted style=autumn,
  minted options={xleftmargin=-6pt, linenos},
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

Code sample \ref{smp:undef_f} contains the fixed code. By adding an explicit return (in the form a macro), the compiler leaves the null check intact. Notice how the comment blames a "gcc bug". This illustrates how even experienced developers seem to misunderstand their language of choice. 

\clearpage

##### Tools

Not a single other programming language comes close to having as many external tools as C (and by extension C++). Many developers heavily depend on these tools in their usual workflow. One of the most established toolsets are the ones in Valgrind.

Valgrind is a suite of dynamic analysis tools, the most famous of which is Memcheck. Memory related bugs are some of the hardest to track down because they're ultimately undefined behavior. For example, using a memory location after freeing might not always crash the program. There's an informal term for bugs like these: _heisenbugs_. Something might go wrong but when you try to isolate the cause everything seems to be just fine. Especially since Address Space Layout Randomization (ASLR) tends to be disabled during debugging but not during normal execution. 

This is where Memcheck comes into play. It analyses the program during regular execution and keeps track of what happens to the memory. This way it can notice memory related bugs such as use-after-free and report them back to the developer. Unfortunately it's not a perfect solution. There can be false positives as well as false negatives, and it is quite incompatible with some libraries such as OpenMPI @mpi. 

A lot of companies rely on analysis tools to manage their large C projects, and when there's demand in a market, the supply will follow. There's an impressive amount of commercial analysis tools available. Coverity is one of the most established ones. 

Dawson Engler, one of Coverity's co-founders, is one of the leading researchers in the field of static analysis. He also co-authored a great paper in which he describes how difficult static analysis is in the real world @coverity. One particularly interesting part of the paper explains that there's a fundamental misunderstanding of what programming languages and compilers are. A programming language can exist as an abstract idea or a piece of paper. While the language a program is written in is whatever the compiler accepts. In other words, compilation is not certification. A good first step for a static analysis tool is to make sure that the input adheres to the specification. They go on to pose the hypothetical question: "Guess how receptive they [the end-users] are to fixing code the “official” compiler accepted but the tool rejected with a parse error?"

Even with a plethora of tools available, C remains an untamed beast. Some tools like Valgrind are great at what they do but are still limited. Other tools like Coverity seem to fight stubborn developers as often as they fight bugs. 

#### Dynamic Languages

According to Guido Van Rossum, he designed Python because he wanted to make a descendant of ABC that would appeal to Unix/C hackers @lutz. This is a bit worrying considering the previous section, as C is a remarkably hard language to analyze.
Javascript's situation isn't great either. There are no namespaces, no modularization system, no classes, no interfaces, no encapsulation, and so on. Eric Lippert, who was on the ECMA committee during the early days of JavaScript, has a Stackoverflow post where he discusses why Javascript is so ill-fit for static analysis and programming in the large @lippert. Even though Stackoverflow is a dubious source for any academic work, the author's experience should make up for it. 

Both Python and JavaScript are popular languages, so one might expect there to be a good amount of analysis tools for these languages as well. There seem to be two classes of analysis tools available for these languages right now: linters and type checkers. The linters are very popular, but as a later section will discuss they focus on error prone patterns instead of errors, and they prioritize efficiency over in-depth analysis. Type checkers are less popular, mostly due to circumstances. 

A type analysis tool for Javascript called JSure was developed in 2009, in part because the authors felt a need for deeper analysis than what JSLint could offer @jsure. They utilize an abstract interpreter to perform a flow- and context-sensitive analysis. Typescript was released a few years later, which fills a similar function as JSure, but with the researching power of Microsoft behind it. 
Python has a similar tool called MyPy which takes a fundamentally different approach @mypy. Python 3.5 introduced type hints for analysis tools such as MyPy to use. Although promising, the tool is plagued with a few limitations. For starters, it only supports Python versions since 3.5, while the users who stand the most to gain from it are the ones with large code bases -- which are predominantly on version 2.x for legacy reasons. On top of that, most existing Python 3 code does not even use type hints at this point, so stand nothing to gain from MyPy. 

What these two classes of analysis tools for dynamic languages have in common is that they're not as in-depth as the ones for static languages. Tools like Coverity for C and Error Prone or FindBugs for Java go as far as detecting racing conditions, while the tools for Javascript and Python are still experimenting with type analysis. This isn't because dynamic languages aren't important, they're used to power some of the biggest websites [@netflix; @facebook], it's just a hard thing to do. The following anecdotal examples illustrate why.

\clearpage

##### Classes

Lots of developers are familiar with class hierarchies and like using them, but there is one glaring difference between how static languages and dynamic languages handle classes. In most static languages, the class definition declares which methods and attributes are defined for objects of that class. This isn't the case with dynamic languages. Adding a method to an object is as simple as adding a callable object to the class's namespace. 
This is bad news for static analyzers that try to do type inferencing on dynamic languages. Consider the following bug that occurred while I was working on the Dodona platform.

The Ruby builtin `String` class provides a `split` method that does the same thing as in most other languages: its argument is some separator that is used to cut the `String` object into an array of smaller `String` objects. While writing a function that takes a few strings as arguments, the strings had to be split on whitespace. Calling the `split` method on one of the strings returns the same object instead. Using `puts` on that object just shows the same exact `String` we wanted to split, with all the whitespace still there, weird. The documentation says that it should work. Testing the method in a REPL (Read–eval–print loop) environment confirms that the `split` method should indeed split the `String`. A few confused hours later, it turned out the object wasn't a `String` but an `Array` of `String` objects. The Ruby documentation doesn't mention that `Array` defines a `split` method. Googling "ruby array split" refers to the Ruby on Rails documentation. A library silently declared a method on a builtin type, and `puts` prints every element of an `Array` on a separate line, so it just prints a single `String`. 

This approach to classes has some serious implications. Not only does it confuse newcomers, it confuses static analyzers as well. As these languages typically don't have type declarations, a logical alternative would be type inference. But consider the following case, `split` is being called on something that's either an `Array` or a `String` at the current stage of  inferencing. A regular Ruby analysis tool might conclude that the object must be a `String`. Analyzing Ruby on Rails code would either require the tool to analyze the entire framework's code first to learn that it adds a `split` method to `Array`, or it might even require a custom analyzer for Rails altogether.

This is why static analysis of dynamic languages usually starts with an abstract interpreter [@madsen; @jsure; @interpret]. You don't have to infer the type of an object if you know where it came from.

##### Refactoring

Python's lack of structure makes it hard to work incrementally. At some point during implementation of the interpreter developed for this dissertation, line and column information had to be added to the AST structure. Python's `ast` module contains everything one could need to work on Python's AST, including `lineno` and `col_offset` for seemingly all classes. With the exception of a few classes, such as `generator`. Implementing generators didn't come until much later, long after the analyzer started relying on the existence of `lineno` and `col_offset` for every node. 

Refactoring dynamic languages is a challenge. What if we want to change the name of a method called `add` to `insert`. Can we replace all occurrences of `.add` to `.insert`? That might change other classes as well. As discussed in the previous section, type inferencing is non-trivial. Even IDEs that are renowned for their ability to refactor code, such as the JetBrains IDEs, rely on manual feedback to filter out the false positives. Reporting false positives is not always an option however. That's what causes people to question your tool @coverity. 

#### `NaN`

There are problems that plague both static and dynamic languages, such as the existence of `NaN`. Some languages like Python and Java do a good job at preventing `NaN` from entering your program through raising exceptions, but once they do find their way in, you're left at the mercy of the IEEE 754 standard. The most recent version of the standard is IEEE 754-2008 and was heavily influenced by the ISO C99 standard, which introduced `NaN` and `INF` to the C language.

Since this standard, if one of the function's arguments is `NaN` but the other arguments already determine the function's value, the result is no longer `NaN`. For example, this means that `pow(1, NaN)` is equal to `1`. Mathematically this is at least a bit dubious, $1^{0/0}$ shouldn't be defined if $0/0$ isn't either. The C99 standard introduced various other oddities @C99: $\texttt{pow(-1, }\pm \infty\texttt{)}$ is `1` because all large positive floating-point values are even integers, and having a `NaN` in the imaginary part of a complex number does not matter when the value is being cast to a real number -- it's apparently possible for a value to be only slightly undefined. 

It has become normal for `NaN` values to somehow disappear as if nothing was wrong, leading to some very confusing bugs. A fellow student ran into the consequences while implementing his own dissertation (personal correspondence). The Theano framework [^7] has a function to compute the gradient of some expression. A `NaN` found its way into the input of this function but the result somehow didn't contain a single `NaN`. It became seemingly random noise instead. When implementing something computational, any person's first instinct would be that the algorithm is wrong. 

[^7]: http://deeplearning.net/software/theano/tutorial/gradients.html

Static analysis should be able to help alleviate this problem. If the floating-point value that's being used came from a `sqrt` function, there should be an `isNaN` check first. Alternatively, the Rust programming language tries not to hide the fact that floating-pointing values can be `NaN`. Without providing your own implementation of the `Ord` trait for the `f64` type, it's impossible to order a vector of `f64` values because comparing two `f64` values might be meaningless. It does however provide an implementation of the `PartialOrd` trait, which returns an `Option<Ordering>`, which makes it explicitly clear that the result can be `None`.
 
### Safe Languages

Some languages try to protect their users against themselves. Most of these languages are functional languages but there are notable exceptions such as Ada. 

#### Haskell

Having strong ties to the lambda calculus @haskell, Haskell is the archetype of a safe language. Like in most functional languages, all state in a Haskell program is implicit and by extension there are no side-effects. One of the core concepts of the language is that Haskell code should be easy to reason about. That's why this language deserves a section in a dissertation about static analysis and data-flow analysis; Haskell's design makes these things pleasantly simple. 

\begin{code}
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=python,
  minted style=autumn,
  minted options={xleftmargin=-6pt, linenos},
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

Consider the Python code in Code sample \ref{smp:aliasing}. Knowing that `x` and `y` refer to the same object is integral to realizing that `x.attr += 1` will result in a type error, as `x.attr` is `None` since the call to `reset`. Haskell has no side-effects -- function calls such as `reset(y)` on line 8 wouldn't be able to change anything. Additionally, it has no explicit state and thus no assignments and no aliasing to begin with. This trait can be mimicked in other languages using the Static Single Assignment (SSA) form where every variable is only defined once. Section \ref{ssa} will discuss this form and how it relates to code analysis.

#### Rust

A newer language that favors safety over accessibility is Rust. While it does have explicit state, Rust also makes the aliasing problem trivially easy for static analyzers. With some exceptions like the reference counting types, every piece of memory has a single owner in Rust -- which means that by default there's a single way to access any piece of data. This prevents aliasing problems like the one in sample \ref{smp:aliasing} because after executing `y = x`, `x` is no longer a valid identifier -- its data has been moved to `y`. On top of that, Rust has explicit mutability. This concept came from C/C++ where it's good practice to label all immutable things with `const`, except that Rust does it the other way around. This means that unless `y` was declared to be mutable, the assignment to `y.attr` wouldn't compile either. 

In languages like Java that heavily advocate encapsulation, it's not uncommon to write something like a `List<Element> getElements()` method, so that other modules can see which data is present. Returning the original list would mean anybody can change its contents. That's why it's considered good practice to return a deep copy of the list instead. Deep copies carry a significant overhead with them, so developers end up choosing between a safe architecture or an efficient implementation. Rust lets data be borrowed, so that other modules can see their contents. The result is a borrowed reference which is very similar to a pointer in C, with the exception that borrowed references carry a lot more type information with them. For starters, it's possible to borrow something mutably or immutably. If there is a single mutable borrow to an object, there can't be any other borrows at the same time. This is a mechanism that's supposed to avoid unexpected side-effects. Another thing that's part of a borrow's type is its lifetime. If object A wants to borrow something from object B, B has to exist at least as long as A. This mechanism directly prevents all sorts of memory related bugs that occur in C.

Rust is an interesting example of the relationship between languages (more specifically the compilers) and static analyzers. The Rust compiler enforces a lot of things that the C compiler doesn't, but that the C community has written their own tools for. One might wonder why C is even still around if Rust seems to be a more complete package. Part of the answer is that Rust is a very restrictive language, leading to frustrated developers. Keeping in mind Coverity's paper @coverity, it's a lot easier to ignore an analyzer than it is to fix a compile error. 

In fact, this can be seen as an instance of the loosely defined concept of an _XY problem_ [^8]. If a person knows how to do something using method X, he's less likely to learn method Y -- even if method Y is clearly the better choice. Rust has received a lot of criticism from renowned developers for ridiculous reasons, such as being unable to have two mutable references to the same object. That's how they would do it in C, so it must be the right way, even though `Alias XOR mutability` is a common design principle. This problem is relevant in this dissertation for two reasons. Static analysis tools run into the same mentality issues @coverity, and it shows that it's important to never pick up bad habits in the first place. The field of application of this dissertation is ultimately helping students learn how to program, before they have a chance to grow any bad habits. 

[^8]: https://manishearth.github.io/blog/2017/04/05/youre-doing-it-wrong/

### Tools

Linters are a class of static analyzers that are particularly interesting in the context of this dissertation. They're made to report error prone patterns during development to prevent actual errors, which is what sets them apart from other analysis tools. In order to do this they have to be very efficient, so that feedback can be given while the code is being written. Other analysis tools such as Coverity are usually only run nightly, or once per commit alongside unit testing @coverity. This also means that the analysis they're able to do is usually quite shallow, and purely syntactic. That doesn't make them any less useless however, they just report on generally bad patterns such as shadowing builtin function or writing long functions. One of the most renowned linting tools is JSLint (and its successors JSHint, ESLint), its Github page contains the following wisdom which describes the underlying philosophy quite well @jslint:

\begin{quote}
\say{The place to express yourself in programming is in the quality of your ideas and
the efficiency of their execution. The role of style in programming is the same
as in literature: It makes for better reading. A great writer doesn't express
herself by putting the spaces before her commas instead of after, or by putting
extra spaces inside her parentheses. A great writer will slavishly conform to
some rules of style, and that in no way constrains her power to express herself
creatively.}
\end{quote}

Few developers should find anything wrong in that reasoning. The main point of contention is which rules to follow. Crockford is a proponent of using certain language features sparsely @goodparts. In his book titled Javascript: The Good Parts he describes which features of Javascript he deems good (or great and even beautiful) and which parts should be avoided. The bad parts are everything that lead to error prone code in his experience. The usual example is the `switch` statement. He has given a presentation at Yahoo where he gives more examples of why JSLint discourages the patterns that it does @crockford. 

All analysis tools serve a common purpose, but are ultimately still very diverse. Static and dynamic analysis tools do things very differently and even within static analysis tools there's a lot of variation. Linters work mostly syntactical, focusing on speed and immediate feedback. Other tools like Coverity do a much deeper analysis and usually run nightly @coverity. Another major difference among static analyzers is soundness. Sound analysis tools are based on formal logic and more generally more comprehensive, but slower and more prone to false positives @soundness. Analysis of dynamic languages relies heavily on abstract interpreters [@interpret; @jsure; @madsen], and the closely related domain of symbolic execution which is discussed in section \ref{symbolic-execution} is applied to all languages for automated unit testing. 

With all these different approaches to the same problem, one might wonder which is the best.  Unsound methods seem to come out on top as most practical languages are unsound themselves @unsound. Others simply say that it doesn't matter how you do it, as long as the results are good @coverity. Every approach has its own pros and cons, and users can always use multiple ones to get the best possible coverage. 

## Code Smells

More important than how to do the source code analysis, is perhaps what to look for. Code smells are a taxonomy of indicators that something may not be quite right in the code, and the term was coined in a book on refactoring @refactor. In that book code smells are indicators that some refactoring might be necessary. They're aimed at software architects, and are informal descriptions of things that may induce technical debt. They're not actual bugs yet but error prone patterns, the sort of things linters aim to detect. One of the code smells is _cyclomatic complexity_, i.e. having too many branches or loops. This is something linters also detect, but it lacks the refactoring aspect of the code smell. 

Code smells were meant to be indicators, so that professional software architects knew where to focus their refactoring efforts. This may not be sufficient when trying to help new programmers, as they might need help knowing what and how to refactor. There are also code smells that linters currently do not pick up because they would require deep analysis. The most notable of which would be the _Code Duplication_ smell. JetBrains has developed some of the best refactoring IDEs such as IntelliJ and PyCharm, but started off by developing code refactoring tools such as IntelliJ Renamer for Java and Resharper for C`#`. These are made for large commercial code bases,  and are ultimately advanced heuristics that still miss obvious duplication. PyCharm 2016 was unable to find the duplication in code sample \ref{smp:duplication}, though it should probably be refactored to something like code sample \ref{smp:duplication_f}.

\begin{figure}[h]
\centering
 \begin{minipage}{0.31\textwidth}
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=python,
  minted style=autumn,
  minted options={xleftmargin=-6pt, linenos},
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
\vspace{10pt}
\captionof{code}{Duplication}\label{smp:duplication}
 \end{minipage}
 \hspace{9pt}
 \begin{minipage}{0.31\textwidth}
 %\vspace{-26pt}
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=python,
  minted style=autumn,
  minted options={xleftmargin=-6pt, linenos},
  colback=bg }
def foo(x):
    x += 1
    x = sqrt(x)
    return x - 1

u = foo(x)
v = foo(y)
w = foo(z)
\end{tcblisting}
\vspace{38pt}
\captionof{code}{Refactored version of code sample \ref{smp:duplication}}\label{smp:duplication_f}
 \end{minipage}
 \hspace{9pt}
  \begin{minipage}{0.31\textwidth}
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=python,
  minted style=autumn,
  minted options={xleftmargin=-6pt, linenos},
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
\captionof{code}{Alternative order of code sample \ref{smp:duplication}}\label{smp:duplication_2}
 \end{minipage}
\end{figure}

\vspace{30pt}

JetBrain's tools are closed source so it's unclear whether or not code sample \ref{smp:duplication} was deemed too simple to refactor. Assuming they work in a similar fashion as competing tools however, it's the order of statements that causes it to fail. Tools like PMD, Duploc, and CCFinder all work on a token stream in the same order as it appears in the file [@pmd; @duploc; @ccfinder]. Code sample \ref{smp:duplication_2} illustrates how sample \ref{smp:duplication} could be reordered so that tools can detect the duplication just fine. Compiler technology is a much more active field of research than duplication detection, and the problem of reordering instructions occurs there as well. One of their solutions is discussed in section \ref{ssa}, which discusses the Static Single Assignment (SSA) form. 

When analyzing student code, this can be a serious limitation. Consider code sample \ref{smp:duplication_3}, which just reads 4 numbers from `stdin`, increments them, and stores them in `x`. Even if this duplication gets detected, most tools are targeted at professional developers who would never write code like this. The critical difference is that the refactoring shouldn't introduce a new function but a loop such as in code sample \ref{smp:duplication_3_f}.

\begin{figure}[h]
\centering
\begin{minipage}{0.31\textwidth}
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=python,
  minted style=autumn,
  minted options={xleftmargin=-6pt, linenos},
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
\captionof{code}{Duplication}\label{smp:duplication_3}
\end{minipage}
\hspace{9pt}
\begin{minipage}{0.31\textwidth}
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=python,
  minted style=autumn,
  minted options={xleftmargin=-6pt, linenos},
  colback=bg }
x = []

for _ in range(4):
  s = int(input()) + 1
  x.append(s)
\end{tcblisting}
\vspace{55pt}
\captionof{code}{Refactored version of code sample \ref{smp:duplication_3}} \label{smp:duplication_3_f}
\end{minipage}
\end{figure}

There are some other code smells besides code duplication that could be interesting for new programmers. The following list contains some that at first glance look like the most promising.

  * _Large class_: A class that's grown too large and probably has too many responsibilities.
  * _Long method_: Much like the previous one, this method probably has too many responsibilities.
  * _Inappropriate intimacy_: When a class heavily depends on implementation details of another class.
  * _Cyclomatic complexity_: Too many branches, also informally referred to as _spaghetti code_. Linters do a fair job at pointing them out but offer little help in fixing them.

## Compilers

Static analyzers aren't the only tools that aim to make code better -- compilers do so as well. Refactoring from a software architect's point of view is aimed at making the code easier to read and maintain. Optimizations are aimed at making code more efficient. Optimization and refactoring sometimes do opposite transformations, for example in sample \ref{smp:duplication_3_f} where unfolding the loop results in the code in sample \ref{smp:duplication_3}. Both operations transform code in a conservative manner though, i.e. without changing the semantics (of valid code). Optimization is a very active area of research, with companies like Google and Apple working on the LLVM [^8], Oracle on the JVM, Red Hat on the GCC [^9], \ldots \ More importantly, even the most esoteric features in compilers have proven their value as they're part of a real product, which gives confidence that an analyzer that uses the same principles will work as well.

[^8]: http://llvm.org/foundation/sponsors.html
[^9]: https://gcc.gnu.org/steering.html

### SSA

Static Single Assignment (SSA) form is a program representation that's well suited to a large number of compiler optimizations such as conditional constant propagation @constant, global value numbering @equality, and code equivalence detection @equivalent. It was introduced in a paper by IBM @equality in the 80s but had little practical success initially. It wasn't until several optimizations were found [@increment; @dominance] that it started becoming popular. Since then it has found its way into most popular compilers. LLVM has used it in its virtual instruction set since its inception in 2002 @lattner, it's what powered Java 6's JIT @hotspot, and it's what's behind recent GCC optimizations [@memoryssa; @treessa].

The idea is very simple, every variable can only be assigned to once. This can be done by adding incremental numbers to each variable. For example, `x = x + 1` becomes $\texttt{x}_1 \texttt{ = x}_0 \texttt{ + 1}$. This is a trivial transformation for straight-line code, but becomes a bit harder when dealing with branches. Figure \ref{fig:ssa} illustrates how this is solved. Every branch introduces new variables, and a $\phi\textit{-node}$ gets inserted at the end of the branch. This node mimics a function call that can return either of its two arguments. This doesn't correspond to an actual function call, it's just a token that gets inserted to help with the analysis. Practically speaking, the $\phi\text{-nodes}$ get inserted at so-called _dominance frontiers_ instead of at the end of every branch @dominance. This optimization is of little relevance to the topic of this dissertation -- it's the underlying principle that counts. 

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

#### Memory SSA

As cited in the beginning of this section, the SSA form has a lot of benefits, but is limited to scalar values and is not well suited for compound structures such as arrays. One of the solutions to this problem is Memory SSA as implemented by GCC @memoryssa. This is a transformation that runs on a sufficiently low-level representation of the source code, where there's a notion of `LOAD` and `STORE` instructions. 

Because the actual memory locations are not known during compilation some abstractions are needed. GCC uses compiler symbols which they called tags to represent regions of memory along with two virtual operators `VDEF` and `VUSE`. For every `LOAD` a `VUSE` of some tag(s) gets inserted, and likewise for `STORE` and `VDEF`. There are three types of tags:

  * _Symbol Memory Tag_ (SMT): These are the result of a flow-insensitive alias analysis and are almost purely type-based. For example, all dereferences of an `int*` receive the same tag to reflect the fact that they might point to the same memory location.
  * _Name Memory Tag_ (NMT): These are the result of a points-to analysis applied after the program is in a SSA form and inherits their flow-sensitive properties @memoryssa. In other words, the GCC uses multiple different SSA forms.
  * _Structure Field Tags_ (SFT): These are symbolic names for the elements of structures and arrays. 

Tags get used in a similar way as variables in the regular SSA algorithm. Assigning to a tag is like assigning to all symbols that have received that tag. In the original implementation, _call clobbering_, i.e. a function call that alters the state of one of its arguments such as in code sample \ref{smp:aliasing}, is handled the same way as global variables @memoryssa. All  arguments that are known to get clobbered sometimes are put into a single set. Calling functions that would clobber one of them is assumed to clobber all of them. This pessimistic approach has been replaced in recent implementations, where all related code has been rewritten to use an _aliasing-oracle_ @oracle.

LLVM's usage of Memory SSA is not quite as documented. Their documentation refers to GGC's paper [@memoryssa; @memoryssallvm] and mentions that \say{Like GCC’s, LLVM’s MemorySSA is intraprocedural.}. As mentioned in the previous section, this isn't entirely true for GCC anymore. It doesn't seem to be true for LLVM either. A recent publication describes _Static Value-flow_ analysis which produces an interprocedural SSA form @svf. It has been part of LLVM since 2016 and like GCC's implementation it uses the results of an external points-to analysis.

## Symbolic Execution

Rather than using concrete values, symbolic execution uses symbolic values. Those values represent possible values through a set of constraints. This technique is commonly used to find interesting corner cases for unit testing [@symb1; @symb2; @symb3] and the original SSA paper used similar principles to make their analysis more precise @equality.

At each branch point a _path constraint_ gets introduced, which is a symbolic expression that must be true for the current branch to be taken. If there are no values that satisfy all the constraints, the branch gets flagged as dead code. In the case of a static analysis tool this will most likely result in a warning, while a compiler would just remove the dead branch. For example, consider code sample \ref{smp:symb}. The condition `y > 0` becomes a constraint during the execution of the positive branch, as well as `x > -1`. The former constraint is pretty simple to add, the latter requires some serious bookkeeping. Which implies a close relation between symbolic execution and data-flow analysis.

\begin{code}
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=python,
  minted style=autumn,
  minted options={xleftmargin=-6pt, linenos},
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

\vspace{-2pt}

Symbolic execution is a very powerful tool but comes with a few limitations. One is the _state explosion problem_. The number of paths can increase very fast. The Java Pathfinder manages this problem using state matching and backtracking @symb2. State matching will check if a similar state has already been processed. If it has, it will backtrack to the next state that still has unexplored choices. 

An advanced symbolic executor for Python is PyExZ3 [@symbex; @symbex2], which is built on top Microsoft Research's Z3 theorem prover. Z3 is a remarkably good fit to reason about programming languages and even supports strings and other sequences @z3. There are even extensions that add support for regular expressions and other advanced constraints @z3str. As another team at Microsoft Research has pointed out however, sound methods such as symbolic execution tend to run into problems when applied to unsound languages @unsound. Z3 heavily relies on type information, which is not present in Python code. Lists and dictionaries are still open challenges @symbex, as Z3 requires an element type upon declaring a sequence @z3. 

\vspace{-1pt}

Symbolic executors typically focus on scalar values. Collections tend to be a lot harder to analyze. Consider code sample \ref{smp:symb2} for example, which defines a function `foo` that prints `homogeneous` if and only if its argument is a collection and all elements in that collection are of the same type. The positive branch introduces a constraint on the length of `x`. Z3 can already handle this quite well. The relation between uniqueness and the length of a set is considerably harder though, and it's a very _pythonic_ pattern. 

\vspace{-1pt}

\begin{code}
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=python,
  minted style=autumn,
  minted options={xleftmargin=-6pt, linenos},
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

There's a relatively unknown Frisian god of reconciliation, justice, and mediation called Foseti. These are also some qualities that a good static analyzer should aim to have as well to gain a user's trust @coverity. The analyzer developed as part of this thesis is called Fosite, as foresight would be another good quality. 

## Goals

Unlike most existing tools, Fosite's focus is analyzing small pieces of code submitted by students, and this comes with both advantages and disadvantages. The biggest advantage is that we're free to explore slow but accurate  methods. Not entirely free however as feedback should still be fast. Imagine being a stressed out student, working on your final exam and every submission takes a minute to run because submissions come in faster than they get processed. 

The most important requirement of all is that all warnings have to be as helpful as possible. Providing the right details should make the errors more convincing and will be met with less resistance from the user @coverity. This is doubly important for new programmers; simply pointing out bad style is not enough if the user does not know how to fix it. Fosite uses a data-flow analysis to pinpoint the source of problems and uses that information to inform the user. 

\begin{code}
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=python,
  minted style=autumn,
  minted options={xleftmargin=-6pt, linenos},
  colback=bg }
if x < y:
  z = x
else:
  z = y
\end{tcblisting}
\caption{Inefficient Implementations} \label{smp:shit}
\end{code}

We'd like to be able to recognize suboptimal patterns as well, such as in code sample \ref{smp:shit}. Replacing all occurrences of this pattern with a call to `min` not only makes the code easier to read, it's also significantly more performant in interpreted languages such as Python. To be able to recommend sound code transformations, we'll rely on the research done in the area of compiler technology. The analysis should also work as close as possible to the submitted code though, and at the very least maintain a one-to-one relationship to it. This is important because the end goal is automated refactoring, which becomes hard when the input becomes mangled beyond recognition. 

## Approach

As pointed out on sections \ref{popular-languages} and \ref{compilers}, analysis of dynamic languages is often done using abstract interpreters, and many modern compilers use an SSA form to perform sound optimizations. The PyPy interpreter for Python constructs this form using an abstract interpreter as well @pypy, although their solution is intraprocedural, and others have successfully used abstract interpreters to perform a may-alias analysis on Python with the goal of optimization @interpret. Fosite is based on their conclusions and results but it ultimately has a different focus, i.e. detailed static analysis and automated refactoring. 

The result of our analysis isn't exactly an SSA form for a few reasons. The Memory SSA approach by the GCC and LLVM is a bad fit since it relies on a low-level representation, which is a luxury we don't have as we need to stay close to the original source code. Regular SSA is of course an option since PyPy does it @pypy, but can't handle collections @memoryssa.

Fosite emits more general _use-def_ information instead. 
A _use_ refers to any data dependency, such as resolving a variable name or retrieving an element from a collection. 
A _def_ is the opposite such as assigning something to a variable name or inserting an element into a collection. 
Within Fosite, a _use_ and a _def_ are respectively called a dependency and a change. 
As with SSA, incremental numbering can be applied to subsequent changes.
The main difference with regular SSA is that this requires a separate data structure. 
This allows a single expression or statement to cause multiple changes. 
A conditional statement is exactly that -- a statement, and it can be useful during refactoring to treat it as a single atomic thing. Its dependencies and changes are the sum of its branches'. In other words, the external data structure allows for a more hierarchical analysis. 

To achieve the same level of precision as Memory SSA, Fosite uses two kinds of changes and dependencies. For starters, there's the usual identifier dependency, which is used to model reachability of data. Consider code sample \ref{smp:dep} in which `x` gets defined twice. In between assignments the first value of `x` gets used to call in a call to `print`. The `print` must come before the second assignment to `x`, as the intended data is no longer reachable after the assignment. 
Another sort of dependency is the object dependency, which is used to model a state dependency. The second assignment to `x` assigns a list to it, which gets printed as well. Before printing however an element gets appended to it. Appending an element to a list doesn't change anything about the identifier and thus can't be modeled in the same way. In other words, the final call to `print` has a dependency to both the `x` identifier and to whichever object `x` points to at the same time -- and both dependencies serve their own purpose. 

\begin{code}
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=python,
  minted style=autumn,
  minted options={xleftmargin=-6pt, linenos},
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

## Languages

Since languages tend to share a lot of features, it's not unthinkable to have an abstract interpreter that can process multiple languages. The first thing this would need is a common input format, which in Fosite's case is its _General Abstract Syntax Tree_ (GAST). It has to be able to capture all _syntactic_ features of the languages it supports -- the semantics aren't relevant for an AST. Adding support for an additional language will require some new nodes or some extra information in existing nodes. Since only things will have to get added interpreting the existing languages can just ignore the additional node types. Another thing that's special about the GAST is that every node has its unique identifier, and the identifiers are totally ordered. If one node's identifier is less than another's, that must mean it came before that other node in the original source file. This is important to accurately report code points, but also because some optimizations rely on it. 

The interpreter has to be able to add semantics to the common AST structure. A different `Executor` instance may be assigned for every supported node for every supported language. Languages that share the same features can reuse existing `Executor` implementations. Common and fundamental features such as function scoping are even available inside the interpreter's implementation itself. 

As the test data itself is in Python, the Python programming language was the main focus during implementation. 

## Analysis

While linters recognize error prone patterns, an interpreter can recognize error prone patterns and logic, as well as some outright errors. An additional benefit of an interpreter-based approach is that it approaches feedback the same way a person would: starting at the beginning, step by step. There are some interesting things that an interpreter can do that linters (or at least PyLint) can't. 

\begin{code}
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=python,
  minted style=autumn,
  minted options={xleftmargin=-6pt, linenos},
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

Code sample \ref{smp:return} contains an error prone pattern of Python-like code. A student had written something like this which resulted in one out of 200 unit tests failing. Written like this, it's possible that none of the intended `return` statements get executed. If this happens the return value is going to be `None`, which makes the unit test fail in an unexpected way -- nowhere did they specify a `None` value should be returned. Fosite gives an accurate description of the cause -- the function did not return under these conditions -- instead of just the result. 

\begin{code}
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=python,
  minted style=autumn,
  minted options={xleftmargin=-6pt, linenos},
  colback=bg }
x = []
...
while tuple([0] * i) not in x:
    ...
    x += tuple([0] * i)
\end{tcblisting}
\caption{Heterogeneous Collections} \label{smp:nohomo}
\end{code}

The test data contains an exercise that required that a sequence of tuples should be generated, stopping whenever a tuple of zeroes has been added. Code sample \ref{smp:nohomo} is based on one of the submissions. Up until the adding of the tuple of zeroes, the type of `x` had been `List[Tuple[int]]` (in the notation used by Python 3.5 type hints). Instead of appending the tuple however, `+=` will concatenate the tuple's elements to `x`. This changes the type to `List[Union[int, Tuple[int]]]`. This transition to a heterogeneous collection is valid Python code but ultimately very error prone. In fact, this causes an infinite loop in this case, as the expected element never gets added.

\begin{code}
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=python,
  minted style=autumn,
  minted options={xleftmargin=-6pt, linenos},
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

Although deciding whether or not any given program will stop is impossible, it is possible in some cases. Those cases also happen to be quite common. Code sample \ref{smp:nostop} is an excerpt from a submission. The student intended to tokenize the string `x`, building the token in `list2`. Every token should then get translated and the translated token gets stored in `list1`. There are a number of mistakes but the most important one is the endless `while` loop. The student wanted index `i` to be a starting position of the token, with the `while` loop building the token from that point. That's of course not what the code does, the same character will get added over and over since none of the values in the loop condition ever change. Data-flow analysis remembers when and where variables get their values, so it can be used to recognize that the variables are still the same. 

The secondary goal of the interpreter is to open the door to code refactoring, through the means of a data-flow analysis. Every evaluated node during interpretation emits a list of changes and dependencies as discussed in section \ref{approach}.

# Implementation

Fosite is an abstract interpreter. It uses abstract pointers, which can be used to fetch abstract objects from an abstract memory. The objects themselves have no value, but they do have a notion of types, attributes, and elements. There is a notion of namespaces where names get mapped to objects, and a notion of heterogeneous collections. In essence, the interpreter tries to get as close at it can to actual interpreter without having actual values. This not as obvious as it sounds. For example, it's tempting to cut corners when implementing the _Method Resolution Order_ (MRO), variable capturing in closure definitions, or the explicit `self` argument in Python. Simple approximations of these behaviors would suffice for well written code -- but targeting such code makes no sense for a static analyzer. We have to be able to analyze _really_ bad code as well. 

\begin{code}
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=python,
  minted style=autumn,
  minted options={xleftmargin=-6pt, linenos},
  colback=bg }
x = 0
y = 0
z = 0

if ...:
    x = 9
    if ...:
        y = 5
        if ...:
            z = 1

a = complex_computation(x)
b = complex_computation(y)
c = complex_computation(z)
\end{tcblisting}
\caption{State Explosion} \label{smp:branch}
\end{code}

As an abstract interpreter uses abstract values, it can't decide which branch to take at every branch point, so it will explore every branch. This can quickly lead to a state explosion problem as the number of branches can increase exponentially. Consider the Python-like code in code sample \ref{smp:branch}. There are branching points on lines 5, 7 and 9, so that the functions calls on lines 12, 13, and 14 can each be executed once for each of the four possible execution paths for a total of 12 calls to `complex_computation`. Each argument `x`, `y`, or `z` only has two possible values though, so we should be able to do better. The Fosite interpreter will create new execution paths at every branching point, but those paths will also be merged after evaluating the branch point. This means that there will only be a single active execution path left upon executing the calls on lines 12-15. Merging execution paths preserves all the relevant information of each branch, as discussed in section \ref{namespace}. The result still has exponential complexity, but no longer in the number of branch points but in the number of possible values 

## Objects 

As alluded to in the beginning of section \ref{implementation}, the Fosite interpreter keeps track of attributes and elements. Attributes can reuse the namespace logic as described in sections \ref{namespace} and \ref{name-resolution}. Elements are a lot harder to model and are covered in section \ref{collections}.

Everything is an object in Python, even classes  become class objects upon definition. An object of a class has a reference to its class object. Among other things, this reference gets used during name resolution. Every class can also extend multiple other classes, called base classes, in a similar way. This can easily be modeled in an abstract interpreter using a list of pointers.

The type of an object is harder to model however. In many object-oriented languages, an object's class is its type. Python's situation is a bit more complex since it has multiple inheritance, and classes are objects as well. 

\begin{code}
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=pycon,
  minted style=autumn,
  minted options={xleftmargin=-6pt, linenos},
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

Code sample \ref{smp:types} illustrates why this is odd. The `type` function returns a class object, so that `type(42)` returns the class object of name `int`. Using the same function to get the class object's type returns a class object of name `type`. Requesting that object's type reveals something strange -- `type` is its own type. This seemingly cyclic dependency gets implemented in CPython using a type flag, if that flag is set it'll return the type class object when needed. In other words, the `type` object doesn't have a reference to itself, it'll get a reference to itself at runtime when needed. 

The type of a value is the same as its class object. A class's basetypes have nothing to do with its type -- a class object's type is always `type`. These semantics are quite straight forward to model in an abstract interpreter: the list of base class references are still there, but there's also a type flag. When that flag is set, the `type` function shouldn't use the base classes but fetch the pointer to the `type` class object.

## Paths and Mappings

In order to report the cause of errors and warnings accurately, we need to know the source of every value. A path corresponds to a sequence of code points so that the user gets an idea of the execution path that lead to a problem. Every entry in the path gets called a path node. Examples of path nodes include which branch of a conditional was followed, assignments, and function calls. The path nodes should submit a logical ordering, so that users can easily interpret results.

As mentioned in section \ref{languages}, AST node have totally ordered unique identifiers. A first attempt at defining the path node would be to just reuse the AST identifiers. This works fine until function call come into the picture. A function call will come after the function definition, and its identifier will be larger than any of the function definition's nodes. 
This would place the function body execution before the function call itself. 
On top of that, this approach does not support executing the same node more than once. 
A better solution is to define a path node to be an ordered collection of AST nodes -- the nodes that are currently being executed. 
Some nodes need extra information, a branch node for example needs to indicate which branch was actually taken. 
Each branch is incrementally numbered, and contains the total number of branches for practical reasons (see sections \ref{namespace} and \ref{function-calls}).
The actual branch numbers are of no concern, their main purpose is telling possible branches apart. Definition \ref{def:path_node} describes the structure of a path node, and definitions \ref{def:path_order}, \ref{def:contain}, \ref{def:complement}, \ref{def:mergeable}, and algorithm \ref{alg:complement} introduce useful properties of paths.



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

A mapping is simply a pair of the form `(Path, Pointer)`. Because they usually appear in multiples, they can be implemented as a list of `(Path, Pointer)` values instead. In this case, every path in a mapping must be distinct; there are no paths that are contained by another path in the same mapping. 

## Boolean Expressions

A boolean expression can be arbitrarily hard to evaluate. When used in a conditional statement, we can't always decide whether or not a given branch gets taken. The best we can do in these cases is concluding that the branch _might_ get taken. Evaluating any boolean expression can thus result in `True` or `False`, as well as `Maybe`. 

\begin{code}
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=python,
  minted style=autumn,
  minted options={xleftmargin=-6pt, linenos},
  colback=bg}

if current is not None:
  print('given {}-{}'.format(current.year, current.month))
else:
  current = datetime.now()

\end{tcblisting}
\caption{Conditions} \label{smp:conds}
\end{code}

Code sample \ref{smp:conds} shows that in some cases, we really need an accurate answer. This is a pattern that occurs when dealing with optional arguments or when writing library functions. The negative branch should only get executed when `current` was not `None`, so that an actual argument doesn't get overwritten. On the other hand, it _must_ be executed if `current` was `None`, so that further evaluation doesn't result in a false type error. 

The `is` operator compares the addresses of two objects and returns `True` if and only if they're equal. We can mimic this behavior -- and answer with certainty and under which conditions the two operands' point to the same location. The resulting mapping will use the merged paths of the operands to point to the `True` object. The `==` operator should be similar. Technically it depends on the implementation of the `__eq__` method, but let's assume that it has a decent implementation. In that case it should at least return `True` if both operands point to the same object -- as with `is`. A similar reasoning can be applied to the `!=`, `<=`, and `>=` operators. 

We can also handle the `and`, `or`, and `not` operators in a similar way. If both operands already point to `True` we can merge the paths and return a mapping that points to `True` as well. The other two operators are analogous.

We combine the paths of both operands to get a new mapping. This means means that we must only consider path pairs that are mergeable, if not those operand pairs cannot actually exist at runtime. Failure to meet this requirement will lead to false positives very quickly.

## Conditionals

Evaluating the test of a conditional branch can give us useful information to evaluate the individual branches with. If that information includes for example that we are sure that `x` is not `None`, we should disregard any mapping that says otherwise. Even better, we can exclude any mapping that would occur under the same contradictory conditions -- even if those mappings don't have an explicit connection to `x`. 

\begin{code}
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=python,
  minted style=autumn,
  minted options={xleftmargin=-6pt, linenos},
  colback=bg}
if cond1: # Condition 1
  y = None
  z = None
  
if y is not None: # Condition 2
  print(z.attribute)

\end{tcblisting}
\caption{Conditions} \label{smp:exclude}
\end{code}

In code sample \ref{smp:exclude}, there's an implicit relation between condition 1 and condition 2. Going back to section \ref{boolean-expressions}, the result of the test of condition 2 will contain a mapping $(p, x)$, where $p$ contains a node indicating that the positive branch of condition 1 was taken, and where $x$ is a pointer value to `False`. This means that any mapping containing $p$ during the execution of the positive branch of condition 2 cannot occur during actual execution. We will call this concept \textit{path exclusion}, and paths such as $p$ are called _restricted paths_. Observation \ref{obs:exclude} summarizes this more formally. 

\begin{observation}
\label{obs:exclude}
Assume that resolving an identifier $x$ results in a set of mappings $M$. Every mapping $m \in M$ is of the form $(p, a)$, where $a$ is a pointer value, and $p$ is the execution path that mapped $x$ to $a$. 

Let $R$ be the set of restricted paths. Given a mapping $(p_m, a) \in M$, if there exists a path $p_r$ in $R$ for which holds that $p_m$ contains $p_r$ (by definition \ref{def:contain}), we can exclude the mapping from the current evaluation. 
\end{observation}

## Loops

Without being able to accurately evaluate loop conditions or generators it's impossible to know how many times a loop body gets executed. There are a few different approaches to this problem. The most accurate one is to iterate until we can conclude doing further iterations won't benefit the analysis anymore, we say that a _fixed-point_ state has been reached in this case. Every iteration will likely change something though, so that recognizing a fixed-point state isn't as easy as waiting for an iteration that changes nothing. 

An easier approach that still has sufficient accuracy is to evaluate every loop body exactly two times. Theoretically it's possible to not even do a single iteration -- this leads to false positives however as most exercises can guarantee that some loops will always have to do something. There are two reasons for why two iterations is significantly more accurate than a single one. For starters, the first iteration can redefine values that are only used within the loop body. If this redefinition is wrong, this can only be recognized by evaluating it a second time. The second reason is to differentiate between the `break` and `continue` statements. As section \ref{namespace} will discuss, these two statements will hide changes until some later point during execution. For the `break` statement that point is the end of the loop, for the `continue` statement however this point is the beginning of the next iteration. 

The analysis of loops goes hand in hand with _watches_, which are used to compare the execution state before and after executing the loop. They start in a setup phase, during which it will learn which components to watch. A new watch gets made at the beginning of evaluating the loop test or loop generator, and it will store all data dependencies (corresponding to the of the data-flow analysis) for that expression. 
It will contain the returned mappings for the identifiers, along with a list of used objects.
The watch leaves the setup phase before evaluating the loop body, and it will now store all data changes for the identifiers and objects that are being watched. 

The information in a watch can be used to see  whether or not iterating affects the loop test or the loop generator. Knowing under which conditions iterating changed something is easy, the watch already contains that information. 
Finding the paths that don't change anything isn't as easy because the watches only contain changes -- the opposite of what we want. The invariants can be found by taking all the complementary paths of all the changes, and retaining only those that are mergeable with all changes. Section \ref{warnings} will discuss how this information can be used as an indicator of error prone code.

## Namespace

Namespaces are the most essential component of the interpreter. We want to give the most descriptive and helpful messages we can, by describing the conditions in which something occurs. Paths are a first step towards describing these conditions, but namespaces are where they're stored. There are a few layers of abstraction required to make this possible, and they will be introduced incrementally.

#### OptionalMapping

Mappings have already been introduced, but they contain a pointer value which is not enough to indicate an uninitialized variable. A different structure is used to this end, where the pointer value is optional. An `OptionalMapping` with a missing pointer value indicates an uninitialized variable and also describes why the variable is uninitialized.

#### Branch

The `Branch` struct is the first layer of a namespace and it's where names are added, its internal structure is of the form `HashMap<String, OptionalMapping>`. Every branching point during execution can induce several new branches in a namespace which are separated during execution, so that the negative and the positive branch of a conditional statement do not influence each other for example. A `Branch` struct only contains the changes that happened in its own branch, the changes that happened before the branching are stored in different `Branch` structs, which leads to a sparse data structure. 

#### StatisChamber

If we encounter a `break` statement while evaluating a loop body, the evaluation of the current execution path terminates. The changes made until that point have to be saved, as they'll become relevant again after the loop has been evaluated. Function calls require the same to handle different return points. A `StatisChamber` contains a `HashMap<Path, Branch>`. The path key is used because the control flow can be broken at multiple points.

#### SubFrame

For every branching point, we'll use a `Branch` and three `StatisChamber`s -- two for loops, one for function calls. Loops require two statis chambers to keep the ones caused by a `continue` separate from those caused by a `break`. These get stored in a `SubFrame` struct.

#### Frame

This is the first namespace component that contains some actual logic. Every branching point leads to the creation of a new `Frame`. This structure contains a _cause_, the path node where the branching happened. It also contains a subframe for each possible branch at the cause node. There is only one subframe active at any point during execution and its index is stored. Algorithm \ref{alg:setframe} describes how a mapping gets inserted into a frame, the `insert` method of a `StatisChamber` will simply insert into each of its branches. 

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

\begin{code}
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=python,
  minted style=autumn,
  minted options={xleftmargin=-6pt, linenos},
  colback=bg}
# frame 1
x = 'x'
y = 'y'

while True:
    if 'cond': 
        # frame 2, subframe 2.1
        if 'cond2': 
            # frame 3, subframe 3.1
            x = 9
            continue
    else:
        # frame 2,subframe 2
        if 'cond4': 
            # frame 4, subframe 4.1
            y = 7
            break

    z = x + y

z + 'z'
\end{tcblisting}
\caption{Broken Control Flow} \label{smp:statis}
\end{code}

Code sample \ref{smp:statis} illustrates how the statis chambers are used. When executing the `continue` statement on line 11, the active branch will only contain a mapping for `x` in which it points to an `int` object. 
This will stop the current execution path, and conclude the execution of the conditional on line 6 as it does not have a negative branch. Rather than merging the contents of subframe `3.1` into subframe `2.1`, it will put the contents into the loop static branch of subframe `3.1`. The key used to store into the statis chamber is a path containing only a single node: the node corresponding to the positive branch of line 8. 

The first time the addition on line 19 gets executed is actually safe -- `x` and `y` will always have type `str` at this point, and `z` will also receive a mapping to an object of type `str`. Every loop gets evaluated twice as discussed in section \ref{loops}, and a mapping in which `x` points to an `int` since line 10 in the previous iteration is now part of the active branch.

Line 21 isn't safe either, there are at least two execution paths that left `z` uninitialized: if either the condition on line 8 or the one on line 14 was true. Before the addition on line 19 was executed, the previous value was stored into the statis chambers that existed at the time. All but one statis chamber will now contain an empty mapping for `z`, indicating an uninitialized value. These mappings enter the active branch at the end of evaluating the loop, to give us the wanted analysis result. 

#### Namespace

The actual namespace is simply a list of frames, one for each branch point that's being executed. Looking up an identifier is simple: look for the most recent frame that contains a mapping for that name. Name resolution is so simple because the other operations do all the heavy lifting. There are three other operations:

  * \textit{Grow}: Uses a path to create new frames until there's a frame for each node in the path
  * \textit{Insert}: Grows the namespace when needed, and then inserts a mapping into the last frame. 
  * \textit{Merge}: Merges the last frame into the previous frame.

For the sake of data sparsity, growing is done upon insertion -- and not upon the actual branching. Bearing in mind that every object has a namespace as well, we don't want to grow each of their namespaces every time -- its namespaces probably won't even change as most objects are immutable literals. 

#### Grow

If the current namespace has $n$ frames, and the given path has $m$ nodes, we must add $m-n$ frames -- corresponding to the last $m-n$ path nodes. The correctness of this approach relies on a bit of inductive reasoning. 

The active execution path will always be at least as long as the number of frames in any namespace. All the namespaces that have been changed have the same number of frames. If a change has been made in the current branch, growing has added frames until there are as many frames as there are nodes in the execution path. If a change was made in some branch that's already been executed, and is thus no longer part of the execution path, merge operations will have reduced the number of frames until the length is equal to the length of execution path. The namespaces that have not been changed have strictly less frames, corresponding to the length of the execution path of their last change.

The cause nodes of the frames of any namespace form a prefix of the active execution path. This is a trait of the language, since Python (or any other language we'd target) does not have a `goto` statement, there's a fixed structure to the branching points.

#### Merge

The merge operation combines the results of the last frame, removes it, and puts the merged result into the frame that is now last. An argument determines the destination of every subframe's content -- either into the regular branch or into a statis chamber. The current method is specific to function scoping, but block scoping can be added in a similar way.

\begin{code}
  \begin{tcblisting}{listing only, 
  arc=0pt,
  outer arc=0pt, 
  boxrule=0.2pt,
  minted language=python,
  minted style=autumn,
  minted options={xleftmargin=-6pt, linenos},
  colback=bg }
x = 42
y = 'string'

if cond:
  x = '42'

x + y 
\end{tcblisting}
\caption{Merging}\label{smp:merge}
\end{code}

The name resolution stops at the most recent frame that contains that name. Code sample  \ref{smp:merge} illustrates that this isn't just an easy solution -- it can also be a useful one. If the negative branch of the condition at line 4 is taken, execution will fail at line 7. Variable `x` still has the value it received at line 1, but only reporting this paints an incomplete picture. It only has that value if the negative branch was taken. So if we want to accurately describe why it has that value, that information should be there as well. 

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

Merging begins with collecting the names of all identifiers that have changed in each branch of the frame. A variant of the cause node gets created for every subframe, using the index of the subframe. These augmented nodes are then used to _augment_ the paths in the statis chambers, which then get moved to the frame that is now last. The statis chambers are all of the form `HashMap<Path, Branch>`, where the `Path` was supposed to keep the different sources of broken control flow apart. The `augment` method will add a new node to every key path -- future additional key paths won't contain this node, which is how different points of broken control flow are being kept separate. 

Code sample \ref{smp:merge} showed us some behavior we want during name resolution. We can achieve this by incorporating name resolution in the merge operation. Every identifier that has been changed in any branch will get resolved in every branch, the result gets augmented gets augmented with a variant of the cause node, and is then stored in the frame that is now last. This will ensure that all the relevant mappings for any given identifier can always be found in a single frame. More importantly, the `augment` method will update path mappings to reflect when an identifier didn't change in some particular branch.

Function calls and loops can also create a new frame in namespaces. While merging after a conditional branch can place things inside a  statis chamber, merging after a loop or a function call will place the contents of a statis chamber into the active branch. The key paths are merged into the mappings of their corresponding branches first because previous merge operations have not updated those mappings yet.

## Name Resolution

Namespaces by themselves aren't enough to implement all the name resolution behavior; name resolution uses several different namespaces. It's possible that a variable is sometimes uninitialized in a namespace, in which case resolution can continue using another namespace. The unresolved paths are carried over to the resolution in the next namespace. All returned mappings have their paths merged with the unresolved paths to reflect the fact that name resolution continued in the next namespace. Algorithm \ref{alg:chain} illustrates the general method of name resolution, the `next_namespace` function depends on the kind of name being resolved. An error is emitted if `unresolved` is not empty and there are no more namespaces to try.

\begin{algorithm}
    \caption{Resolve}\label{alg:chain}
    \begin{algorithmic}[1]
        \Function{resolve} {name}
          \State $\texttt{result} \gets \texttt{Mapping}::new()$

          \State $\texttt{unresolved} \gets [\, \texttt{Path}::empty()\,]$

          \While{ \texttt{unresolved.len() > 0} }
              \State $\texttt{new\_unresolved} \gets [\,]$

              \State $\texttt{namespace} \gets \texttt{next\_namespace()}$
              \State $\texttt{mapping} \gets \texttt{namespace.resolve(name)}$

              \ForAll{$(p, x)$ \textbf{in} mapping}
                \ForAll{ \texttt{unresolved\_path} \textbf{in} \texttt{unresolved}}
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

Identifiers (also called names) are stored in up to four different namespaces:

  * The local scope gets created at the start of every function call, which why it's also called the function scope. All variables made during a function call will live here, and stop existing when the function call is done.
  * The enclosing scope is most commonly used to capture variables in lambda definitions, but the same principle holds for any sort of function definition. Variables that occur in a function definition and that are already in scope during the definition will get "saved" to the enclosing scope of the function. 
  * The global scope, where most class and function definitions end up going, and where students first write their first programs in. 
  * The builtin scope, which is not to be confused with the standard libraries, contains essential Python features such as the `int` type and the `sorted` function. 

At any point during execution there is either two or four active scopes, depending on whether or not a function call is being executed. Every function call creates a new empty local scope, but reuses the existing enclosing scope associated with its definition. 

### Attributes

An object's attributes don't necessarily exist in its own namespace. A lot of them, especially its methods, will exist in the namespace of its class object or one of its base classes. Python uses its own _Method Resolution Order_ (MRO) to define the order of the base classes in case of multiple inheritance. The first base class in the definition will get used first, where MRO can be applied again, ... This means that the second base class of the definition will only get used if the first base class did not contain an attribute of that name, and neither did any of its extended base classes. 

### Methods

Functions are just callable objects, and a method seems to be a function that's part of an object's namespace. There is one obvious difference however, the explicit self of a method definition isn't part of the method call. This is because Python hides an implicit creation of a `method` object, which isn't just a callable object. In fact, it encapsulates a callable object -- the function attribute. It also contains a reference to object it's being called on. The method object itself is callable as well, and calling it will call the embedded function with the embedded parent object prepended to the arguments. 

## Collections

Collections are a challenging component of dynamic programming languages. When an element of a collection is used in any operation, we have to know which one to interpret the result. Static programming languages can just return a value of the defined element type, but dynamic programming languages typically have heterogeneous collections. Features like multiple return values rely in this feature. These are actually tuples, which can be indexed or unpacked just like any other collection. Remembering length and the order of the elements of this tuple is paramount to avoiding false positives. 

The solution is similar to namespaces, but with additional abstractions to reflect the additional uncertainty. Not every element gets stored explicitly. If a list contains only objects of type `A`, it doesn't really matter which objects those are. Most collections are iterated over, so that all the elements have to be treated equally, and it doesn't really matter which instance of `A` gets used.
One or more _representants_ are chosen for each type of object in the collection. 
The order of elements can be important for a heterogeneous collections, which is why these can have multiple representants for the same type of element. 
Non-empty collection literals such as multiple return values are exceptions, every element does get stored explicitly for these. Chaining representants together can describe both ordered and unordered collections. Unordered collections do have an order after all, just not a reliable one which programmers should rely on. 

The components of a collection are introduced incrementally, in the same way the ones of the namespaces were introduced.

#### Representants 

The core components of a collection are the representants, in essence these are an alias for object pointers. The most important information of a representant is its type, so this has been added as well for the sake of usability. 

#### Chunks

Chunks represent contiguous regions within a collection. Their size is defined as an interval $[\,a, b\,]$ with $a \in [\,0,\infty\,[ \,\wedge\, b \in [\,1, \infty\,]$. Adding an element to a collection as as part of a loop will create a chunk with $b=\infty$ as we have no knowledge of how many times a loop gets executed. Chunks also contain representants. If `x` is either an `int` or a `list` for example, adding it to a collection will add a chunk with two representants. 

#### Chains

A sequence of chunks forms a chain. A notion of length is added here as well. Insertion will either replace an existing element or a new one, so that only the upper bound is affected. The same insertion will however decrement the lower bound of all existing chunks, while simultaneously incrementing their upper bounds. Most of the collection logic is implemented here, such as indexing and slicing.

#### Branches

Branches form the next layer and they are similar to the ones that are part of namespaces. A branch in the context of a namespace get initialized as an empty `HashMap<String, Mapping>`, and only stores changes. This sparse structure is possible because with namespaces you know exactly which elements have been changed -- they have a unique name. Collections don't have this luxury. Unless we absolutely know which element was changed, we have to assume the entire structure has been changed. This is why a collection's `Branch` contains a list of `(Path, Chain)` tuples, where every `Chain` is an entire "collection" in its own right.

#### Frames

A collection also has frames, which are similar to the ones in namespaces as well. The main difference is that collections currently don't have a notion of statis chambers. These can certainly be added to improve accuracy, but they are less impactful for collections as they would only have a noticeable benefit when analyzing heterogeneous collections together with broken control flow. A `Frame` contains a list of branches, the path node that caused the branching, and the index of the active branch. 

#### Collections

Collections are stacks of frames, just like namespaces. They also have a grow and a merge operation, but the implementations are slightly different. Growing still creates a new frame for every branch point, but it will copy the previously active branch's content into every branch of the new frame. Merging is done naively: the last frame gets popped, the paths of its content get augmented with the frame's cause node and then replace the contents of the branch that is now active. A different approach could try to merge branches to the chunk level to avoid data redundancies, but this is a considerable challenge to implement. It might become necessary for large and complex files, but the current solution suffices for now.

### Operations

There are a few different ways an element can be added to a collection. The most fundamental way is simply by definition, as a list of mappings. In this case, a single chunk gets added for each mapping -- every given element is its own representant in this case for optimal accuracy, and every chunk has a known size of exactly 1. Inserting an element is the most complex operation. Any chunk that has only a single representant of the same type as the new element can have its upper bound incremented. All other chunks have their minimum size decremented -- to reflect that one of its elements may have been replaced. A new chunk of size $[\,0, 1\,]$ gets wedged in between chunks that haven't had their upper bounds incremented. The most common way to add something to a `list` in Python is through the `append` method, which is a much easier case. We can repeat the previous process, but only applied to the last chunk. We can either increment the upper bound of the existing chunk, or add a new chunk of size exactly 1. 

\begin{algorithm}
    \caption{Linearize Collection}\label{alg:lin}
    \begin{algorithmic}[1]
      \Function{linearize} {n, chunks}
        \If {\texttt{chunks.is\_empty()}}
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

There are a few different ways to retrieve an element from a collection. If done through a dynamically calculated index, a mapping for every representant is returned. We can do better for static indices however. The collections contents get linearized first -- replacing the chain of chunks with a list of mappings as described in algorithm \ref{alg:lin}. This process reveals which possible elements there can be at that position. This is a powerful feature for several reasons. The first (or last) element of a collection can be given some special meaning, even if it's generally a bad idea, in which case retrieving the correct element is a good thing. More importantly, students often use explicit indices instead of unpacking when using multiple return values. A static analyzer might want to recommend using multiple return values instead, but it will only be able to do so if its analysis didn't stumble over the indices. The linearized representation can also be used to create slices of collections, which is a very _pythonic_ operation so interpreting it accurately is important. 

All indexing operations may return duplicates, because the merge operation isn't very thorough. This can be alleviated by only returning a single mapping for every unique representant. As long as we know that an object had been added, all the different different that may have happened are less important. 

## Function Definitions

A function definition creates a callable object and assigns the result to the given name. The object contains a closure, which in turn contains the function body as a block of AST nodes and the required argument definitions. Calling the closure will map the given arguments to the defined ones, as shown in section \ref{function-calls}. Once the arguments are in place it executes the function body. Return values get treated as assignments to an `___result` identifier, so the existing namespace logic can be reused. Anonymous functions have a similar implementation.

Python's own functions and modules are harder to implement. Depending on the interpreter they may not even be in Python, but in C for example. Although the Fosite interpreter is designed to accommodate multiple languages, it's made with dynamic languages in mind -- C is quite far out of scope. A solution is to implement function behavior in closures, which then get injected into the interpreter as callable objects. These internal functions don't contain a function body AST, but manipulate the interpreter state directly. 
This is a laborsome endeavor, but it does have a few upsides. Builtin functions such as `sorted` contain hundreds of lines of code -- and none of them are relevant to our analysis. Including implementation details in the paths can only confuse users, as these usually have no knowledge of the language internals. Giving a summarized, result is both more efficient and more useful. 

Modules are implemented as closures that can inject callable objects into the interpreter at runtime. This means that with some time and dedication, third party libraries can be easily added in the same way. 

## Function Calls

A few things have to be evaluated before a function call, the call target has to be evaluated first, then the arguments get evaluated from left to right. Evaluating the target will return a mapping which can contain a variable amount of pointers. Evaluating the call target can result in several different function objects, and we have to consider every possible case. These all have to be evaluated independently, which is why a frame can have a variable amount of subframes -- and why path nodes contain information about how many branches there are. 

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

        \State

        \Function{assign\_vararg} {gpos, gkw, arg, kwonly, vararg, kwarg}
          \If {\texttt{vararg.is\_some()}}
            \State $\texttt{arg\_list} \gets \texttt{abstract\_list(gpos)}$
            \State $\texttt{assign(vararg.name, arg\_list)}$
          \EndIf
          \State $\texttt{assign\_kw(gkw, arg ++ kwonly, kwarg)}$
        \EndFunction

        \State

        \Function{assign\_kw} {gkw, arg, kwarg}
          \If {\texttt{(arg.names $\cap$ gkw.names).len() > 0}}
            \State $\texttt{name} \gets (\texttt{arg.names} \cap \texttt{gkw.names})\texttt{.pick\_one()}$
            \State $\texttt{assign(name, gkw[name])}$
            \State $\texttt{assign\_kw(gkw $\setminus$ name, arg $\setminus$ name, kwarg)}$
          \Else
            \State $\texttt{assign\_kwarg(gkw, arg, kwarg)}$
          \EndIf
        \EndFunction

        \State

        \Function{assign\_kwarg} {gkw, arg, kwarg}
          \If {\texttt{kwarg.is\_some() }}
            \State $\texttt{arg\_dict} \gets \texttt{abstract\_dict(gkw)}$
            \State $\texttt{assign(kwarg.name, arg\_dict)}$
          \EndIf

          \State $\texttt{assign\_arg\_default(arg)}$
        \EndFunction

        \State

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

Interpreting recursive functions requires a way to _tie the knot_ -- so that analysis doesn't get stuck in an endless loop. The interpreter maintains a call stack, and only permits the same function to be called a set amount of times. An execution branch that would perform a recursive call that exceeds the recursion depth is simply terminated, so that only the branches that result in a base case get executed at this depth. A call at a higher recursion depth will then use this value in its analysis of the entire function body. This method's accuracy is sufficient for most use cases, even for low recursion depth limits @interpret. 

## Warnings and Errors

The interpreter itself can recognize some interesting things. If the interpreter encounters something it cannot process, the actual runtimes probably won't be able to either. In this case the interpreter will emit an error, but with more information about the conditions that lead to the error. The result is similar to printing a call stack, but with even more detail.

It might also encounter error prone patterns during execution. If execution does fail, the warnings' information will add to the errors'  to paint an even more accurate picture. Even if execution doesn't fail, some of the warnings might indicate a deeper logical flaw. 

### Errors

Python doesn't always validate the input to the builtin functions. The `sum` function will just error with `TypeError: unsupported operand type(s) for +` if the argument wasn't an iterable containing things that can be summed together. We can be more helpful in our errors by describing which elements can't be added together and how they got there, perhaps even hinting towards implementing `__add__` and `__radd__`. Similar errors can be emitted when trying to incorrectly apply an operator to an object, such as adding an `int` and a `str` or trying to insert an element into a `str`. 

Uninitialized variables are some of the most common bugs in many programming languages. Badly written code can make these hard to track down. The way our interpreter is structured gives us all the needed information to provide helpful errors. 

Every branch of a collection knows its own largest possible length. This length can be used to compare to statically known indices, to make sure there are at least enough elements in the collection. This can be quite useful as students often try to get the first or last element out of a empty list.

### Warnings

A variable can point to objects of different types. With the exception of the numeric types, this makes the variable very unwieldy to use. This can be checked after merging a namespace by simply looking at the contents, and looking up the types of the results.

Heterogeneous collections are hard to deal with; unless there's a fixed structure to the collection, operations on its elements have to support all of the possible types. A collection keeps track of its elements' types. Adding something new to a collection will compare the types before and after adding, and will emit a warning if something's changed. 

Changing a collection while iterating over its contents is generally a bad idea. Python will not throw an error when this happens, which can lead to weird behavior such as skipping elements, endless loops, and collections consuming all available memory. This can be checked using the watches introduced in section \ref{loops}.

A while loop whose condition does not change after an iteration is prone to endless loops. This is the case if all the variables in the condition still point to the same object, and all those objects haven't been changed either. This can be done using watches as well.

A function call that did not end with an explicit return statement will return a `Ǹone` value. This can lead to confusing errors when a user forgot to return in just one of its branches. Since return values are treated as regular identifiers, we can use the existing logic provided by the namespaces to see which `OptionalMapping`s are still uninitialized at the of a function call. 

# Results and Discussion

The ultimate goal was helping students learn faster, but testing this would require a semester-long A/B test. We can analyze old submissions, and the Dodona platform has over 800,000 of them, but the results would need manual verification to weed out false positives. In fact, most students work incrementally, implementing the required functionality piece by piece. Pointing out that they haven't implemented some functions is ultimately useless, since they were aware of that already. 

Seven interesting submissions have been selected instead, from three different students. That may be a small selection of students, but even helping a single student overcome the difficulties of programming is a worthwhile effort. At least some of the submissions stood out because they evaluating them was terminated because the time limited was exceeded. This can mean two things: the submission was too inefficient, or there's an endless loop in the implementation. Students have expressed that they want different error messages for the two cases, and just referring them to the halting problem isn't very helpful. The next best thing is recognizing patterns that could lead to endless loops, which is exactly what the Fosite analysis tool is able to do. 

As the submissions tend to be large and hard to read, a prior section will show analysis results on handcrafted trivial examples. These examples will also contain the results of the data-flow analysis, as these results are too hard to interpret (and represent) for actual submissions.

\clearpage

## Artificial Examples

The analysis result is composed of four things: the analyzed code, the data dependencies, the data changes, and the feedback that's given during interpreting. There are two sorts of entries in the data-flow results as introduced in section \ref{approach}: identifiers and object pointers. Every line contains all the changes or dependencies of that line, in no particular order. 

The results of some statements are hard to express on a single line. 
Analysing a conditional statement is done in multiple steps -- the test gets evaluated before evaluating the two branches. These steps are separated in the data-flow representation using a `|` character and indentation. Everything left of the `|` is part of the test, everything to the right is part of the body. As mentioned in section \ref{approach}, the result is hierarchical; the sum of all changes and dependencies of a conditional body is placed to those of the test.
Functions are expressed in a similar way. Everything to the left of the `|` corresponds to the definition of the function. Its only change is usually the function name it introduces, and it may have dependencies to any variables the definition might capture as described in \ref{function-definitions}. The right hand side of the `|` corresponds to the changes and dependencies of a function call. The function call itself copies the sum of those results, with the exception that all identifier information is discarded as they're only relevant within that function's scope.

To keep the examples small, placeholder strings are used instead of actual conditions. Non-empty strings always evaluate to `True` in Python, but the analyzer currently evaluates all strings to `Maybe`. Builtin objects in Python can't be given any new attributes either, but it's more convenient to allow this in the analyzer -- for now at least -- as well.

### Assignments

\input{results/double_assign}

\Cref{lst:assign} illustrates how assignments are analyzed. Line 4 resolves the name `y` to some object, and assigns `7` to the attribute with name `attr` in the object's namespace. The changes show that `y` points to the object at address 36 at the time. Line 7 resolves `y` as well, and `attr` is resolved in the resulting objects' namespaces. We can see that line 5 created a new object at address 40. Not shown here are the objects 37-39, which are the `'cond1'` string, its boolean value, and the `7` that is used on line 4. Line 7 depends on both the `y` and `y.attr` identifiers, as well as the two possible objects `y` might refer to. This might seem like duplicate information, but two different identifiers can point to the same object, and can thus change its internal state as well as shown in \cref{lst:aliasing}. 

The analysis reveals that `y.attr` will never exist upon executing line 7 as the object that does have an `attr` attributed is no longer reachable. 


### Aliasing

\input{results/aliasing}

Both `x` and `y` point to the same object in \cref{lst:aliasing}, so that the resolution of `x.attr` on line 7 depends on the the assignment to `y.attr` on line 5. Both `x` and `y` point to object 36, but the paths in the mappings are different -- the mapping for `x` also has a node for the assignment to `y`. There error message on line 7 presents this information in a human-readable form.

### Call Clobbering

\input{results/clobbering}

\Cref{approach} mentioned that we'd like an interprocedural analysis, unlike the intraprocedural analysis that's part of PyPy. Call clobbering is one of the challenges to achieve this. The test on line 7 in \cref{lst:clobbering} calls `foo` with argument `y`. Line 4 assigns `'changed'` to attribute `attr` of the given argument. The call to `foo` has side-effects, which can be seen in the data-flow analysis on lines 3 and 7. It's the test of the conditional on line 7 that has the side-effects, there is a change to the left of the `|`. The `print` function itself has side-effects as well, which gets represented using an internal state object -- which happens to be on position 5. 

### Path Exclusion

\input{results/path_exclusion}

\Cref{conditionals} introduced the concept of path exclusion as a means to support simple type checks and other forms of input validation. \Cref{lst:path_exclusion} gives an example of this principle in action. The definition of `foo` includes an optional argument `x`. This argument should probably be a list, in order to evaluate lines 2 and 7. Default values should absolutely be immutable in Python because subsequent calls to the same function use the same default values. The usual solution is to have a default value of `None`, and to initialize it at the start of the function with a fresh empty list. The list concatenation on line 2 occurs before this initialization, so it results in an error. The one on line 7 is fine however, and the analysis recognizes it as such. Without path exclusion there would be two possible mappings for `x` after merging on line 4 -- but as the path where `x` is `None` cannot occur in the negative branch of the conditional, that mapping does not find its way back into the scope after merging.

\input{results/break}

### Control Flow 

Breaking out of a loop hides all the changes made in that branch for the duration of the loop, but those changes become visible again after the loop. \Cref{lst:break1} contains an example where this is important. The conditional on line 2 determines the type of `x` after the loop, but is completely irrelevant in the loop itself. Line 9 might cause an error if the condition on line 2 was true. More interesting is that it also finds uninitialized variables, as explained in the statis chamber subsection of \cref{namespace}. 

\ 

\ 

### Endless Loops

\input{results/endless_loop}

\Cref{lst:endless1} contains an example of the most trivial cause of endless loops: forgetting to update the loop condition. The code itself is a rough implementation of the exponentiation by squaring algorithm. It computes $\texttt{x}^\texttt{n}$, and uses the binary representation of `n`. This is done using bitwise shifts (or division by 2) until there are no more bits left. This halting criterion translates to the condition on line 7. If the condition on line 8 is true however, that iteration of the loop will not even affect the loop condition -- a clear case of an endless loop. The invariance of the loop condition is detected using the watches introduced in \cref{loops}.

\input{results/endless_loop2}

Quite often the loop condition will depend on some internal state of an object, rather than a variable mapping. The loop in \cref{lst:endless2} is harder to detect. There is a code path that does not alter the variable mapping nor the internal object state, which results in a warning. The other two execution paths aren't very helpful either though, even if they do alter the object state. This is where the invariance checks meet their limits. Other aspects of the analysis may still prove useful however, such as the type warning on line 8.

\input{results/endless_loop3}

While loops are often discouraged in favor of for loops, as the latter is less prone to implementation oversights. That doesn't mean they're entirely safe though, as the code in \cref{lst:endless3} illustrates. The code in the example will consume all available memory until the OS intervenes. There's also a subtle difference between the augmented assign `x=` and the `x` operator when applied to lists. The latter creates a new list by concatenating its two operands, while the former will extend the `lvalue` of the assignment. 
Python does not crash when changing a collection that's being iterated over. 

\ 

\ 

\ 

### Static Indexing

\input{results/indexing}

As discussed in \cref{collections}, static indexes enjoy special treatment. \Cref{lst:indexing} contains a possible type error on line 11. The second element of `x` gets added to the third element, but the second element is either an `int` or a `str` depending on the condition on line 1. The error message accurately describes this problem. The analysis did not suffer any loss of accuracy because of the collection. 

Notice how lines 8 and 9 depend on the identifier `foo` as well as the state of object `41`. The identifier dependency indicates which object the identifier should point to, the object dependencies indicate that those lines depend on the original contents of the collection -- as there have been no changes to object 41 yet.

#### Out-of-Bounds Index

\input{results/out_ouf_bounds}

\Cref{collections} discussed how every branch in the `Collection` struct contains its possible length as a size range. The upper bound can be used to compare to static indexes to detect out of bounds errors. Line 7 of \cref{lst:indexing2} tries to get the third element of `x`, but if `x` has the value it has been assigned to on line 2 it only has two values. Line 6 succeeds because `x` has either two or three elements, so getting the second one is safe. Lines 6 and 7 both depend on identifier `x`, as well as the two possible objects it might point to. 

\clearpage

### Unpacking and Slicing

\input{results/unpacking}

Slicing and unpacking collections are both closely related to indexing with static indexes. \Cref{lst:unpacking} contains an example with slicing and unpacking. Everything except for the first and last element of `foo` gets placed in a list called `b` on line 8. Line 9 then unpacks the two values and assigns to identifiers `d` and `e`. The analysis also recognizes when these operations will certainly fail, as they reuse the same logic as static indexing. 

## Submissions

### Submission 1

\input{results/submission2}

\clearpage

\Cref{sbm:sub1} contains a student's submission to one of the first assignments of the semester -- it's one of their first pieces of Python code. The problem in their submission is obvious, line 2 assigns a list of strings to `getal`, but its elements get used as an arguments to `cos` and `tan`. It takes some time to get used to the fact that not everything that looks like a number is actually a number. Python's error message isn't very helpful: `TypeError: a float is required` doesn't even mention _where_ the float is required. This student submitted the same code multiple times, and they even tried adding explicit float casts to remedy the error message. Perhaps the student wasn't aware that you can multiply a string with an integer in Python to duplicate the string, and ruled out the possibility that the `getal[i]` was to blame. In any case, Fosite provides the user with additional information to help with debugging. 

### Submission 2

\input{results/submission3}

\clearpage

Students sometimes shoot themselves in the foot such as in \cref{sbm:sub2}. The suspicious check on line 10 is a case of a student trying to game the system by hardcoding the result for one of the unit tests. They probably didn't expect that this would cause an endless loop though. Students don't usually start hardcoding the unit tests until they've given up out of frustration, at which point the last thing they'd want to encounter is an endless loop. The analysis in this case highlights the endless loop, but not the original cause of their frustration. It might help with their mental state though, which could in turn lead to solving it more efficiently. 

\clearpage

### Submission 3

\input{results/submission4}

It happens quite often that a student wants to check some condition in the body of a loop, but uses a `while` loop instead of a simple `if` statement. This makes some sense since the condition should indeed get checked multiple times -- but that's what the outer loop is for. \Cref{sbm:sub3} contains a submission where a student did exactly this. The loops on lines 6 and 12 were obviously not meant to be while loops. Oddly enough the checks on line 8 and 9 are mostly fine, which could mea that the student is close to realizing which language construct was the right choice. 

Somewhere in the barely legible code, there's a more subtle mistake. The `==` operator can compare objects of any type, a type check is probably even the first thing it does. In the case of the condition on line 13, this causes the check to be trivially false -- a string is being compared to a method. The student simply forgot to call the method, which is a mistake that easily goes unnoticed. Fosite detects this, and gives an outright error that two incompatible types are being compared. This makes Fosite more strict than Python, in the same way that Google's Error Prone analyzer is more strict than Java (see \cref{smp:shortset}). In fact, there's little reason to ever use the `==` operator on callable objects. 

### Submission 4

\input{results/submission5}

\clearpage

Not all endless loops are caused because the loop condition never gets updated, some are even caused by type errors such as in \cref{sbm:sub4}. Using the syntax introduced in Python 3.5 for type hints, the `volgende` function returns an object of type `Tuple[int]`. The loop that starts on line 12 should add additional `Tuple[int]` values to the `ducci` variable until the value that would get added is already part of the collection. Line 13 does not add `Tuple[int]` values to the collection though -- it adds every `int` separately. Using the augmented `+=` assignment operator on a list is equivalent to calling the `extend` method -- and not the `append` method that the student wanted. No `Tuple[int]` gets added, and the loop never terminates. The endless loop itself is hard to recognize, but Fosite does warn when addings elements of a new type to a collection. Personal correspondance has shown that giving this warning along with the difference between appending to a collection and extending a collection would've been enough for the student to fix their mistake themselves.

### Submission 5

\input{results/submission6}

Executing `import this` in Python prints out "The Zen of Python" which is an informal Python style guide. Number two on that list is \say{Explicit is better than implicit}, but oddly enough truthiness is considered to be _pythonic_. The truthiness of a value is its boolean value, and in practice this is equivalent to an implicit `bool` cast. This is quite error prone as \cref{sbm:sub5} illustrates. The condition on line 3 is always true, since `'-'` is never empty. Fosite does not consider `str` values to be usable in boolean operations. They can used as standalone boolean expressions however, so that truthiness can still be used for input validation. 

### Submission 6

\input{results/submission7}

Reusing the same identifiers can have unexpected consequences, such as in \cref{sbm:sub6}. The `for` loop on line 8 redefines `x`, while the condition on line 9 was intended to use the old value. This actually means that the test on line 9 immediatly returns false, so that the loop is never executed, and same thing happens to the next loop. In effect, the iterations of the loop on line 5 don't change anything. Unfortunately Fosite doesn't know that the tests on line 9 and 13 are trivially false. This means that it doesn't find the most urgent problem, but it does find another problem -- line 11 should have an additional `float` cast. 

### Submission 7

\input{results/submission1}

\Cref{sbm:sub7} is about as large of a submission as one can comfortably put on paper. It also served as a performance stress test, as the analysis that Fosite does has an exponential complexity. On top of that it contains all the non-trivial things, such as default arguments, function objects, list concatenation, and loops. Evaluating this submission takes about 60 ms on an 2.2 GHz i5-5200U. Only 20 ms of which is spent in the analysis itself -- the rest is spent in parsing and transforming the input. 

\clearpage

Apart from the curious check on line 30, there are some interesting mistakes in the submission as well. It's possible that none of the return statements in the `radar` function ever get executed, perhaps because the check on line 42 should get swapped with the `else` on line 40. Line 89 is interesting because students write `==` instead of `=` more often than one might expect. Reusing the same identifier actually helps the student here, as line 90 would result in an type error during normal execution. The results aren't always as benign, but the boolean operation is usually nonsensical, so that Fosite is able to help. The other cases such as the loop on line 92 are even more problematic, as line 95 causes an endless loop.

# Conclusion and Future Work

We can successfully recognize some common, but non-trivial, mistakes that students make. The error messages describe the circumstances in which a problem may arise so that users are more likely to agree with the analysis. There are other interesting patterns the analysis currently doesn't recognize, even though they could prove useful. Unused variables for example can be an indication that the user forgot to write a few statements. The rationale behind JSLint is that experience has shown that some patterns lead to errors -- and experiences should be the driving force behind expanding Fosite's feature set as well. 

The Rust programming language is difficult to learn, and most people who do learn it do so in their free time. The author of Rust's linter tool Clippy is one of the people trying to make it more accessible to newcomers. In a blog post [^1] he discusses the different ways people learn to program. One of which is just by trial-and-error, and he has several interesting proposals to make this process easier. The results have shown that students sometimes struggle with testing conditions in loops -- where they use a separate `while` loop in the outer loop instead of a simple `if` statement. The current analysis warns about the potential endless loops it may cause, but it would be even better if it also explains what the current code does, proposes a different solution, and explains the difference between the two versions. The goal is to have an automated system that provides feedback in the same way an educator would -- by also taking into account the context in which an error occurs. 

[^1]: http://manishearth.github.io/blog/2017/05/19/teaching-programming-proactive-vs-reactive/

Abstract interpreters have already been used to perform an aliasing analysis to optimize dynamic languages @interpret. They accredit their succes to three characteristics of their analysis:

\clearpage

 * \textit{Flow-Sensitivity}: The order of the statements are taken into account. 
 * \textit{Type sensitivity}: Through the use of an abstract interpreter and their own typing system. 
 * \textit{Context sensitivity}: Distinguishing between different calling contexts. They achieve this using inlining, which is equivalent to the function calls we perform. 

Our own analysis shares all these characteristics, mostly through the use of an abstract interpreter, while also being inherently path-sensitive. This gives us confidence that our own data-flow analysis can be successfully applied to code transformations as well. The analysis is done in a matter of milliseconds on even large submissions, which leaves a lot of room to work with. Additional analysis features might include detecting dead code or simplifying nested code branches.

Namespaces and heterogeneous collections are accurately modeled in an path-sensitive fashion, but all branch points are currently presumed to be independent. Adding symbolic execution may significantly improve the path-sensitivity. This is expected to be a challenging endeavour, not just theoretically but from a software architectural point as well. There are other architectural challenges, such as adding support for more languages and third-party libraries. 


# References