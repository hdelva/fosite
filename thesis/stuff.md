A powerful languages is thus a language where things of interest are at least perceived to be easy. As we'll discuss in the following sections; this perception is often deceptive. 

There are developers on the other side of the spectrum however, who have a more stoic approach to programming. These people value reliability above all else. More reliable code is in turn easier to reason about and easier to collaborate on with other people. These languages often have a sound theoretical foundation, such as the lambda calculus. But unfortunately, unless you intend on using those languages for mathematical work, they can get stuck in the Turing tar-pit. 

Both sides raise valid points, so which is the correct way to program? Recent development seem to point towards finding a middle ground. Many languages that could be considered to be on the powerful side of the spectrum are moving towards the center of the spectrum. C# has been under heavy influence of Haskell @csharp, Scala emerged as a more strict alternative to Java and in turn influenced Java's development. New languages have been created in search of the sweet spot of the spectrum. Erlang was created at Ericsson to make their telecommunications backend more fault-tolerant @erlang. Go was made by Google to be an efficient programming language, that's easy to work with and that has reliable concurrency @go. Rust was created by Mozilla because they needed a more reliable alternative to C to power their new Servo rendering engine which makes extensive use of concurrency @rust. Many of these new languages come with considerable limitations but are undoubtedly powerful — they're all created to serve a real and non-trivial goal. 

Some languages are left behind in this recent trend towards reliability however. They are stuck with what they do because of backwards compatibility. This doesn't mean it's impossible to write reliable code in the languages however. Static analysis tools have been around ever since developers wanted more insurances about their code. One class of programming languages that is notoriously hard to analyze statically are the dynamic languages. Put optimistically, these languages are just too powerful. 

In the following sections we'll illustrate why powerful languages aren't always in the best interest of a developer. After that we'll look at some _best practices_ of programming and how some languages enforce them. Finally we'll look at what static analyzers are able to achieve for certain programming languages.

Programming languages have a lot in common with regular products from regular companies. The success of a programming languages relies on public perception. The way a programming language is presented directly influences how many will even consider using it. One effective selling point for a programming language is how powerful they can make a developer feel. These are usually the programming languages that don't get in the way of the developers. Since productivity is hard to measure, giving the feeling of being productive is a powerful thing. But from a fundamental point of view, our entire field of research is based on logic -- surely we can do better than that.



This analogy goes further: programming languages have their own priorities — aside from what developers actually want.

## Self-Imposed Strictness


 Something that plagues our field of work — whether academic of professional — is that developers seem to overestimate their own grasp of a programming language. Why this is the case is probably a subject of psychology instead of computer science. 

Douglas Crockford postulates that humans can approach a problem from two ways: either using their head or their gut @crockford. The head is obviously the most reliable way but it's also the slowest way. Following your gut on the other hand is fast but error-prone. Ideally we a professional developer would only follow his head but realistically, the developers are under time constraints as well. So despite knowing better, we fall back on our gut. Our gut is also notoriously bad at following good programming practices. We all know we should document our code, that we should use clear variable names, avoid certain anti-patterns, ... But we're all sinners in this regard. And to cope with the guilt we make up excuses: "we know what we're doing" or "nobody else is going to read this anyway'. 

And herein lies the danger, we trivialize certain bad practices for the sake of productivity.