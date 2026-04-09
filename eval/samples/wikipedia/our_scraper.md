From Wikipedia, the free encyclopedia

General-purpose programming language

Rust

[Paradigms](https://en.wikipedia.org/wiki/Programming_paradigm)
- [Concurrent](https://en.wikipedia.org/wiki/Concurrent_computing)
- [functional](https://en.wikipedia.org/wiki/Functional_programming)
- [generic](https://en.wikipedia.org/wiki/Generic_programming)
- [imperative](https://en.wikipedia.org/wiki/Imperative_programming)
- [structured](https://en.wikipedia.org/wiki/Structured_programming)

[Developer](https://en.wikipedia.org/wiki/Software_developer) The Rust Team
First appeared January 19, 2012 ; 14 years ago ( 2012-01-19 )

[Stable release](https://en.wikipedia.org/wiki/Software_release_life_cycle)
1.94.1 [1] / March 26, 2026 ; 12 days ago ( March 26, 2026 )

[Typing discipline](https://en.wikipedia.org/wiki/Type_system)
- [Affine](https://en.wikipedia.org/wiki/Affine_type_system)
- [inferred](https://en.wikipedia.org/wiki/Type_inference)
- [nominal](https://en.wikipedia.org/wiki/Nominal_type_system)
- [static](https://en.wikipedia.org/wiki/Static_typing)
- [strong](https://en.wikipedia.org/wiki/Strong_and_weak_typing)

Implementation language [OCaml](https://en.wikipedia.org/wiki/OCaml) (2006–2011)
Rust (2012–present)
[Platform](https://en.wikipedia.org/wiki/Computing_platform) [Cross-platform](https://en.wikipedia.org/wiki/Cross-platform_software) [note 1]
[OS](https://en.wikipedia.org/wiki/Operating_system) [Cross-platform](https://en.wikipedia.org/wiki/Cross-platform_software) [note 2]
[License](https://en.wikipedia.org/wiki/Software_license) [MIT](https://en.wikipedia.org/wiki/MIT_License) , [Apache 2.0](https://en.wikipedia.org/wiki/Apache_License) [note 3]
[Filename extensions](https://en.wikipedia.org/wiki/Filename_extension) `.rs `, `.rlib `
Website [rust-lang.org](https://www.rust-lang.org/)
Influenced by

- [Alef](https://en.wikipedia.org/wiki/Alef_(programming_language))
- [BETA](https://en.wikipedia.org/wiki/BETA_(programming_language))
- [CLU](https://en.wikipedia.org/wiki/CLU_(programming_language))
- [C#](https://en.wikipedia.org/wiki/C_Sharp_(programming_language))
- [C++](https://en.wikipedia.org/wiki/C%2B%2B)
- [Cyclone](https://en.wikipedia.org/wiki/Cyclone_(programming_language))
- [Elm](https://en.wikipedia.org/wiki/Elm_(programming_language))
- [Erlang](https://en.wikipedia.org/wiki/Erlang_(programming_language))
- [Haskell](https://en.wikipedia.org/wiki/Haskell)
- [Hermes](https://en.wikipedia.org/wiki/Hermes_(programming_language))
- [Limbo](https://en.wikipedia.org/wiki/Limbo_(programming_language))
- [Mesa](https://en.wikipedia.org/wiki/Mesa_(programming_language))
- [Napier](https://en.wikipedia.org/wiki/Napier88)
- [Newsqueak](https://en.wikipedia.org/wiki/Newsqueak)
- [NIL](https://en.wikipedia.org/wiki/Typestate_analysis) [note 4]
- [OCaml](https://en.wikipedia.org/wiki/OCaml)
- [Ruby](https://en.wikipedia.org/wiki/Ruby_(programming_language))
- [Sather](https://en.wikipedia.org/wiki/Sather)
- [Scheme](https://en.wikipedia.org/wiki/Scheme_(programming_language))
- [Standard ML](https://en.wikipedia.org/wiki/Standard_ML)
- [Swift](https://en.wikipedia.org/wiki/Swift_(programming_language)) [7] [8]

Influenced

- [Idris](https://en.wikipedia.org/wiki/Idris_(programming_language)) [9]
- [Project Verona](https://en.wikipedia.org/wiki/Project_Verona) [10]
- [SPARK](https://en.wikipedia.org/wiki/SPARK_(programming_language)) [11]
- [Swift](https://en.wikipedia.org/wiki/Swift_(programming_language)) [12]
- [V](https://en.wikipedia.org/wiki/V_(programming_language)) [13]
- [Zig](https://en.wikipedia.org/wiki/Zig_(programming_language)) [14]
- [Gleam](https://en.wikipedia.org/wiki/Gleam_(programming_language)) [15]

**Rust ** is a [general-purpose](https://en.wikipedia.org/wiki/General-purpose_programming_language) [programming language](https://en.wikipedia.org/wiki/Programming_language) . It is noted for its emphasis on [performance](https://en.wikipedia.org/wiki/Computer_performance) , [type safety](https://en.wikipedia.org/wiki/Type_safety) , [concurrency](https://en.wikipedia.org/wiki/Concurrency_(computer_science)) , and [memory safety](https://en.wikipedia.org/wiki/Memory_safety) .

Rust supports multiple [programming paradigms](https://en.wikipedia.org/wiki/Programming_paradigm) . It was influenced by ideas from [functional programming](https://en.wikipedia.org/wiki/Functional_programming) , including [immutability](https://en.wikipedia.org/wiki/Immutable_object) , [higher-order functions](https://en.wikipedia.org/wiki/Higher-order_function) , [algebraic data types](https://en.wikipedia.org/wiki/Algebraic_data_type) , and [pattern matching](https://en.wikipedia.org/wiki/Pattern_matching) . It also supports [object-oriented programming](https://en.wikipedia.org/wiki/Object-oriented_programming) via structs, [enums](https://en.wikipedia.org/wiki/Union_type) , traits, and methods. Rust is noted for enforcing memory safety (i.e., that all [references](https://en.wikipedia.org/wiki/Reference_(computer_science)) point to valid memory) without a conventional [garbage collector](https://en.wikipedia.org/wiki/Garbage_collection_(computer_science)) ; instead, memory safety errors and [data races](https://en.wikipedia.org/wiki/Data_race) are prevented by the "borrow checker", which tracks the [object lifetime](https://en.wikipedia.org/wiki/Object_lifetime) of references [at compile time](https://en.wikipedia.org/wiki/Compiler) .

Software developer Graydon Hoare created Rust in 2006 while working at [Mozilla](https://en.wikipedia.org/wiki/Mozilla) , which officially sponsored the project in 2009. The first stable release, Rust 1.0, was published in May 2015. Following a layoff of Mozilla employees in August 2020, four other companies joined Mozilla in sponsoring Rust through the creation of the Rust Foundation in February 2021.

Rust has been adopted by many software projects, especially [web services](https://en.wikipedia.org/wiki/Web_service) and [system software](https://en.wikipedia.org/wiki/System_software) . It has been studied academically and has a growing community of developers.

## History

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=1) ]

### 2006–2009: Early years

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=2) ]
Mozilla Foundation headquarters, 650 Castro Street in [Mountain View, California](https://en.wikipedia.org/wiki/Mountain_View,_California) , June 2009
Rust began as a personal project by [Mozilla](https://en.wikipedia.org/wiki/Mozilla) employee Graydon Hoare in 2006. According to _MIT Technology Review _ , he started the project due to his frustration with a broken elevator in his apartment building whose software had crashed, [16] and named the language after the [group of fungi of the same name](https://en.wikipedia.org/wiki/Rust_(fungus)) that is "over-engineered for survival". [16] During the time period between 2006 and 2009, Rust was not publicized to others at Mozilla and was written in Hoare's free time; [17] : 7:50 Hoare began speaking about the language around 2009 after a small group at Mozilla became interested in the project. [18] Hoare cited languages from the 1970s, 1980s, and 1990s as influences — including [CLU](https://en.wikipedia.org/wiki/CLU_(programming_language)) , [BETA](https://en.wikipedia.org/wiki/BETA_(programming_language)) , [Mesa](https://en.wikipedia.org/wiki/Mesa_(programming_language)) , NIL, [note 4] [Erlang](https://en.wikipedia.org/wiki/Erlang_(programming_language)) , [Newsqueak](https://en.wikipedia.org/wiki/Newsqueak) , [Napier](https://en.wikipedia.org/wiki/Napier88) , [Hermes](https://en.wikipedia.org/wiki/Hermes_(programming_language)) , [Sather](https://en.wikipedia.org/wiki/Sather) , [Alef](https://en.wikipedia.org/wiki/Alef_(programming_language)) , and [Limbo](https://en.wikipedia.org/wiki/Limbo_(programming_language)) . [18] He described the language as "technology from the past come to save the future from itself." [17] : 8:17 [18] Early Rust developer Manish Goregaokar similarly described Rust as being based on "mostly decades-old research." [16]

During the early years, the Rust [compiler](https://en.wikipedia.org/wiki/Compiler) was written in about 38,000 lines of [OCaml](https://en.wikipedia.org/wiki/OCaml) . [17] : 15:34 [19] Early Rust contained several features no longer present today, including explicit [object-oriented programming](https://en.wikipedia.org/wiki/Object-oriented_programming) via an `obj `keyword [17] : 10:08 and a [typestates](https://en.wikipedia.org/wiki/Typestate_analysis) system for variable state changes, such as going from uninitialized to initialized. [17] : 13:12

### 2009–2012: Mozilla sponsorship

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=3) ]

Mozilla officially sponsored the Rust project in 2009. [16] [Brendan Eich](https://en.wikipedia.org/wiki/Brendan_Eich) and other executives, intrigued by the possibility of using Rust for a safe [web browser](https://en.wikipedia.org/wiki/Web_browser) [engine](https://en.wikipedia.org/wiki/Browser_engine) , placed engineers on the project including Patrick Walton, Niko Matsakis, Felix Klock, and Manish Goregaokar. [16] A conference room taken by the project developers was dubbed "the nerd cave," with a sign placed outside the door. [16]

During this time period, work had shifted from the initial OCaml compiler to a [self-hosting compiler](https://en.wikipedia.org/wiki/Self-hosting_(compilers)) ( _i.e. _ , written in Rust) targeting [LLVM](https://en.wikipedia.org/wiki/LLVM) . [20] [note 5] The ownership system was in place by 2010. [16] The Rust logo was developed in 2011 based on a bicycle chainring. [22]

Rust 0.1 became the first public release on January 20, 2012 [23] for Windows, Linux, and MacOS. [24] The early 2010s witnessed increasing involvement from full-time engineers at Mozilla, open source volunteers outside Mozilla, and open source volunteers outside the United States. [16]

### 2012–2015: Evolution

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=4) ]

The years from 2012 to 2015 were marked by substantial changes to the Rust [type system](https://en.wikipedia.org/wiki/Type_system) . [17] : 18:36 [16] Memory management through the ownership system was gradually consolidated and expanded. By 2013, the [garbage collector](https://en.wikipedia.org/wiki/Garbage_collection_(computer_science)) was rarely used, and was removed in favor of the ownership system. [16] Other features were removed in order to simplify the language, including typestates, the `pure `keyword, [25] various specialized pointer types, and syntax support for [channels](https://en.wikipedia.org/wiki/Channel_(programming)) . [17] : 22:32

According to Steve Klabnik, Rust was influenced during this period by developers coming from [C++](https://en.wikipedia.org/wiki/C%2B%2B) (e.g., low-level performance of features), [scripting languages](https://en.wikipedia.org/wiki/Scripting_language) (e.g., Cargo and package management), and [functional programming](https://en.wikipedia.org/wiki/Functional_programming) (e.g., type systems development). [17] : 30:50

Graydon Hoare stepped down from Rust in 2013. [16] After Hoare's departure, it evolved organically under a federated governance structure, with a "core team" of initially six people, [17] : 21:45 and around 30-40 developers total across various other teams. [17] : 22:22 A [Request for Comments](https://en.wikipedia.org/wiki/Request_for_Comments) (RFC) process for new language features was added in March 2014. [17] : 33:47 The core team would grow to nine people by 2016 [17] : 21:45 with over 1600 RFCs. [17] : 34:08

According to Andrew Binstock for _[Dr. Dobb's Journal](https://en.wikipedia.org/wiki/Dr._Dobb%27s_Journal) _ in January 2014, while Rust was "widely viewed as a remarkably elegant language", adoption slowed because it radically changed from version to version. [26] Rust development at this time focused on finalizing features for version 1.0 so that it could begin promising [backward compatibility](https://en.wikipedia.org/wiki/Backward_compatibility) . [17] : 41:26

Six years after Mozilla's sponsorship, Rust 1.0 was published and became the first [stable release](https://en.wikipedia.org/wiki/Stable_release) on May 15, 2015. [16] A year later, the Rust compiler had accumulated over 1,400 contributors and there were over 5,000 third-party libraries published on the Rust package management website Crates.io. [17] : 43:15

### 2015–2020: Servo and early adoption

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=5) ]
Early homepage of Mozilla's [Servo browser engine](https://en.wikipedia.org/wiki/Servo_(software))
The development of the [Servo browser engine](https://en.wikipedia.org/wiki/Servo_(software)) continued in parallel with Rust, jointly funded by Mozilla and [Samsung](https://en.wikipedia.org/wiki/Samsung) . [27] The teams behind the two projects worked in close collaboration; new features in Rust were tested out by the Servo team, and new features in Servo were used to give feedback back to the Rust team. [17] : 5:41 The first version of Servo was released in 2016. [16] The [Firefox](https://en.wikipedia.org/wiki/Firefox) web browser shipped with Rust code as of 2016 (version 45), [17] : 53:30 [28] but components of Servo did not appear in Firefox until September 2017 (version 57) as part of the [Gecko](https://en.wikipedia.org/wiki/Gecko_(software)) and [Quantum](https://en.wikipedia.org/wiki/Gecko_(software)#Quantum) projects. [29]

Improvements were made to the Rust toolchain ecosystem during the years following 1.0 including Rustfmt , [integrated development environment](https://en.wikipedia.org/wiki/Integrated_development_environment) integration, [17] : 44:56 and a regular compiler testing and release cycle. [17] : 46:48 Rust's community gained a [code of conduct](https://en.wikipedia.org/wiki/Code_of_conduct) and an [IRC](https://en.wikipedia.org/wiki/IRC) chat for discussion. [17] : 50:36

The earliest known adoption outside of Mozilla was by individual projects at Samsung, [Facebook](https://en.wikipedia.org/wiki/Facebook) (now [Meta Platforms](https://en.wikipedia.org/wiki/Meta_Platforms) ), [Dropbox](https://en.wikipedia.org/wiki/Dropbox) , and Tilde, Inc., the company behind [ember.js](https://en.wikipedia.org/wiki/Ember.js) . [17] : 55:44 [16] [Amazon Web Services](https://en.wikipedia.org/wiki/Amazon_Web_Services) followed in 2020. [16] Engineers cited performance, lack of a garbage collector, safety, and pleasantness of working in the language as reasons for the adoption. Amazon developers cited a finding by Portuguese researchers that Rust code used [less energy](https://en.wikipedia.org/wiki/Energy_efficiency_in_computing) compared to similar code written in [Java](https://en.wikipedia.org/wiki/Java_(programming_language)) . [16] [30]

### 2020–present: Mozilla layoffs and Rust Foundation

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=6) ]

In August 2020, Mozilla laid off 250 of its 1,000 employees worldwide, as part of a corporate restructuring caused by the [COVID-19 pandemic](https://en.wikipedia.org/wiki/COVID-19_pandemic) . [31] [32] The team behind Servo was disbanded. The event raised concerns about the future of Rust. [33] In the following week, the Rust Core Team acknowledged the severe impact of the layoffs and announced that plans for a Rust foundation were underway. The first goal of the foundation would be to take ownership of all [trademarks](https://en.wikipedia.org/wiki/Trademark) and [domain names](https://en.wikipedia.org/wiki/Domain_name) and to take financial responsibility for their costs. [34]

On February 8, 2021, the formation of the Rust Foundation was announced by five founding companies: [Amazon Web Services](https://en.wikipedia.org/wiki/Amazon_Web_Services) , [Google](https://en.wikipedia.org/wiki/Google) , [Huawei](https://en.wikipedia.org/wiki/Huawei) , [Microsoft](https://en.wikipedia.org/wiki/Microsoft) , and [Mozilla](https://en.wikipedia.org/wiki/Mozilla) . [35] [36] The foundation would provide financial support for Rust developers in the form of grants and server funding. [16] In a blog post published on April 6, 2021, Google announced support for Rust within the [Android Open Source Project](https://en.wikipedia.org/wiki/Android_Open_Source_Project) as an alternative to C/C++. [37]

On November 22, 2021, the Moderation Team, which was responsible for enforcing the community code of conduct, announced their resignation "in protest of the Core Team placing themselves unaccountable to anyone but themselves". [38] In May 2022, members of the Rust leadership council posted a public response to the incident. [39]

The Rust Foundation posted a draft for a new trademark policy on April 6, 2023, which resulted in widespread negative reactions from Rust users and contributors. [40] The trademark policy included rules for how the Rust logo and name could be used. [40]

On February 26, 2024, the U.S. [White House](https://en.wikipedia.org/wiki/White_House) [Office of the National Cyber Director](https://en.wikipedia.org/wiki/Office_of_the_National_Cyber_Director) released a 19-page press report urging software development to move away from C and C++ to memory-safe languages like C#, Go, Java, Ruby, Swift, and Rust. [41] [42] [43]

## Syntax and features

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=7) ]

Main article: [Rust syntax](https://en.wikipedia.org/wiki/Rust_syntax)

Rust's [syntax](https://en.wikipedia.org/wiki/Syntax_(programming_languages)) is similar to that of [C](https://en.wikipedia.org/wiki/C_(programming_language)) and [C++](https://en.wikipedia.org/wiki/C%2B%2B) , [44] [45] although many of its features were influenced by [functional programming](https://en.wikipedia.org/wiki/Functional_programming) languages such as [OCaml](https://en.wikipedia.org/wiki/OCaml) . [46] Hoare has described Rust as targeted at frustrated C++ developers. [18]

### Hello World program

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=8) ]

Below is a ["Hello, World!" program](https://en.wikipedia.org/wiki/%22Hello,_World!%22_program) in Rust. The `fn `keyword denotes a [function](https://en.wikipedia.org/wiki/Function_(computer_programming)) , and the `println! `[macro](https://en.wikipedia.org/wiki/Macro_(computer_science)) (see § Macros ) prints the message to [standard output](https://en.wikipedia.org/wiki/Standard_output) . [47] [Statements](https://en.wikipedia.org/wiki/Statement_(computer_science)) in Rust are separated by [semicolons](https://en.wikipedia.org/wiki/Semicolon#Programming) .

```
fn main() {
    println!("Hello, World!");
}

```

### Variables

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=9) ]

[Variables](https://en.wikipedia.org/wiki/Variable_(computer_science)) in Rust are defined through the `let `keyword. [48] The example below assigns a value to the variable with name `foo `of type `i32 `and outputs its value; the type annotation `: i32 `can be omitted.

```
fn main() {
    let foo: i32 = 10;
    println!("The value of foo is {foo}");
}

```

Variables are [immutable](https://en.wikipedia.org/wiki/Immutable_object) by default, unless the `mut `keyword is added. [49] The following example uses `// `, which denotes the start of a [comment](https://en.wikipedia.org/wiki/Comment_(computer_programming)) . [50]

```
fn main() {
    // This code would not compile without adding "mut".
    let mut foo = 10; 
    println!("The value of foo is {foo}");
    foo = 20;
    println!("The value of foo is {foo}");
}

```

Multiple `let `expressions can define multiple variables with the same name, known as [variable shadowing](https://en.wikipedia.org/wiki/Variable_shadowing) . Variable shadowing allows transforming variables without having to name the variables differently. [51] The example below declares a new variable with the same name that is double the original value:

```
fn main() {
    let foo = 10;
    // This will output "The value of foo is 10"
    println!("The value of foo is {foo}");
    let foo = foo * 2;
    // This will output "The value of foo is 20"
    println!("The value of foo is {foo}");
}

```

Variable shadowing is also possible for values of different types. For example, going from a string to its length:

```
fn main() {
    let letters = "abc";
    let letters = letters.len();
}

```

### Block expressions and control flow

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=10) ]

A _block expression _ is delimited by [curly brackets](https://en.wikipedia.org/wiki/Bracket#Curly_brackets) . When the last expression inside a block does not end with a semicolon, the block evaluates to the value of that trailing expression: [52]

```
fn main() {
    let x = {
        println!("this is inside the block");
        1 + 2
    };
    println!("1 + 2 = {x}");
}

```

Trailing expressions of function bodies are used as the return value: [53]

```
fn add_two(x: i32) -> i32 {
    x + 2
}

```

#### if expressions

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=11) ]

An `if `[conditional expression](https://en.wikipedia.org/wiki/Conditional_expression) executes code based on whether the given value is `true `. `else `can be used for when the value evaluates to `false `, and `else if `can be used for combining multiple expressions. [54]

```
fn main() {
    let x = 10;
    if x > 5 {
        println!("value is greater than five");
    }

    if x % 7 == 0 {
        println!("value is divisible by 7");
    } else if x % 5 == 0 {
        println!("value is divisible by 5");
    } else {
        println!("value is not divisible by 7 or 5");
    }
}

```

`if `and `else `blocks can evaluate to a value, which can then be assigned to a variable: [54]

```
fn main() {
    let x = 10;
    let new_x = if x % 2 == 0 { x / 2 } else { 3 * x + 1 };
    println!("{new_x}");
}

```

#### while loops

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=12) ]

`[while](https://en.wikipedia.org/wiki/While_loop) `can be used to repeat a block of code while a condition is met. [55]

```
fn main() {
    // Iterate over all integers from 4 to 10
    let mut value = 4;
    while value <= 10 {
         println!("value = {value}");
         value += 1;
    }
}

```

#### for loops and iterators

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=13) ]

[For loops](https://en.wikipedia.org/wiki/For_loop) in Rust loop over elements of a collection. [56] `for `expressions work over any [iterator](https://en.wikipedia.org/wiki/Iterator) type.

```
fn main() {
    // Using `for` with range syntax for the same functionality as above
    // The syntax 4..=10 means the range from 4 to 10, up to and including 10.
    for value in 4..=10 {
        println!("value = {value}");
    }
}

```

In the above code, `4 ..= 10 `is a value of type `Range `which implements the `Iterator `trait. The code within the curly braces is applied to each element returned by the iterator.

Iterators can be combined with functions over iterators like `map `, `filter `, and `sum `. For example, the following adds up all numbers between 1 and 100 that are multiples of 3:

```
(1..=100).filter(|x| x % 3 == 0).sum()

```

#### loop and break statements

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=14) ]

More generally, the `loop `keyword allows repeating a portion of code until a `break `occurs. `break `may optionally exit the loop with a value. In the case of nested loops, labels denoted by `' label_name `can be used to break an outer loop rather than the innermost loop. [57]

```
fn main() {
    let value = 456;
    let mut x = 1;
    let y = loop {
        x *= 10;
        if x > value {
            break x / 10;
        }
    };
    println!("largest power of ten that is smaller than or equal to value: {y}");

    let mut up = 1;
    'outer: loop {
        let mut down = 120;
        loop {
            if up > 100 {
                break 'outer;
            }

            if down < 4 {
                break;
            }

            down /= 2;
            up += 1;
            println!("up: {up}, down: {down}");
        }
        up *= 2;
    }
}

```

### Pattern matching

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=15) ]

The `match `and `if let `expressions can be used for [pattern matching](https://en.wikipedia.org/wiki/Pattern_matching) . For example, `match `can be used to double an optional integer value if present, and return zero otherwise: [58]

```
fn double(x: Option<u64>) -> u64 {
    match x {
        Some(y) => y * 2,
        None => 0,
    }
}

```

Equivalently, this can be written with `if let `and `else `:

```
fn double(x: Option<u64>) -> u64 {
    if let Some(y) = x {
        y * 2
    } else {
        0
    }
}

```

### Types

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=16) ]

Rust is [strongly typed](https://en.wikipedia.org/wiki/Strongly_typed) and [statically typed](https://en.wikipedia.org/wiki/Statically_typed) , meaning that the types of all variables must be known at compilation time. Assigning a value of a particular type to a differently typed variable causes a [compilation error](https://en.wikipedia.org/wiki/Compilation_error) . [Type inference](https://en.wikipedia.org/wiki/Type_inference) is used to determine the type of variables if unspecified. [59]

The type `() `, called the "unit type" in Rust, is a concrete type that has exactly one value. It occupies no memory (as it represents the absence of value). All functions that do not have an indicated return type implicitly return `() `. It is similar to `void `in other C-style languages, however `void `denotes the absence of a type and cannot have any value.

The default integer type is `i32 `, and the default [floating point](https://en.wikipedia.org/wiki/Floating_point) type is `f64 `. If the type of a [literal](https://en.wikipedia.org/wiki/Literal_(computer_programming)) number is not explicitly provided, it is either inferred from the context or the default type is used. [60]

#### Primitive types

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=17) ]

[Integer types](https://en.wikipedia.org/wiki/Integer_type) in Rust are named based on the [signedness](https://en.wikipedia.org/wiki/Signedness) and the number of bits the type takes. For example, `i32 `is a signed integer that takes 32 bits of storage, whereas `u8 `is unsigned and only takes 8 bits of storage. `isize `and `usize `take storage depending on the [memory address bus width](https://en.wikipedia.org/wiki/Bus_(computing)#Address_bus) of the compilation target. For example, when building for [32-bit targets](https://en.wikipedia.org/wiki/32-bit_computing) , both types will take up 32 bits of space. [61] [62]

By default, integer literals are in base-10, but different [radices](https://en.wikipedia.org/wiki/Radix) are supported with prefixes, for example, `0b11 `for [binary numbers](https://en.wikipedia.org/wiki/Binary_number) , `0o567 `for [octals](https://en.wikipedia.org/wiki/Octal) , and `0xDB `for [hexadecimals](https://en.wikipedia.org/wiki/Hexadecimal) . By default, integer literals default to `i32 `as its type. Suffixes such as `4 u32 `can be used to explicitly set the type of a literal. [63] Byte literals such as `b'X' `are available to represent the [ASCII](https://en.wikipedia.org/wiki/ASCII) value (as a `u8 `) of a specific character. [64]

The [Boolean type](https://en.wikipedia.org/wiki/Boolean_type) is referred to as `bool `which can take a value of either `true `or `false `. A `char `takes up 32 bits of space and represents a Unicode scalar value: [65] a [Unicode codepoint](https://en.wikipedia.org/wiki/Unicode_codepoint) that is not a [surrogate](https://en.wikipedia.org/wiki/Universal_Character_Set_characters#Surrogates) . [66] [IEEE 754](https://en.wikipedia.org/wiki/IEEE_754) floating point numbers are supported with `f32 `for [single precision floats](https://en.wikipedia.org/wiki/Single_precision_float) and `f64 `for [double precision floats](https://en.wikipedia.org/wiki/Double_precision_float) . [67]

#### Compound types

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=18) ]

Compound types can contain multiple values. Tuples are fixed-size lists that can contain values whose types can be different. Arrays are fixed-size lists whose values are of the same type. Expressions of the tuple and array types can be written through listing the values, and can be accessed with `. index `(with tuples) or `[ index ] `(with arrays): [68]

```
let tuple: (u32, bool) = (3, true);
let array: [i8; 5] = [1, 2, 3, 4, 5];
let value = tuple.1; // true
let value = array[2]; // 3

```

Arrays can also be constructed through copying a single value a number of times: [69]

```
let array2: [char; 10] = [' '; 10];

```

### Ownership and references

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=19) ]

Rust's ownership system consists of rules that ensure memory safety without using a garbage collector. At compile time, each value must be attached to a variable called the _owner _ of that value, and every value must have exactly one owner. [70] Values are moved between different owners through assignment or passing a value as a function parameter. Values can also be _borrowed, _ meaning they are temporarily passed to a different function before being returned to the owner. [71] With these rules, Rust can prevent the creation and use of [dangling pointers](https://en.wikipedia.org/wiki/Dangling_pointers) : [71] [72]

```
fn print_string(s: String) {
    println!("{}", s);
}

fn main() {
    let s = String::from("Hello, World");
    print_string(s); // s consumed by print_string
    // s has been moved, so cannot be used any more
    // another print_string(s); would result in a compile error
}

```

The function `print_string `takes ownership over the `String `value passed in; Alternatively, `& `can be used to indicate a [reference](https://en.wikipedia.org/wiki/Reference_(computer_science)) type (in `& String `) and to create a reference (in `& s `): [73]

```
fn print_string(s: &String) {
    println!("{}", s);
}

fn main() {
    let s = String::from("Hello, World");
    print_string(&s); // s borrowed by print_string
    print_string(&s); // s has not been consumed; we can call the function many times
}

```

Because of these ownership rules, Rust types are known as _[affine types](https://en.wikipedia.org/wiki/Affine_type) _ , meaning each value may be used at most once. This enforces a form of [software fault isolation](https://en.wikipedia.org/wiki/Software_fault_isolation) as the owner of a value is solely responsible for its correctness and deallocation. [74]

When a value goes out of scope, it is _dropped _ by running its [destructor](https://en.wikipedia.org/wiki/Destructor_(computer_programming)) . The destructor may be programmatically defined through implementing the `Drop `trait . This helps manage resources such as file handles, network sockets, and [locks](https://en.wikipedia.org/wiki/Lock_(computer_science)) , since when objects are dropped, the resources associated with them are closed or released automatically. [75]

#### Lifetimes

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=20) ]

[Object lifetime](https://en.wikipedia.org/wiki/Object_lifetime) refers to the period of time during which a reference is valid; that is, the time between the object creation and destruction. [76] These _lifetimes _ are implicitly associated with all Rust reference types. While often inferred, they can also be indicated explicitly with named lifetime parameters (often denoted `' a `, `' b `, and so on). [77]

A value's lifetime in Rust can be thought of as [lexically scoped](https://en.wikipedia.org/wiki/Scope_(computer_science)) , meaning that the duration of an object lifetime is inferred from the set of locations in the source code (i.e., function, line, and column numbers) for which a variable is valid. [78] For example, a reference to a local variable has a lifetime from the expression it is declared in up until the last use of it. [78]

```
fn main() {
    let mut x = 5;            // ------------------+- Lifetime 'a
                              //                   |
    let r = &x;               // -+-- Lifetime 'b  |
                              //  |                |
    println!("r: {}", r);     // -+                |
    // Since r is no longer used,                  |
    // its lifetime ends                           |
    let r2 = &mut x;          // -+-- Lifetime 'c  |
}                             // ------------------+

```

The borrow checker in the Rust compiler then enforces that references are only used in the locations of the source code where the associated lifetime is valid. [79] [80] In the example above, storing a reference to variable `x `in `r `is valid, as variable `x `has a longer lifetime ( `' a `) than variable `r `( `' b `). However, when `x `has a shorter lifetime, the borrow checker would reject the program:

```
fn main() {
    let r;                    // ------------------+- Lifetime 'a
                              //                   |
    {                         //                   |
        let x = 5;            // -+-- Lifetime 'b  |
        r = &x; // ERROR: x does  |                |
    }           // not live long -|                |
                // enough                          |
    println!("r: {}", r);     //                   |
}                             // ------------------+

```

Since the lifetime of the referenced variable ( `' b `) is shorter than the lifetime of the variable holding the reference ( `' a `), the borrow checker errors, preventing `x `from being used from outside its scope. [81]

Lifetimes can be indicated using explicit _lifetime parameters _ on function arguments. For example, the following code specifies that the reference returned by the function has the same lifetime as `original `(and _not _ necessarily the same lifetime as `prefix `): [82]

```
fn remove_prefix<'a>(mut original: &'a str, prefix: &str) -> &'a str {
    if original.starts_with(prefix) {
        original = original[prefix.len()..];
    }
    original
}

```

In the compiler, ownership and lifetimes work together to prevent memory safety issues such as dangling pointers. [83] [84]

### User-defined types

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=21) ]

User-defined types are created with the `struct `or `enum `keywords. The `struct `keyword is used to denote a [record type](https://en.wikipedia.org/wiki/Record_(computer_science)) that groups multiple related values. [85] `enum `s can take on different variants at runtime, with its capabilities similar to [algebraic data types](https://en.wikipedia.org/wiki/Algebraic_data_types) found in functional programming languages. [86] Both records and enum variants can contain [fields](https://en.wikipedia.org/wiki/Field_(computer_science)) with different types. [87] Alternative names, or aliases, for the same type can be defined with the `type `keyword. [88]

The `impl `keyword can define methods for a user-defined type. Data and functions are defined separately. Implementations fulfill a role similar to that of [classes](https://en.wikipedia.org/wiki/Class_(programming)) within other languages. [89]

#### Standard library

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=22) ]
A diagram of the dependencies between the standard library modules of Rust
The Rust [standard library](https://en.wikipedia.org/wiki/Standard_library) defines and implements many widely used custom data types, including core data structures such as `Vec `, `Option `, and `HashMap `, as well as [smart pointer](https://en.wikipedia.org/wiki/Smart_pointer) types. Rust provides a way to exclude most of the standard library using the attribute `#![no_std] `, for applications such as embedded devices. Internally, the standard library is divided into three parts, `core `, `alloc `, and `std `, where `std `and `alloc `are excluded by `#![no_std] `. [90]

Rust uses the [option type](https://en.wikipedia.org/wiki/Option_type) `Option<T> `to define optional values, which can be matched using `if let `or `match `to access the inner value: [91]

```
fn main() {
    let name1: Option<&str> = None;
    // In this case, nothing will be printed out
    if let Some(name) = name1 {
        println!("{name}");
    }

    let name2: Option<&str> = Some("Matthew");
    // In this case, the word "Matthew" will be printed out
    if let Some(name) = name2 {
        println!("{name}");
    }
}

```

Similarly, Rust's [result type](https://en.wikipedia.org/wiki/Result_type) `Result<T, E> `holds either a successfully computed value (the `Ok `variant) or an error (the `Err `variant). [92] Like `Option `, the use of `Result `means that the inner value cannot be used directly; programmers must use a `match `expression, syntactic sugar such as `? `(the “try” operator), or an explicit `unwrap `assertion to access it. Both `Option `and `Result `are used throughout the standard library and are a fundamental part of Rust's explicit approach to handling errors and missing data.

### Pointers

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=23) ]

The `& `and `& mut `reference types are guaranteed to not be null and point to valid memory. [93] The raw pointer types `* const `and `* mut `opt out of the safety guarantees, thus they may be null or invalid; however, it is impossible to dereference them unless the code is explicitly declared unsafe through the use of an `unsafe `block. [94] Unlike dereferencing, the creation of raw pointers is allowed inside safe Rust code. [95]

### Type conversion

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=24) ]

This section is an excerpt from [Type conversion § Rust](https://en.wikipedia.org/wiki/Type_conversion#Rust) . [ [edit](https://en.wikipedia.org/w/index.php?title=Type_conversion&action=edit) ]

Rust provides no implicit type conversion (coercion) between most primitive types. But, explicit type conversion (casting) can be performed using the `as `keyword. [96]

```
let x: i32 = 1000;
println!("1000 as a u16 is: {}", x as u16);

```

A presentation on Rust by Emily Dunham from [Mozilla](https://en.wikipedia.org/wiki/Mozilla) 's Rust team ( [linux.conf.au](https://en.wikipedia.org/wiki/Linux.conf.au) conference, Hobart, 2017)

### Polymorphism

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=25) ]

Rust supports [polymorphism](https://en.wikipedia.org/wiki/Polymorphism_(computer_science)) through [traits](https://en.wikipedia.org/wiki/Trait_(computer_programming)) , [generic functions](https://en.wikipedia.org/wiki/Generic_function) , and [trait objects](https://en.wikipedia.org/wiki/Trait_object_(Rust)) . [97]

#### Traits

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=26) ]

Common behavior between types is declared using traits and `impl `blocks: [98]

```
trait Zero: Sized {
    fn zero() -> Self;
    fn is_zero(&self) -> bool
    where
        Self: PartialEq,
    {
        self == &Zero::zero()
    }
}

impl Zero for u32 {
    fn zero() -> u32 { 0 }
}

impl Zero for f32 {
    fn zero() -> Self { 0.0 }
}

```

The example above includes a method `is_zero `which provides a default implementation that may be overridden when implementing the trait. [98]

#### Generic functions

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=27) ]

A function can be made generic by adding type parameters inside angle brackets ( `< Num > `), which only allow types that implement the trait:

```
// zero is a generic function with one type parameter, Num
fn zero<Num: Zero>() -> Num {
    Num::zero()
}

fn main() {
    let a: u32 = zero();
    let b: f32 = zero();
    assert!(a.is_zero() && b.is_zero());
}

```

In the examples above, `Num : Zero `as well as `where Self : PartialEq `are trait bounds that constrain the type to only allow types that implement `Zero `or `PartialEq `. [98] Within a trait or impl, `Self `refers to the type that the code is implementing. [99]

Generics can be used in functions to allow implementing a behavior for different types without repeating the same code (see [bounded parametric polymorphism](https://en.wikipedia.org/wiki/Bounded_parametric_polymorphism) ). Generic functions can be written in relation to other generics, without knowing the actual type. [100]

#### Trait objects

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=28) ]

By default, traits use [static dispatch](https://en.wikipedia.org/wiki/Static_dispatch) : the compiler [monomorphizes](https://en.wikipedia.org/wiki/Monomorphization) the function for each concrete type instance, yielding performance equivalent to type-specific code at the cost of longer compile times and larger binaries. [101]

When the exact type is not known at compile time, Rust provides [trait objects](https://en.wikipedia.org/wiki/Dynamic_dispatch) `&dyn Trait `and `Box<dyn Trait> `. [102] Trait object calls use [dynamic dispatch](https://en.wikipedia.org/wiki/Dynamic_dispatch) via a lookup table; a trait object is a "fat pointer" carrying both a data pointer and a method table pointer. [101] This indirection adds a small runtime cost, but it keeps a single copy of the code and reduces binary size. Only "object-safe" traits are eligible to be used as trait objects. [103]

This approach is similar to [duck typing](https://en.wikipedia.org/wiki/Duck_typing) , where all data types that implement a given trait can be treated as functionally interchangeable. [104] The following example creates a list of objects where each object implements the `Display `trait:

```
use std::fmt::Display;

let v: Vec<Box<dyn Display>> = vec![
    Box::new(3),
    Box::new(5.0),
    Box::new("hi"),
];

for x in v {
    println!("{x}");
}

```

If an element in the list does not implement the `Display `trait, it will cause a compile-time error. [105]

### Memory management

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=29) ]

Rust does not use [garbage collection](https://en.wikipedia.org/wiki/Garbage_collection_(computer_science)) . Memory and other resources are instead managed through the "resource acquisition is initialization" convention, [106] with optional [reference counting](https://en.wikipedia.org/wiki/Reference_counting) . Rust provides deterministic management of resources, with very low [overhead](https://en.wikipedia.org/wiki/Overhead_(computing)) . [107] Values are [allocated on the stack](https://en.wikipedia.org/wiki/Stack-based_memory_allocation) by default, and all [dynamic allocations](https://en.wikipedia.org/wiki/Dynamic_allocation) must be explicit. [108]

The built-in reference types using the `& `symbol do not involve run-time reference counting. The safety and validity of the underlying pointers is verified at compile time, preventing [dangling pointers](https://en.wikipedia.org/wiki/Dangling_pointers) and other forms of [undefined behavior](https://en.wikipedia.org/wiki/Undefined_behavior) . [109] Rust's type system separates shared, [immutable](https://en.wikipedia.org/wiki/Immutable_object) references of the form `&T `from unique, mutable references of the form `&mut T `. A mutable reference can be coerced to an immutable reference, but not vice versa. [110]

### Unsafe

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=30) ]

Rust's memory safety checks (See #Safety ) may be circumvented through the use of `unsafe `blocks. This allows programmers to dereference arbitrary raw pointers, call external code, or perform other low-level functionality not allowed by safe Rust. [111] Some low-level functionality enabled in this way includes [volatile memory access](https://en.wikipedia.org/wiki/Volatile_(computer_programming)) , architecture-specific intrinsics, [type punning](https://en.wikipedia.org/wiki/Type_punning) , and inline assembly. [112]

Unsafe code is needed, for example, in the implementation of data structures. [113] A frequently cited example is that it is difficult or impossible to implement [doubly linked lists](https://en.wikipedia.org/wiki/Doubly_linked_list) in safe Rust. [114] [115] [116] [117]

Programmers using unsafe Rust are considered responsible for upholding Rust's memory and type safety requirements, for example, that no two mutable references exist pointing to the same location. [118] If programmers write code which violates these requirements, this results in [undefined behavior](https://en.wikipedia.org/wiki/Undefined_behavior) . [118] The Rust documentation includes a list of behavior considered undefined, including accessing dangling or misaligned pointers, or breaking the aliasing rules for references. [119]

### Macros

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=31) ]

Macros allow generation and transformation of Rust code to reduce repetition. Macros come in two forms, with _declarative macros _ defined through `macro_rules! `, and _procedural macros _ , which are defined in separate crates. [120] [121]

#### Declarative macros

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=32) ]

A declarative macro (also called a "macro by example") is a macro, defined using the `macro_rules! `keyword, that uses pattern matching to determine its expansion. [122] [123] Below is an example that sums over all its arguments:

```
macro_rules! sum {
    ( $initial:expr $(, $expr:expr )* $(,)? ) => {
        $initial $(+ $expr)*
    }
}

fn main() {
    let x = sum!(1, 2, 3);
    println!("{x}"); // prints 6
}

```

In this example, the macro named `sum `is defined using the form `macro_rules! sum { ``(...) => { ... } } `. The first part inside the parentheses of the definition, the macro pattern `( $initial:expr $(, $expr:expr )* $(,)? ) `specifies the structure of input it can take. Here, `$initial:expr `represents the first expression, while `$(, $expr:expr )* `means there can be zero or more additional comma-separated expressions after it. The trailing `$(,)? `allows the caller to optionally include a final comma without causing an error. The second part after the arrow `=> `describes what code will be generated when the macro is invoked. In this case, `$initial $(+ $expr)* `means that the generated code will start with the first expression, followed by a `+ `and each of the additional expressions in sequence. The `* `again means "repeat this pattern zero or more times". This means, when the macro is later called in line 8, as `sum!(1, 2, 3) `the macro will resolve to `1 + 2 + 3 `representing the addition of all of the passed expressions.

#### Procedural macros

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=33) ]

Procedural macros are Rust functions that run and modify the compiler's input [token](https://en.wikipedia.org/wiki/Token_(parser)) stream, before any other components are compiled. They are generally more flexible than declarative macros, but are more difficult to maintain due to their complexity. [124] [125]

Procedural macros come in three flavors:

- Function-like macros `custom!(...) `
- Derive macros `#[derive(CustomDerive)] `
- Attribute macros `#[custom_attribute] `

### Interface with C and C++

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=34) ]

Rust supports the creation of [foreign function interfaces](https://en.wikipedia.org/wiki/Foreign_function_interface) (FFI) through the `extern `keyword. A function that uses the C [calling convention](https://en.wikipedia.org/wiki/Calling_convention) can be written using `extern "C" fn `. Symbols can be exported from Rust to other languages through the `#[unsafe(no_mangle)] `attribute, and symbols can be imported into Rust through `extern `blocks: [note 6] [127]

```
#[unsafe(no_mangle)]
pub extern "C" fn exported_from_rust(x: i32) -> i32 { x + 1 }
unsafe extern "C" {
    fn imported_into_rust(x: i32) -> i32;
}

```

The `#[repr(C)] `attribute enables deterministic memory layouts for `struct `s and `enum `s for use across FFI boundaries. [127] External libraries such as `bindgen `and `cxx `can generate Rust bindings for C/C++. [127] [128]

## Safety

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=35) ]

[Safety properties](https://en.wikipedia.org/wiki/Safety_properties) guaranteed by Rust include [memory safety](https://en.wikipedia.org/wiki/Memory_safety) , [type safety](https://en.wikipedia.org/wiki/Type_safety) , and [data race](https://en.wikipedia.org/wiki/Data_race) freedom. As described above, these guarantees can be circumvented by using the `unsafe `keyword.

Memory safety includes the absence of dereferences to [null](https://en.wikipedia.org/wiki/Null_pointer) , [dangling](https://en.wikipedia.org/wiki/Dangling_pointer) , and misaligned [pointers](https://en.wikipedia.org/wiki/Pointer_(computer_programming)) , and the absence of [buffer overflows](https://en.wikipedia.org/wiki/Buffer_overflow) and [double free](https://en.wikipedia.org/wiki/Double_free) errors. [129] [130] [131] [132]

[Memory leaks](https://en.wikipedia.org/wiki/Memory_leak) are possible in safe Rust. [133] Memory leaks may occur as a result of creating reference counted pointers that point at each other (a reference cycle) [133] or can be deliberately created through calling `Box::leak `. [134]

## Ecosystem

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=36) ]
Compiling a Rust program with Cargo
The Rust ecosystem includes its compiler, its standard library , and additional components for software development. Component installation is typically managed by `rustup `, a Rust [toolchain](https://en.wikipedia.org/wiki/Toolchain) installer developed by the Rust project. [135]

### Compiler

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=37) ]

The Rust compiler, `rustc `, compiles Rust code into [binaries](https://en.wikipedia.org/wiki/Executable) . First, the compiler parses the source code into an [AST](https://en.wikipedia.org/wiki/Abstract_syntax_tree) . Next, this AST is lowered to [IR](https://en.wikipedia.org/wiki/Intermediate_representation) . The compiler backend is then invoked as a subcomponent to apply [optimizations](https://en.wikipedia.org/wiki/Optimizing_compiler) and translate the resulting IR into [object code](https://en.wikipedia.org/wiki/Object_code) . Finally, a [linker](https://en.wikipedia.org/wiki/Linker_(computing)) is used to combine the object(s) into a single executable image. [136]

rustc uses [LLVM](https://en.wikipedia.org/wiki/LLVM) as its compiler backend by default, but it also supports using alternative backends such as [GCC](https://en.wikipedia.org/wiki/GNU_Compiler_Collection) and [Cranelift](https://en.wikipedia.org/wiki/Cranelift) . [137] The intention of those alternative backends is to increase platform coverage of Rust or to improve compilation times. [138] [139]

### Cargo

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=38) ]
Screenshot of crates.io in June 2022
Cargo is Rust's [build system](https://en.wikipedia.org/wiki/Build_system_(software_development)) and [package manager](https://en.wikipedia.org/wiki/Package_manager) . It downloads, compiles, distributes, and uploads packages—called _crates _ —that are maintained in an official registry. It also acts as a front-end for Clippy and other Rust components. [140]

By default, Cargo sources its dependencies from the user-contributed registry _crates.io _ , but [Git](https://en.wikipedia.org/wiki/Git) repositories, crates in the local filesystem, and other external sources can also be specified as dependencies. [141]

Cargo supports reproducible builds through two metadata files: Cargo.toml and Cargo.lock. [142] Cargo.toml declares each package used and their version requirements. Cargo.lock is generated automatically during dependency resolution and records exact versions of all dependencies, including [transitive dependencies](https://en.wikipedia.org/wiki/Transitive_dependency) . [143]

### Rustfmt

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=39) ]

Rustfmt is a [code formatter](https://en.wikipedia.org/wiki/Code_formatter) for Rust. It formats whitespace and [indentation](https://en.wikipedia.org/wiki/Indentation_style) to produce code in accordance with a common [style](https://en.wikipedia.org/wiki/Programming_style) , unless otherwise specified. It can be invoked as a standalone program, or from a Rust project through Cargo. [144]

### Clippy

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=40) ]
Example output of Clippy on a hello world Rust program
Clippy is Rust's built-in [linting](https://en.wikipedia.org/wiki/Linting) tool to improve the correctness, performance, and readability of Rust code. As of 2026 [[update]](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit) , it has over 800 rules. [145] [146]

### Versioning system

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=41) ]

Following Rust 1.0, new features are developed in _nightly _ versions which are released daily. During each six-week release cycle, changes to nightly versions are released to beta, while changes from the previous beta version are released to a new stable version. [147]

Every two or three years, a new "edition" is produced. Editions are released to allow making limited [breaking changes](https://en.wikipedia.org/wiki/Breaking_changes) , such as promoting `await `to a keyword to support [async/await](https://en.wikipedia.org/wiki/Async/await) features. Crates targeting different editions can interoperate with each other, so a crate can upgrade to a new edition even if its callers or its dependencies still target older editions. Migration to a new edition can be assisted with automated tooling. [148]

### IDE support

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=42) ]

_rust-analyzer _ is a set of [utilities](https://en.wikipedia.org/wiki/Utility_software) that provides [integrated development environments](https://en.wikipedia.org/wiki/Integrated_development_environment) (IDEs) and [text editors](https://en.wikipedia.org/wiki/Text_editor) with information about a Rust project through the [Language Server Protocol](https://en.wikipedia.org/wiki/Language_Server_Protocol) . This enables features including [autocomplete](https://en.wikipedia.org/wiki/Autocomplete) , and [compilation error](https://en.wikipedia.org/wiki/Compilation_error) display, while editing code. [149]

## Performance

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=43) ]

Since it performs no garbage collection, Rust is often faster than other memory-safe languages. [150] [74] [151] Most of Rust's memory safety guarantees impose no runtime overhead, [152] with the exception of [array indexing](https://en.wikipedia.org/wiki/Array_(data_structure)) which is checked at runtime by default. [153] The performance impact of array indexing bounds checks varies, but can be significant in some cases. [153]

Many of Rust's features are so-called _zero-cost abstractions _ , meaning they are optimized away at compile time and incur no runtime penalty. [154] The ownership and borrowing system permits [zero-copy](https://en.wikipedia.org/wiki/Zero-copy) implementations for some performance-sensitive tasks, such as [parsing](https://en.wikipedia.org/wiki/Parsing) . [155] [Static dispatch](https://en.wikipedia.org/wiki/Static_dispatch) is used by default to eliminate [method calls](https://en.wikipedia.org/wiki/Method_call) , except for methods called on dynamic trait objects. [156] The compiler uses [inline expansion](https://en.wikipedia.org/wiki/Inline_expansion) to eliminate [function calls](https://en.wikipedia.org/wiki/Function_call) and statically dispatched method invocations. [157]

Since Rust uses [LLVM](https://en.wikipedia.org/wiki/LLVM) , all performance improvements in LLVM apply to Rust also. [158] Unlike C and C++, Rust allows the compiler to reorder struct and enum elements unless a `#[repr(C)] `representation attribute is applied. [159] This allows the compiler to optimize for memory footprint, alignment, and padding, which can be used to produce more efficient code in some cases. [160]

## Adoption

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=44) ]

See also: [Category:Software programmed in Rust](https://en.wikipedia.org/wiki/Category:Software_programmed_in_Rust)
[Firefox](https://en.wikipedia.org/wiki/Firefox) has components written in Rust as part of the underlying [Gecko](https://en.wikipedia.org/wiki/Gecko_(software)) browser engine.
In [web services](https://en.wikipedia.org/wiki/Web_service) , [OpenDNS](https://en.wikipedia.org/wiki/OpenDNS) , a [DNS](https://en.wikipedia.org/wiki/Domain_Name_System) resolution service owned by [Cisco](https://en.wikipedia.org/wiki/Cisco) , uses Rust internally. [161] [162] [Amazon Web Services](https://en.wikipedia.org/wiki/Amazon_Web_Services) uses Rust in "performance-sensitive components" of its several services. In 2019, AWS [open-sourced](https://en.wikipedia.org/wiki/Open_sourced) [Firecracker](https://en.wikipedia.org/wiki/Firecracker_(software)) , a virtualization solution primarily written in Rust. [163] [Microsoft Azure](https://en.wikipedia.org/wiki/Microsoft_Azure) IoT Edge, a platform used to run Azure services on [IoT](https://en.wikipedia.org/wiki/Internet_of_things) devices, has components implemented in Rust. [164] Microsoft also uses Rust to run containerized modules with [WebAssembly](https://en.wikipedia.org/wiki/WebAssembly) and [Kubernetes](https://en.wikipedia.org/wiki/Kubernetes) . [165] [Cloudflare](https://en.wikipedia.org/wiki/Cloudflare) , a company providing [content delivery network](https://en.wikipedia.org/wiki/Content_delivery_network) services, used Rust to build a new [web proxy](https://en.wikipedia.org/wiki/Web_proxy) named Pingora for increased performance and efficiency. [166] The [npm package manager](https://en.wikipedia.org/wiki/Npm) used Rust for its production authentication service in 2019. [167] [168] [169]
The [Rust for Linux](https://en.wikipedia.org/wiki/Rust_for_Linux) project has been supported in the [Linux kernel](https://en.wikipedia.org/wiki/Linux_kernel) since 2022.
In operating systems, the Linux kernel began introducing experimental support for Rust code in Version 6.1 in late 2022, as part of the [Rust for Linux](https://en.wikipedia.org/wiki/Rust_for_Linux) project. [170] [171] [172] The first drivers written in Rust were included in version 6.8. [170] In 2025, kernel developers at the [Linux Kernel Developers Summit](https://en.wikipedia.org/wiki/Linux_Kernel_Developers_Summit) determined the project to be a success, and Rust usage for kernel code will no longer be considered experimental. [173] The [Android](https://en.wikipedia.org/wiki/Android_(operating_system)) developers used Rust in 2021 to rewrite existing components. [174] [175] [Microsoft](https://en.wikipedia.org/wiki/Microsoft) has rewritten parts of [Windows](https://en.wikipedia.org/wiki/Windows) in Rust. [176] The r9 project aims to re-implement [Plan 9 from Bell Labs](https://en.wikipedia.org/wiki/Plan_9_from_Bell_Labs) in Rust. [177] Rust has also been used in the development of new operating systems such as [Redox](https://en.wikipedia.org/wiki/Redox_(operating_system)) , a "Unix-like" operating system and [microkernel](https://en.wikipedia.org/wiki/Microkernel) , [178] Theseus, an experimental operating system with modular state management, [179] [180] and most of [Fuchsia](https://en.wikipedia.org/wiki/Fuchsia_(operating_system)) . [181] Rust is used for command-line tools and operating system components such as [stratisd](https://en.wikipedia.org/wiki/Stratis_(configuration_daemon)) , a [file system](https://en.wikipedia.org/wiki/File_system) manager [182] [183] and COSMIC, a [desktop environment](https://en.wikipedia.org/wiki/Desktop_environment) by [System76](https://en.wikipedia.org/wiki/System76) . [184]

In web development, [Deno](https://en.wikipedia.org/wiki/Deno_(software)) , a secure runtime for [JavaScript](https://en.wikipedia.org/wiki/JavaScript) and [TypeScript](https://en.wikipedia.org/wiki/TypeScript) , is built on top of [V8](https://en.wikipedia.org/wiki/V8_(JavaScript_engine)) using Rust and Tokio. [185] Other notable adoptions in this space include [Ruffle](https://en.wikipedia.org/wiki/Ruffle_(software)) , an open-source [SWF](https://en.wikipedia.org/wiki/SWF) emulator, [186] and [Polkadot](https://en.wikipedia.org/wiki/Polkadot_(cryptocurrency)) , an open source [blockchain](https://en.wikipedia.org/wiki/Blockchain) and [cryptocurrency](https://en.wikipedia.org/wiki/Cryptocurrency) platform. [187] Components from the Servo browser engine (funded by [Mozilla](https://en.wikipedia.org/wiki/Mozilla) and [Samsung](https://en.wikipedia.org/wiki/Samsung) ) were incorporated in the [Gecko](https://en.wikipedia.org/wiki/Gecko_(software)) browser engine underlying [Firefox](https://en.wikipedia.org/wiki/Firefox) . [188] In January 2023, Google ( [Alphabet](https://en.wikipedia.org/wiki/Alphabet_Inc.) ) announced support for using third party Rust libraries in [Chromium](https://en.wikipedia.org/wiki/Chromium_(web_browser)) . [189] [190]

In other uses, [Discord](https://en.wikipedia.org/wiki/Discord) , an [instant messaging](https://en.wikipedia.org/wiki/Instant_messaging) software company, rewrote parts of its system in Rust for increased performance in 2020. In the same year, Dropbox announced that its [file synchronization](https://en.wikipedia.org/wiki/File_synchronization) had been rewritten in Rust. [Facebook](https://en.wikipedia.org/wiki/Facebook) ( [Meta](https://en.wikipedia.org/wiki/Meta_Platforms) ) used Rust to redesign its system that manages source code for internal projects. [16]

In the 2025 [Stack Overflow](https://en.wikipedia.org/wiki/Stack_Overflow) Developer Survey, 14.8% of respondents had recently done extensive development in Rust. [191] The survey named Rust the "most admired programming language" annually from 2016 to 2025 (inclusive), as measured by the number of existing developers interested in continuing to work in the language. [192] [note 7] In 2025, 29.2% of developers not currently working in Rust expressed an interest in doing so. [191]

## In academic research

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=45) ]

Rust's safety and performance have been investigated in programming languages research. [193] [113] [194]

In other fields, a journal article published to _[Proceedings of the International Astronomical Union](https://en.wikipedia.org/wiki/Proceedings_of_the_International_Astronomical_Union) _ used Rust to simulate multi-planet systems. [195] An article published in _[Nature](https://en.wikipedia.org/wiki/Nature_(journal)) _ shared stories of bioinformaticians using Rust. [140] Both articles cited Rust's performance and safety as advantages, and the [learning curve](https://en.wikipedia.org/wiki/Learning_curve) as being a primary drawback to Rust adoption.

The 2025 [DARPA](https://en.wikipedia.org/wiki/DARPA) project TRACTOR aims to automatically translate C to Rust using techniques such as static analysis, dynamic analysis, and large language models. [196]

## Community

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=46) ]
Some Rust users refer to themselves as Rustaceans (similar to the word [crustacean](https://en.wikipedia.org/wiki/Crustacean) ) and have adopted an orange crab, Ferris, as their unofficial mascot. [197] [198]
According to the _[MIT Technology Review](https://en.wikipedia.org/wiki/MIT_Technology_Review) _ , the Rust community has been seen as "unusually friendly" to newcomers and particularly attracted people from the [queer community](https://en.wikipedia.org/wiki/Queer_community) , partly due to its [code of conduct](https://en.wikipedia.org/wiki/Code_of_conduct) . [16] Inclusiveness has been cited as an important factor for some Rust developers. [140] The official Rust blog collects and publishes demographic data each year. [199]

### Rust Foundation

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=47) ]

Rust Foundation

Formation February 8, 2021 ; 5 years ago ( 2021-02-08 )
Founders

- [Amazon Web Services](https://en.wikipedia.org/wiki/Amazon_Web_Services)
- [Google](https://en.wikipedia.org/wiki/Google)
- [Huawei](https://en.wikipedia.org/wiki/Huawei)
- [Microsoft](https://en.wikipedia.org/wiki/Microsoft)
- [Mozilla Foundation](https://en.wikipedia.org/wiki/Mozilla_Foundation)

Type [Nonprofit organization](https://en.wikipedia.org/wiki/Nonprofit_organization)
Location

- [United States](https://en.wikipedia.org/wiki/United_States)

[Chairperson](https://en.wikipedia.org/wiki/Chairperson)
Shane Miller

[Executive Director](https://en.wikipedia.org/wiki/Executive_Director)
Rebecca Rumbul
Website [foundation.rust-lang.org](http://foundation.rust-lang.org)

The **Rust Foundation ** is a [non-profit](https://en.wikipedia.org/wiki/Nonprofit_organization) [membership organization](https://en.wikipedia.org/wiki/Membership_organization) incorporated in [United States](https://en.wikipedia.org/wiki/United_States) ; it manages the Rust trademark, infrastructure, and assets. [200] [45]

It was established on February 8, 2021, with five founding corporate members (Amazon Web Services, Huawei, Google, Microsoft, and Mozilla). [201] The foundation's board was chaired by Shane Miller, [202] with Ashley Williams as interim executive director. [45] In late 2021, Rebecca Rumbul became executive director and CEO. [203]

### Governance teams

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=48) ]

The Rust project is maintained by 8 top-level _teams _ as of November 2025 [[update]](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit) : the leadership council, compiler team, dev tools team, infrastructure team, language team, launching pad, library team, and moderation team. [204] The leadership council oversees the project and is formed by representatives among the other teams. [205]

## See also

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=49) ]

- [Comparison of programming languages](https://en.wikipedia.org/wiki/Comparison_of_programming_languages)
- [History of programming languages](https://en.wikipedia.org/wiki/History_of_programming_languages)
- [List of programming languages](https://en.wikipedia.org/wiki/List_of_programming_languages)
- [List of Rust software and tools](https://en.wikipedia.org/wiki/List_of_Rust_software_and_tools)
- [Outline of the Rust programming language](https://en.wikipedia.org/wiki/Outline_of_the_Rust_programming_language)

## Notes

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=50) ]

1. **^ ** Including build tools, host tools, and standard library support for [x86-64](https://en.wikipedia.org/wiki/X86-64) , [ARM](https://en.wikipedia.org/wiki/ARM_architecture_family) , [MIPS](https://en.wikipedia.org/wiki/MIPS_architecture) , [RISC-V](https://en.wikipedia.org/wiki/RISC-V) , [WebAssembly](https://en.wikipedia.org/wiki/WebAssembly) , [i686](https://en.wikipedia.org/wiki/P6_(microarchitecture)) , [AArch64](https://en.wikipedia.org/wiki/AArch64) , [PowerPC](https://en.wikipedia.org/wiki/PowerPC) , and [s390x](https://en.wikipedia.org/wiki/Linux_on_IBM_Z) . [2]
2. **^ ** Including [Windows](https://en.wikipedia.org/wiki/Windows) , [Linux](https://en.wikipedia.org/wiki/Linux) , [macOS](https://en.wikipedia.org/wiki/MacOS) , [FreeBSD](https://en.wikipedia.org/wiki/FreeBSD) , [NetBSD](https://en.wikipedia.org/wiki/NetBSD) , and [Illumos](https://en.wikipedia.org/wiki/Illumos) . Host build tools on [Android](https://en.wikipedia.org/wiki/Android_(operating_system)) , [iOS](https://en.wikipedia.org/wiki/IOS) , [Haiku](https://en.wikipedia.org/wiki/Haiku_(operating_system)) , [Redox](https://en.wikipedia.org/wiki/Redox_(operating_system)) , and [Fuchsia](https://en.wikipedia.org/wiki/Fuchsia_(operating_system)) are not officially shipped; these operating systems are supported as targets. [2]
3. **^ ** Third-party dependencies, e.g., [LLVM](https://en.wikipedia.org/wiki/LLVM) or [MSVC](https://en.wikipedia.org/wiki/MSVC) , are subject to their own licenses. [3] [4]
4. ^ a b NIL is cited as an influence for Rust in multiple sources; this likely refers to Network Implementation Language developed by Robert Strom and others at [IBM](https://en.wikipedia.org/wiki/IBM) , which pioneered [typestate analysis](https://en.wikipedia.org/wiki/Typestate_analysis) , [5] [6] not to be confused with [New Implementation of LISP](https://en.wikipedia.org/wiki/NIL_(programming_language)) .
5. **^ ** The list of Rust compiler versions (referred to as a bootstrapping chain) has history going back to 2012. [21]
6. **^ ** wrapping `no_mangle `with `unsafe `as well as prefacing the `extern "C" `block with `unsafe `are required in the 2024 edition or later. [126]
7. **^ ** That is, among respondents who have done "extensive development work [with Rust] in over the past year" (14.8%), Rust had the largest percentage who also expressed interest to "work in [Rust] over the next year" (72.4%). [191]

## References

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=51) ]

### Book sources

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=52) ]

- Gjengset, Jon (2021). _Rust for Rustaceans _ (1st ed.). No Starch Press. [ISBN](https://en.wikipedia.org/wiki/ISBN_(identifier)) [9781718501850](https://en.wikipedia.org/wiki/Special:BookSources/9781718501850) . [OCLC](https://en.wikipedia.org/wiki/OCLC_(identifier)) [1277511986](https://search.worldcat.org/oclc/1277511986) .
- Klabnik, Steve; Nichols, Carol (2019-08-12). [The Rust Programming Language (Covers Rust 2018)](https://books.google.com/books?id=0Vv6DwAAQBAJ) . No Starch Press. [ISBN](https://en.wikipedia.org/wiki/ISBN_(identifier)) [978-1-7185-0044-0](https://en.wikipedia.org/wiki/Special:BookSources/978-1-7185-0044-0) .
- Blandy, Jim; Orendorff, Jason; Tindall, Leonora F. S. (2021). _Programming Rust: Fast, Safe Systems Development _ (2nd ed.). O'Reilly Media. [ISBN](https://en.wikipedia.org/wiki/ISBN_(identifier)) [978-1-4920-5254-8](https://en.wikipedia.org/wiki/Special:BookSources/978-1-4920-5254-8) . [OCLC](https://en.wikipedia.org/wiki/OCLC_(identifier)) [1289839504](https://search.worldcat.org/oclc/1289839504) .
- McNamara, Tim (2021). _Rust in Action _ . Manning Publications. [ISBN](https://en.wikipedia.org/wiki/ISBN_(identifier)) [978-1-6172-9455-6](https://en.wikipedia.org/wiki/Special:BookSources/978-1-6172-9455-6) . [OCLC](https://en.wikipedia.org/wiki/OCLC_(identifier)) [1153044639](https://search.worldcat.org/oclc/1153044639) .
- Klabnik, Steve; Nichols, Carol (2023). _The Rust programming language _ (2nd ed.). No Starch Press. [ISBN](https://en.wikipedia.org/wiki/ISBN_(identifier)) [978-1-7185-0310-6](https://en.wikipedia.org/wiki/Special:BookSources/978-1-7185-0310-6) . [OCLC](https://en.wikipedia.org/wiki/OCLC_(identifier)) [1363816350](https://search.worldcat.org/oclc/1363816350) .

### Others

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=53) ]

1. **^ ** ["Announcing Rust 1.94.1"](https://blog.rust-lang.org/2026/03/26/1.94.1-release/) .
2. ^ a b ["Platform Support"](https://doc.rust-lang.org/rustc/platform-support.html) . _The rustc book _ . [Archived](https://web.archive.org/web/20220630164523/https://doc.rust-lang.org/rustc/platform-support.html) from the original on 2022-06-30 . Retrieved 2022-06-27 .
3. **^ ** ["Copyright"](https://github.com/rust-lang/rust/blob/master/COPYRIGHT) . _[GitHub](https://en.wikipedia.org/wiki/GitHub) _ . The Rust Programming Language. 2022-10-19. [Archived](https://web.archive.org/web/20230722190056/http://github.com/rust-lang/rust/blob/master/COPYRIGHT) from the original on 2023-07-22 . Retrieved 2022-10-19 .
4. **^ ** ["Licenses"](https://www.rust-lang.org/policies/licenses) . _The Rust Programming Language _ . [Archived](https://web.archive.org/web/20250223193908/https://www.rust-lang.org/policies/licenses) from the original on 2025-02-23 . Retrieved 2025-03-07 .
5. **^ ** Strom, Robert E. (1983). "Mechanisms for compile-time enforcement of security". _Proceedings of the 10th ACM SIGACT-SIGPLAN symposium on Principles of programming languages - POPL '83 _ . pp. 276– 284. [doi](https://en.wikipedia.org/wiki/Doi_(identifier)) : [10.1145/567067.567093](https://doi.org/10.1145%2F567067.567093) . [ISBN](https://en.wikipedia.org/wiki/ISBN_(identifier)) [0897910907](https://en.wikipedia.org/wiki/Special:BookSources/0897910907) . [S2CID](https://en.wikipedia.org/wiki/S2CID_(identifier)) [6630704](https://api.semanticscholar.org/CorpusID:6630704) .
6. **^ ** Strom, Robert E.; Yemini, Shaula (1986). ["Typestate: A programming language concept for enhancing software reliability"](https://www.cs.cmu.edu/~aldrich/papers/classic/tse12-typestate.pdf) (PDF) . _IEEE Transactions on Software Engineering _ . **12 ** (1). IEEE: 157– 171. [Bibcode](https://en.wikipedia.org/wiki/Bibcode_(identifier)) : [1986ITSEn..12..157S](https://ui.adsabs.harvard.edu/abs/1986ITSEn..12..157S) . [doi](https://en.wikipedia.org/wiki/Doi_(identifier)) : [10.1109/tse.1986.6312929](https://doi.org/10.1109%2Ftse.1986.6312929) . [S2CID](https://en.wikipedia.org/wiki/S2CID_(identifier)) [15575346](https://api.semanticscholar.org/CorpusID:15575346) .
7. **^ ** ["Uniqueness Types"](https://blog.rust-lang.org/2016/08/10/Shape-of-errors-to-come.html) . _Rust Blog _ . [Archived](https://web.archive.org/web/20160915133745/https://blog.rust-lang.org/2016/08/10/Shape-of-errors-to-come.html) from the original on 2016-09-15 . Retrieved 2016-10-08 . Those of you familiar with the Elm style may recognize that the updated --explain messages draw heavy inspiration from the Elm approach.
8. **^ ** ["Influences"](https://doc.rust-lang.org/reference/influences.html) . _The Rust Reference _ . [Archived](https://web.archive.org/web/20231126231034/https://doc.rust-lang.org/reference/influences.html) from the original on 2023-11-26 . Retrieved 2023-12-31 .
9. **^ ** ["Uniqueness Types"](http://docs.idris-lang.org/en/latest/reference/uniqueness-types.html) . _Idris 1.3.3 documentation _ . [Archived](https://web.archive.org/web/20181121072557/http://docs.idris-lang.org/en/latest/reference/uniqueness-types.html) from the original on 2018-11-21 . Retrieved 2022-07-14 . They are inspired by ... ownership types and borrowed pointers in the Rust programming language.
10. **^ ** Tung, Liam. ["Microsoft opens up Rust-inspired Project Verona programming language on GitHub"](https://www.zdnet.com/article/microsoft-opens-up-rust-inspired-project-verona-programming-language-on-github/) . _[ZDNET](https://en.wikipedia.org/wiki/ZDNET) _ . [Archived](https://web.archive.org/web/20200117143852/https://www.zdnet.com/article/microsoft-opens-up-rust-inspired-project-verona-programming-language-on-github/) from the original on 2020-01-17 . Retrieved 2020-01-17 .
11. **^ ** Jaloyan, Georges-Axel (2017-10-19). "Safe Pointers in SPARK 2014". [arXiv](https://en.wikipedia.org/wiki/ArXiv_(identifier)) : [1710.07047](https://arxiv.org/abs/1710.07047) [ [cs.PL](https://arxiv.org/archive/cs.PL) ].
12. **^ ** Lattner, Chris. ["Chris Lattner's Homepage"](http://nondot.org/sabre/) . _Nondot _ . [Archived](https://web.archive.org/web/20181225175312/http://nondot.org/sabre/) from the original on 2018-12-25 . Retrieved 2019-05-14 .
13. **^ ** ["V documentation (Introduction)"](https://github.com/vlang/v/blob/master/doc/docs.md#introduction) . _[GitHub](https://en.wikipedia.org/wiki/GitHub) _ . The V Programming Language . Retrieved 2023-11-04 .
14. **^ ** Yegulalp, Serdar (2016-08-29). ["New challenger joins Rust to topple C language"](https://www.infoworld.com/article/3113083/new-challenger-joins-rust-to-upend-c-language.html) . _[InfoWorld](https://en.wikipedia.org/wiki/InfoWorld) _ . [Archived](https://web.archive.org/web/20211125104022/https://www.infoworld.com/article/3113083/new-challenger-joins-rust-to-upend-c-language.html) from the original on 2021-11-25 . Retrieved 2022-10-19 .
15. **^ ** ["Gleam for Rust users"](https://gleam.run/cheatsheets/gleam-for-rust-users/) . [Archived](https://web.archive.org/web/20260127121406/https://gleam.run/cheatsheets/gleam-for-rust-users/) from the original on 2026-01-27 . Retrieved 2026-01-27 .
16. ^ a b c d e f g h i j k l m n o p q r s Thompson, Clive (2023-02-14). ["How Rust went from a side project to the world's most-loved programming language"](https://www.technologyreview.com/2023/02/14/1067869/rust-worlds-fastest-growing-programming-language/) . _MIT Technology Review _ . [Archived](https://web.archive.org/web/20240919102849/https://www.technologyreview.com/2023/02/14/1067869/rust-worlds-fastest-growing-programming-language/) from the original on 2024-09-19 . Retrieved 2023-02-23 .
17. ^ a b c d e f g h i j k l m n o p q r s t u Klabnik, Steve (2016-06-02). ["The History of Rust"](https://dl.acm.org/doi/10.1145/2959689.2960081) . _Applicative 2016 _ . New York, NY, USA: Association for Computing Machinery. p. 80. [doi](https://en.wikipedia.org/wiki/Doi_(identifier)) : [10.1145/2959689.2960081](https://doi.org/10.1145%2F2959689.2960081) . [ISBN](https://en.wikipedia.org/wiki/ISBN_(identifier)) [978-1-4503-4464-7](https://en.wikipedia.org/wiki/Special:BookSources/978-1-4503-4464-7) .
18. ^ a b c d Hoare, Graydon (July 2010). [Project Servo: Technology from the past come to save the future from itself](http://venge.net/graydon/talks/intro-talk-2.pdf) (PDF) . Mozilla Annual Summit . Retrieved 2024-10-29 . `{{ [cite conference](https://en.wikipedia.org/wiki/Template:Cite_conference) }} `: CS1 maint: deprecated archival service ( [link](https://en.wikipedia.org/wiki/Category:CS1_maint:_deprecated_archival_service) )
19. **^ ** Hoare, Graydon (November 2016). ["Rust Prehistory (Archive of the original Rust OCaml compiler source code)"](https://github.com/graydon/rust-prehistory/tree/master) . _[GitHub](https://en.wikipedia.org/wiki/GitHub) _ . Retrieved 2024-10-29 .
20. **^ ** ["0.1 first supported public release Milestone · rust-lang/rust"](https://github.com/rust-lang/rust/milestone/3?closed=1) . _[GitHub](https://en.wikipedia.org/wiki/GitHub) _ . Retrieved 2024-10-29 .
21. **^ ** Nelson, Jynn (2022-08-05). [RustConf 2022 - Bootstrapping: The once and future compiler](https://www.youtube.com/watch?v=oUIjG-y4zaA) . Portland, Oregon: Rust Team . Retrieved 2024-10-29 – via YouTube.
22. **^ ** ["Rust logo"](https://bugzilla.mozilla.org/show_bug.cgi?id=680521) . _[Bugzilla](https://en.wikipedia.org/wiki/Bugzilla) _ . [Archived](https://web.archive.org/web/20240202045212/https://bugzilla.mozilla.org/show_bug.cgi?id=680521) from the original on 2024-02-02 . Retrieved 2024-02-02 .
23. **^ ** Anderson, Brian (2012-01-24). ["[rust-dev] The Rust compiler 0.1 is unleashed"](https://web.archive.org/web/20120124160628/https://mail.mozilla.org/pipermail/rust-dev/2012-January/001256.html) . _rust-dev _ (Mailing list). Archived from [the original](https://mail.mozilla.org/pipermail/rust-dev/2012-January/001256.html) on 2012-01-24 . Retrieved 2025-01-07 .
24. **^ ** Anthony, Sebastian (2012-01-24). ["Mozilla releases Rust 0.1, the language that will eventually usurp Firefox's C++"](https://www.extremetech.com/internet/115207-mozilla-releases-rust-0-1-the-language-that-will-eventually-usurp-firefoxs-c) . _ExtremeTech _ . Retrieved 2025-01-07 .
25. **^ ** ["Purity by pcwalton · Pull Request #5412 · rust-lang/rust"](https://github.com/rust-lang/rust/pull/5412) . _[GitHub](https://en.wikipedia.org/wiki/GitHub) _ . Retrieved 2024-10-29 .
26. **^ ** Binstock, Andrew (2014-01-07). ["The Rise And Fall of Languages in 2013"](https://web.archive.org/web/20160807075745/http://www.drdobbs.com/jvm/the-rise-and-fall-of-languages-in-2013/240165192) . _[Dr. Dobb's Journal](https://en.wikipedia.org/wiki/Dr._Dobb%27s_Journal) _ . Archived from [the original](https://www.drdobbs.com/jvm/the-rise-and-fall-of-languages-in-2013/240165192) on 2016-08-07 . Retrieved 2022-11-20 .
27. **^ ** Lardinois, Frederic (2015-04-03). ["Mozilla And Samsung Team Up To Develop Servo, Mozilla's Next-Gen Browser Engine For Multicore Processors"](https://techcrunch.com/2013/04/03/mozilla-and-samsung-collaborate-on-servo-mozillas-next-gen-browser-engine-for-tomorrows-multicore-processors/) . _[TechCrunch](https://en.wikipedia.org/wiki/TechCrunch) _ . [Archived](https://web.archive.org/web/20160910211537/https://techcrunch.com/2013/04/03/mozilla-and-samsung-collaborate-on-servo-mozillas-next-gen-browser-engine-for-tomorrows-multicore-processors/) from the original on 2016-09-10 . Retrieved 2017-06-25 .
28. **^ ** ["Firefox 45.0, See All New Features, Updates and Fixes"](https://www.mozilla.org/en-US/firefox/45.0/releasenotes/) . _Mozilla _ . [Archived](https://web.archive.org/web/20160317215950/https://www.mozilla.org/en-US/firefox/45.0/releasenotes/) from the original on 2016-03-17 . Retrieved 2024-10-31 .
29. **^ ** Lardinois, Frederic (2017-09-29). ["It's time to give Firefox another chance"](https://techcrunch.com/2017/09/29/its-time-to-give-firefox-another-chance/) . _[TechCrunch](https://en.wikipedia.org/wiki/TechCrunch) _ . [Archived](https://web.archive.org/web/20230815025149/https://techcrunch.com/2017/09/29/its-time-to-give-firefox-another-chance/) from the original on 2023-08-15 . Retrieved 2023-08-15 .
30. **^ ** Pereira, Rui; Couto, Marco; Ribeiro, Francisco; Rua, Rui; Cunha, Jácome; Fernandes, João Paulo; Saraiva, João (2017-10-23). ["Energy efficiency across programming languages: How do energy, time, and memory relate?"](https://dl.acm.org/doi/10.1145/3136014.3136031) . [Proceedings of the 10th ACM SIGPLAN International Conference on Software Language Engineering](http://repositorio.inesctec.pt/handle/123456789/5492) . SLE 2017. New York, NY, USA: Association for Computing Machinery. pp. 256– 267. [doi](https://en.wikipedia.org/wiki/Doi_(identifier)) : [10.1145/3136014.3136031](https://doi.org/10.1145%2F3136014.3136031) . [hdl](https://en.wikipedia.org/wiki/Hdl_(identifier)) : [1822/65359](https://hdl.handle.net/1822%2F65359) . [ISBN](https://en.wikipedia.org/wiki/ISBN_(identifier)) [978-1-4503-5525-4](https://en.wikipedia.org/wiki/Special:BookSources/978-1-4503-5525-4) .
31. **^ ** Cimpanu, Catalin (2020-08-11). ["Mozilla lays off 250 employees while it refocuses on commercial products"](https://www.zdnet.com/article/mozilla-lays-off-250-employees-while-it-refocuses-on-commercial-products/) . _[ZDNET](https://en.wikipedia.org/wiki/ZDNET) _ . [Archived](https://web.archive.org/web/20220318025804/https://www.zdnet.com/article/mozilla-lays-off-250-employees-while-it-refocuses-on-commercial-products/) from the original on 2022-03-18 . Retrieved 2020-12-02 .
32. **^ ** Cooper, Daniel (2020-08-11). ["Mozilla lays off 250 employees due to the pandemic"](https://www.engadget.com/mozilla-firefox-250-employees-layoffs-151324924.html) . _[Engadget](https://en.wikipedia.org/wiki/Engadget) _ . [Archived](https://web.archive.org/web/20201213020220/https://www.engadget.com/mozilla-firefox-250-employees-layoffs-151324924.html) from the original on 2020-12-13 . Retrieved 2020-12-02 .
33. **^ ** Tung, Liam (2020-08-21). ["Programming language Rust: Mozilla job cuts have hit us badly but here's how we'll survive"](https://www.zdnet.com/article/programming-language-rust-mozilla-job-cuts-have-hit-us-badly-but-heres-how-well-survive/) . _[ZDNET](https://en.wikipedia.org/wiki/ZDNET) _ . [Archived](https://web.archive.org/web/20220421083509/https://www.zdnet.com/article/programming-language-rust-mozilla-job-cuts-have-hit-us-badly-but-heres-how-well-survive/) from the original on 2022-04-21 . Retrieved 2022-04-21 .
34. **^ ** ["Laying the foundation for Rust's future"](https://blog.rust-lang.org/2020/08/18/laying-the-foundation-for-rusts-future.html) . _Rust Blog _ . 2020-08-18. [Archived](https://web.archive.org/web/20201202022933/https://blog.rust-lang.org/2020/08/18/laying-the-foundation-for-rusts-future.html) from the original on 2020-12-02 . Retrieved 2020-12-02 .
35. **^ ** ["Hello World!"](https://foundation.rust-lang.org/news/2021-02-08-hello-world/) . _Rust Foundation _ . 2020-02-08. [Archived](https://web.archive.org/web/20220419124635/https://foundation.rust-lang.org/news/2021-02-08-hello-world/) from the original on 2022-04-19 . Retrieved 2022-06-04 .
36. **^ ** ["Mozilla Welcomes the Rust Foundation"](https://blog.mozilla.org/blog/2021/02/08/mozilla-welcomes-the-rust-foundation) . _Mozilla Blog _ . 2021-02-09. [Archived](https://web.archive.org/web/20210208212031/https://blog.mozilla.org/blog/2021/02/08/mozilla-welcomes-the-rust-foundation/) from the original on 2021-02-08 . Retrieved 2021-02-09 .
37. **^ ** Amadeo, Ron (2021-04-07). ["Google is now writing low-level Android code in Rust"](https://arstechnica.com/gadgets/2021/04/google-is-now-writing-low-level-android-code-in-rust/) . _Ars Technica _ . [Archived](https://web.archive.org/web/20210408001446/https://arstechnica.com/gadgets/2021/04/google-is-now-writing-low-level-android-code-in-rust/) from the original on 2021-04-08 . Retrieved 2021-04-08 .
38. **^ ** Anderson, Tim (2021-11-23). ["Entire Rust moderation team resigns"](https://www.theregister.com/2021/11/23/rust_moderation_team_quits/) . _[The Register](https://en.wikipedia.org/wiki/The_Register) _ . [Archived](https://web.archive.org/web/20220714093245/https://www.theregister.com/2021/11/23/rust_moderation_team_quits/) from the original on 2022-07-14 . Retrieved 2022-08-04 .
39. **^ ** Levick, Ryan; Bos, Mara. ["Governance Update"](https://blog.rust-lang.org/inside-rust/2022/05/19/governance-update.html) . _Inside Rust Blog _ . [Archived](https://web.archive.org/web/20221027030926/https://blog.rust-lang.org/inside-rust/2022/05/19/governance-update.html) from the original on 2022-10-27 . Retrieved 2022-10-27 .
40. ^ a b Claburn, Thomas (2023-04-17). ["Rust Foundation apologizes for trademark policy confusion"](https://www.theregister.com/2023/04/17/rust_foundation_apologizes_trademark_policy/) . _[The Register](https://en.wikipedia.org/wiki/The_Register) _ . [Archived](https://web.archive.org/web/20230507053637/https://www.theregister.com/2023/04/17/rust_foundation_apologizes_trademark_policy/) from the original on 2023-05-07 . Retrieved 2023-05-07 .
41. **^ ** Gross, Grant (2024-02-27). ["White House urges developers to dump C and C++"](https://www.infoworld.com/article/2336216/white-house-urges-developers-to-dump-c-and-c.html) . _[InfoWorld](https://en.wikipedia.org/wiki/InfoWorld) _ . Retrieved 2025-01-26 .
42. **^ ** Warminsky, Joe (2024-02-27). ["After decades of memory-related software bugs, White House calls on industry to act"](https://therecord.media/memory-related-software-bugs-white-house-code-report-oncd) . _The Record _ . Retrieved 2025-01-26 .
43. **^ ** ["Press Release: Future Software Should Be Memory Safe"](https://web.archive.org/web/20250118013136/https://www.whitehouse.gov/oncd/briefing-room/2024/02/26/press-release-technical-report/) . [The White House](https://en.wikipedia.org/wiki/White_House) . 2024-02-26. Archived from [the original](https://www.whitehouse.gov/oncd/briefing-room/2024/02/26/press-release-technical-report/) on 2025-01-18 . Retrieved 2025-01-26 .
44. **^ ** Proven, Liam (2019-11-27). ["Rebecca Rumbul named new CEO of The Rust Foundation"](https://www.theregister.com/2021/11/19/rust_foundation_ceo/) . _[The Register](https://en.wikipedia.org/wiki/The_Register) _ . [Archived](https://web.archive.org/web/20220714110957/https://www.theregister.com/2021/11/19/rust_foundation_ceo/) from the original on 2022-07-14 . Retrieved 2022-07-14 . Both are curly bracket languages, with C-like syntax that makes them unintimidating for C programmers.
45. ^ a b c Vigliarolo, Brandon (2021-02-10). ["The Rust programming language now has its own independent foundation"](https://web.archive.org/web/20230320172900/https://www.techrepublic.com/article/the-rust-programming-language-now-has-its-own-independent-foundation/) . _[TechRepublic](https://en.wikipedia.org/wiki/TechRepublic) _ . Archived from [the original](https://www.techrepublic.com/article/the-rust-programming-language-now-has-its-own-independent-foundation/) on 2023-03-20 . Retrieved 2022-07-14 .
46. **^ ** Klabnik & Nichols 2019 , p. 263.
47. **^ ** Klabnik & Nichols 2019 , pp. 5–6.
48. **^ ** Klabnik & Nichols 2023 , p. 32.
49. **^ ** Klabnik & Nichols 2023 , pp. 32–33.
50. **^ ** Klabnik & Nichols 2023 , pp. 49–50.
51. **^ ** Klabnik & Nichols 2023 , pp. 34–36.
52. **^ ** Klabnik & Nichols 2023 , pp. 6, 47, 53.
53. **^ ** Klabnik & Nichols 2023 , pp. 47–48.
54. ^ a b Klabnik & Nichols 2023 , pp. 50–53.
55. **^ ** Klabnik & Nichols 2023 , p. 56.
56. **^ ** Klabnik & Nichols 2023 , pp. 57–58.
57. **^ ** Klabnik & Nichols 2023 , pp. 54–56.
58. **^ ** Klabnik & Nichols 2019 , pp. 104–109.
59. **^ ** Klabnik & Nichols 2019 , pp. 24.
60. **^ ** Klabnik & Nichols 2019 , pp. 36–38.
61. **^ ** ["isize"](https://doc.rust-lang.org/stable/std/primitive.isize.html) . _doc.rust-lang.org _ . Retrieved 2025-09-28 .
62. **^ ** ["usize"](https://doc.rust-lang.org/stable/std/primitive.usize.html) . _doc.rust-lang.org _ . Retrieved 2025-09-28 .
63. **^ ** Klabnik & Nichols 2023 , pp. 36–38.
64. **^ ** Klabnik & Nichols 2023 , p. 502.
65. **^ ** ["Primitive Type char"](https://doc.rust-lang.org/std/primitive.char.html) . _The Rust Standard Library documentation _ . Retrieved 2025-09-07 .
66. **^ ** ["Glossary of Unicode Terms"](https://www.unicode.org/glossary/) . _[Unicode Consortium](https://en.wikipedia.org/wiki/Unicode_Consortium) _ . [Archived](https://web.archive.org/web/20180924092749/http://www.unicode.org/glossary/) from the original on 2018-09-24 . Retrieved 2024-07-30 .
67. **^ ** Klabnik & Nichols 2019 , pp. 38–40.
68. **^ ** Klabnik & Nichols 2023 , pp. 40–42.
69. **^ ** Klabnik & Nichols 2023 , p. 42.
70. **^ ** Klabnik & Nichols 2019 , pp. 59–61.
71. ^ a b Klabnik & Nichols 2019 , pp. 63–68.
72. **^ ** Klabnik & Nichols 2019 , pp. 74–75.
73. **^ ** Klabnik & Nichols 2023 , pp. 71–72.
74. ^ a b Balasubramanian, Abhiram; Baranowski, Marek S.; Burtsev, Anton; Panda, Aurojit; Rakamarić, Zvonimir; Ryzhyk, Leonid (2017-05-07). ["System Programming in Rust"](https://doi.org/10.1145/3102980.3103006) . _Proceedings of the 16th Workshop on Hot Topics in Operating Systems _ . HotOS '17. New York, NY, US: Association for Computing Machinery. pp. 156– 161. [doi](https://en.wikipedia.org/wiki/Doi_(identifier)) : [10.1145/3102980.3103006](https://doi.org/10.1145%2F3102980.3103006) . [ISBN](https://en.wikipedia.org/wiki/ISBN_(identifier)) [978-1-4503-5068-6](https://en.wikipedia.org/wiki/Special:BookSources/978-1-4503-5068-6) . [S2CID](https://en.wikipedia.org/wiki/S2CID_(identifier)) [24100599](https://api.semanticscholar.org/CorpusID:24100599) . [Archived](https://web.archive.org/web/20220611034046/https://dl.acm.org/doi/10.1145/3102980.3103006) from the original on 2022-06-11 . Retrieved 2022-06-01 .
75. **^ ** Klabnik & Nichols 2023 , pp. 327–30.
76. **^ ** ["Lifetimes"](https://doc.rust-lang.org/rust-by-example/scope/lifetime.html) . _Rust by Example _ . [Archived](https://web.archive.org/web/20241116192422/https://doc.rust-lang.org/rust-by-example/scope/lifetime.html) from the original on 2024-11-16 . Retrieved 2024-10-29 .
77. **^ ** ["Explicit annotation"](https://doc.rust-lang.org/rust-by-example/scope/lifetime/explicit.html) . _Rust by Example _ . Retrieved 2024-10-29 .
78. ^ a b Klabnik & Nichols 2019 , p. 194.
79. **^ ** Klabnik & Nichols 2019 , pp. 75, 134.
80. **^ ** Shamrell-Harrington, Nell (2022-04-15). ["The Rust Borrow Checker – a Deep Dive"](https://www.infoq.com/presentations/rust-borrow-checker/) . _InfoQ _ . [Archived](https://web.archive.org/web/20220625140128/https://www.infoq.com/presentations/rust-borrow-checker/) from the original on 2022-06-25 . Retrieved 2022-06-25 .
81. **^ ** Klabnik & Nichols 2019 , pp. 194–195.
82. **^ ** Klabnik & Nichols 2023 , pp. 208–12.
83. **^ ** Klabnik & Nichols 2023 , [4.2. References and Borrowing](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html) .
84. **^ ** Pearce, David (2021-04-17). ["A Lightweight Formalism for Reference Lifetimes and Borrowing in Rust"](https://dl.acm.org/doi/10.1145/3443420) . _ACM Transactions on Programming Languages and Systems _ . **43 ** : 1– 73. [doi](https://en.wikipedia.org/wiki/Doi_(identifier)) : [10.1145/3443420](https://doi.org/10.1145%2F3443420) . [Archived](https://web.archive.org/web/20240415053627/https://dl.acm.org/doi/10.1145/3443420) from the original on 2024-04-15 . Retrieved 2024-12-11 .
85. **^ ** Klabnik & Nichols 2019 , p. 83.
86. **^ ** Klabnik & Nichols 2019 , p. 97.
87. **^ ** Klabnik & Nichols 2019 , pp. 98–101.
88. **^ ** Klabnik & Nichols 2019 , pp. 438–440.
89. **^ ** Klabnik & Nichols 2019 , pp. 93.
90. **^ ** Gjengset 2021 , pp. 213–215.
91. **^ ** Klabnik & Nichols 2023 , pp. 108–110, 113–114, 116–117.
92. **^ ** ["Rust error handling is perfect actually"](https://bitfieldconsulting.com/posts/rust-errors-option-result) . _Bitfield Consulting _ . 2024-05-27. [Archived](https://web.archive.org/web/20250807061432/https://bitfieldconsulting.com/posts/rust-errors-option-result) from the original on 2025-08-07 . Retrieved 2025-09-15 .
93. **^ ** Gjengset 2021 , p. 155-156.
94. **^ ** Klabnik & Nichols 2023 , pp. 421–423.
95. **^ ** Klabnik & Nichols 2019 , pp. 418–427.
96. **^ ** ["Casting"](https://doc.rust-lang.org/rust-by-example/types/cast.html) . _Rust by Example _ . Retrieved 2025-04-01 .
97. **^ ** Klabnik & Nichols 2023 , p. 378.
98. ^ a b c Klabnik & Nichols 2023 , pp. 192–198.
99. **^ ** Klabnik & Nichols 2023 , p. 98.
100. **^ ** Klabnik & Nichols 2019 , pp. 171–172, 205.
101. ^ a b Klabnik & Nichols 2023 , pp. 191–192.
102. **^ ** Klabnik & Nichols 2019 , pp. 441–442.
103. **^ ** Gjengset 2021 , p. 25.
104. **^ ** Klabnik & Nichols 2023 , [18.2. Using Trait Objects That Allow for Values of Different Types](https://doc.rust-lang.org/book/ch18-02-trait-objects.html) .
105. **^ ** Klabnik & Nichols 2019 , pp. 379–380.
106. **^ ** ["RAII"](https://doc.rust-lang.org/rust-by-example/scope/raii.html) . _Rust by Example _ . [Archived](https://web.archive.org/web/20190421131142/https://doc.rust-lang.org/rust-by-example/scope/raii.html) from the original on 2019-04-21 . Retrieved 2020-11-22 .
107. **^ ** ["Abstraction without overhead: traits in Rust"](https://blog.rust-lang.org/2015/05/11/traits.html) . _Rust Blog _ . [Archived](https://web.archive.org/web/20210923101530/https://blog.rust-lang.org/2015/05/11/traits.html) from the original on 2021-09-23 . Retrieved 2021-10-19 .
108. **^ ** ["Box, stack and heap"](https://doc.rust-lang.org/stable/rust-by-example/std/box.html) . _Rust by Example _ . [Archived](https://web.archive.org/web/20220531114141/https://doc.rust-lang.org/stable/rust-by-example/std/box.html) from the original on 2022-05-31 . Retrieved 2022-06-13 .
109. **^ ** Klabnik & Nichols 2019 , pp. 70–75.
110. **^ ** Klabnik & Nichols 2019 , p. 323.
111. **^ ** Klabnik & Nichols 2023 , pp. 420–429.
112. **^ ** McNamara 2021 , p. 139, 376–379, 395.
113. ^ a b Astrauskas, Vytautas; Matheja, Christoph; Poli, Federico; Müller, Peter; Summers, Alexander J. (2020-11-13). ["How do programmers use unsafe rust?"](https://dl.acm.org/doi/10.1145/3428204) . _Proceedings of the ACM on Programming Languages _ . **4 ** : 1– 27. [doi](https://en.wikipedia.org/wiki/Doi_(identifier)) : [10.1145/3428204](https://doi.org/10.1145%2F3428204) . [hdl](https://en.wikipedia.org/wiki/Hdl_(identifier)) : [20.500.11850/465785](https://hdl.handle.net/20.500.11850%2F465785) . [ISSN](https://en.wikipedia.org/wiki/ISSN_(identifier)) [2475-1421](https://search.worldcat.org/issn/2475-1421) .
114. **^ ** Lattuada, Andrea; Hance, Travis; Cho, Chanhee; Brun, Matthias; Subasinghe, Isitha; Zhou, Yi; Howell, Jon; Parno, Bryan; Hawblitzel, Chris (2023-04-06). ["Verus: Verifying Rust Programs using Linear Ghost Types"](https://dl.acm.org/doi/10.1145/3586037) . _Proceedings of the ACM on Programming Languages _ . **7 ** : 85:286–85:315. [doi](https://en.wikipedia.org/wiki/Doi_(identifier)) : [10.1145/3586037](https://doi.org/10.1145%2F3586037) . [hdl](https://en.wikipedia.org/wiki/Hdl_(identifier)) : [20.500.11850/610518](https://hdl.handle.net/20.500.11850%2F610518) .
115. **^ ** Milano, Mae; Turcotti, Julia; Myers, Andrew C. (2022-06-09). "A flexible type system for fearless concurrency". _Proceedings of the 43rd ACM SIGPLAN International Conference on Programming Language Design and Implementation _ . New York, NY, USA: Association for Computing Machinery. pp. 458– 473. [doi](https://en.wikipedia.org/wiki/Doi_(identifier)) : [10.1145/3519939.3523443](https://doi.org/10.1145%2F3519939.3523443) . [ISBN](https://en.wikipedia.org/wiki/ISBN_(identifier)) [978-1-4503-9265-5](https://en.wikipedia.org/wiki/Special:BookSources/978-1-4503-9265-5) .
116. **^ ** ["Introduction - Learning Rust With Entirely Too Many Linked Lists"](https://rust-unofficial.github.io/too-many-lists/) . _rust-unofficial.github.io _ . Retrieved 2025-08-06 .
117. **^ ** Noble, James; Mackay, Julian; Wrigstad, Tobias (2023-10-16). ["Rusty Links in Local Chains✱"](https://doi.org/10.1145/3611096.3611097) . _Proceedings of the 24th ACM International Workshop on Formal Techniques for Java-like Programs _ . New York, NY, USA: Association for Computing Machinery. pp. 1– 3. [doi](https://en.wikipedia.org/wiki/Doi_(identifier)) : [10.1145/3611096.3611097](https://doi.org/10.1145%2F3611096.3611097) . [ISBN](https://en.wikipedia.org/wiki/ISBN_(identifier)) [979-8-4007-0784-1](https://en.wikipedia.org/wiki/Special:BookSources/979-8-4007-0784-1) .
118. ^ a b Evans, Ana Nora; Campbell, Bradford; Soffa, Mary Lou (2020-10-01). ["Is rust used safely by software developers?"](https://doi.org/10.1145/3377811.3380413) . _Proceedings of the ACM/IEEE 42nd International Conference on Software Engineering _ . New York, NY, USA: Association for Computing Machinery. pp. 246– 257. [arXiv](https://en.wikipedia.org/wiki/ArXiv_(identifier)) : [2007.00752](https://arxiv.org/abs/2007.00752) . [doi](https://en.wikipedia.org/wiki/Doi_(identifier)) : [10.1145/3377811.3380413](https://doi.org/10.1145%2F3377811.3380413) . [ISBN](https://en.wikipedia.org/wiki/ISBN_(identifier)) [978-1-4503-7121-6](https://en.wikipedia.org/wiki/Special:BookSources/978-1-4503-7121-6) .
119. **^ ** ["Behavior considered undefined"](https://doc.rust-lang.org/reference/behavior-considered-undefined.html) . _The Rust Reference _ . Retrieved 2025-08-06 .
120. **^ ** Klabnik & Nichols 2023 , pp. 449–455.
121. **^ ** Gjengset 2021 , pp. 101–102.
122. **^ ** ["Macros By Example"](https://doc.rust-lang.org/reference/macros-by-example.html) . _The Rust Reference _ . [Archived](https://web.archive.org/web/20230421052332/https://doc.rust-lang.org/reference/macros-by-example.html) from the original on 2023-04-21 . Retrieved 2023-04-21 .
123. **^ ** Klabnik & Nichols 2019 , pp. 446–448.
124. **^ ** ["Procedural Macros"](https://doc.rust-lang.org/reference/procedural-macros.html) . _The Rust Programming Language Reference _ . [Archived](https://web.archive.org/web/20201107233444/https://doc.rust-lang.org/reference/procedural-macros.html) from the original on 2020-11-07 . Retrieved 2021-03-23 .
125. **^ ** Klabnik & Nichols 2019 , pp. 449–455.
126. **^ ** Baumgartner, Stefan (2025-05-23). ["Programming language: Rust 2024 is the most comprehensive edition to date"](https://www.heise.de/en/background/Programming-language-Rust-2024-is-the-most-comprehensive-edition-to-date-10393917.html) . _[heise online](https://en.wikipedia.org/wiki/Heise_online) _ . Retrieved 2025-06-28 .
127. ^ a b c Gjengset 2021 , pp. 193–209.
128. **^ ** ["Safe Interoperability between Rust and C++ with CXX"](https://www.infoq.com/news/2020/12/cpp-rust-interop-cxx/) . _InfoQ _ . 2020-12-06. [Archived](https://web.archive.org/web/20210122142035/https://www.infoq.com/news/2020/12/cpp-rust-interop-cxx/) from the original on 2021-01-22 . Retrieved 2021-01-03 .
129. **^ ** Rosenblatt, Seth (2013-04-03). ["Samsung joins Mozilla's quest for Rust"](https://reviews.cnet.com/8301-3514_7-57577639/samsung-joins-mozillas-quest-for-rust/) . [CNET](https://en.wikipedia.org/wiki/CNET) . [Archived](https://web.archive.org/web/20130404142333/http://reviews.cnet.com/8301-3514_7-57577639/samsung-joins-mozillas-quest-for-rust/) from the original on 2013-04-04 . Retrieved 2013-04-05 .
130. **^ ** Brown, Neil (2013-04-17). ["A taste of Rust"](https://lwn.net/Articles/547145/) . _[LWN.net](https://en.wikipedia.org/wiki/LWN.net) _ . [Archived](https://web.archive.org/web/20130426010754/http://lwn.net/Articles/547145/) from the original on 2013-04-26 . Retrieved 2013-04-25 .
131. **^ ** ["Races"](https://doc.rust-lang.org/nomicon/races.html) . _The Rustonomicon _ . [Archived](https://web.archive.org/web/20170710194643/https://doc.rust-lang.org/nomicon/races.html) from the original on 2017-07-10 . Retrieved 2017-07-03 .
132. **^ ** Vandervelden, Thibaut; De Smet, Ruben; Deac, Diana; Steenhaut, Kris; Braeken, An (2024-09-07). ["Overview of Embedded Rust Operating Systems and Frameworks"](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC11398098) . _Sensors _ . **24 ** (17): 5818. [Bibcode](https://en.wikipedia.org/wiki/Bibcode_(identifier)) : [2024Senso..24.5818V](https://ui.adsabs.harvard.edu/abs/2024Senso..24.5818V) . [doi](https://en.wikipedia.org/wiki/Doi_(identifier)) : [10.3390/s24175818](https://doi.org/10.3390%2Fs24175818) . [PMC](https://en.wikipedia.org/wiki/PMC_(identifier)) [11398098](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC11398098) . [PMID](https://en.wikipedia.org/wiki/PMID_(identifier)) [39275729](https://pubmed.ncbi.nlm.nih.gov/39275729) .
133. ^ a b Klabnik & Nichols 2023 , pp. 343–346.
134. **^ ** Gjengset 2021 , p. 6.
135. **^ ** Blandy, Orendorff & Tindall 2021 , pp. 6–8.
136. **^ ** ["Overview of the compiler"](https://rustc-dev-guide.rust-lang.org/overview.html) . _Rust Compiler Development Guide _ . Rust project contributors. [Archived](https://web.archive.org/web/20230531035222/https://rustc-dev-guide.rust-lang.org/overview.html) from the original on 2023-05-31 . Retrieved 2024-11-07 .
137. **^ ** ["Code Generation"](https://rustc-dev-guide.rust-lang.org/backend/codegen.html) . _Rust Compiler Development Guide _ . Rust project contributors . Retrieved 2024-03-03 .
138. **^ ** ["rust-lang/rustc_codegen_gcc"](https://github.com/rust-lang/rustc_codegen_gcc#Motivation) . _[GitHub](https://en.wikipedia.org/wiki/GitHub) _ . The Rust Programming Language. 2024-03-02 . Retrieved 2024-03-03 .
139. **^ ** ["rust-lang/rustc_codegen_cranelift"](https://github.com/rust-lang/rustc_codegen_cranelift) . _[GitHub](https://en.wikipedia.org/wiki/GitHub) _ . The Rust Programming Language. 2024-03-02 . Retrieved 2024-03-03 .
140. ^ a b c Perkel, Jeffrey M. (2020-12-01). ["Why scientists are turning to Rust"](https://www.nature.com/articles/d41586-020-03382-2) . _[Nature](https://en.wikipedia.org/wiki/Nature_(journal)) _ . **588 ** (7836): 185– 186. [Bibcode](https://en.wikipedia.org/wiki/Bibcode_(identifier)) : [2020Natur.588..185P](https://ui.adsabs.harvard.edu/abs/2020Natur.588..185P) . [doi](https://en.wikipedia.org/wiki/Doi_(identifier)) : [10.1038/d41586-020-03382-2](https://doi.org/10.1038%2Fd41586-020-03382-2) . [PMID](https://en.wikipedia.org/wiki/PMID_(identifier)) [33262490](https://pubmed.ncbi.nlm.nih.gov/33262490) . [S2CID](https://en.wikipedia.org/wiki/S2CID_(identifier)) [227251258](https://api.semanticscholar.org/CorpusID:227251258) . [Archived](https://web.archive.org/web/20220506040523/https://www.nature.com/articles/d41586-020-03382-2) from the original on 2022-05-06 . Retrieved 2022-05-15 .
141. **^ ** Simone, Sergio De (2019-04-18). ["Rust 1.34 Introduces Alternative Registries for Non-Public Crates"](https://www.infoq.com/news/2019/04/rust-1.34-additional-registries) . _InfoQ _ . [Archived](https://web.archive.org/web/20220714164454/https://www.infoq.com/news/2019/04/rust-1.34-additional-registries) from the original on 2022-07-14 . Retrieved 2022-07-14 .
142. **^ ** ["Why Cargo Exists - The Cargo Book"](https://doc.rust-lang.org/cargo/guide/why-cargo-exists.html) . _doc.rust-lang.org _ . Retrieved 2026-03-22 .
143. **^ ** ["Cargo.toml vs Cargo.lock - The Cargo Book"](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html) . _doc.rust-lang.org _ . Retrieved 2026-03-22 .
144. **^ ** Klabnik & Nichols 2019 , pp. 511–512.
145. **^ ** ["rust-lang/rust-clippy"](https://github.com/rust-lang/rust-clippy) . The Rust Programming Language. 2026-01-04 . Retrieved 2026-01-04 .
146. **^ ** ["Clippy Lints"](https://rust-lang.github.io/rust-clippy/master/index.html) . _The Rust Programming Language _ . Retrieved 2023-11-30 .
147. **^ ** Klabnik & Nichols 2019 , Appendix G – How Rust is Made and "Nightly Rust"
148. **^ ** Blandy, Orendorff & Tindall 2021 , pp. 176–177.
149. **^ ** Klabnik & Nichols 2023 , p. 623.
150. **^ ** Anderson, Tim (2021-11-30). ["Can Rust save the planet? Why, and why not"](https://www.theregister.com/2021/11/30/aws_reinvent_rust/) . _[The Register](https://en.wikipedia.org/wiki/The_Register) _ . [Archived](https://web.archive.org/web/20220711001629/https://www.theregister.com/2021/11/30/aws_reinvent_rust/) from the original on 2022-07-11 . Retrieved 2022-07-11 .
151. **^ ** Yegulalp, Serdar (2021-10-06). ["What is the Rust language? Safe, fast, and easy software development"](https://www.infoworld.com/article/3218074/what-is-rust-safe-fast-and-easy-software-development.html) . _[InfoWorld](https://en.wikipedia.org/wiki/InfoWorld) _ . [Archived](https://web.archive.org/web/20220624101013/https://www.infoworld.com/article/3218074/what-is-rust-safe-fast-and-easy-software-development.html) from the original on 2022-06-24 . Retrieved 2022-06-25 .
152. **^ ** McNamara 2021 , p. 11.
153. ^ a b Popescu, Natalie; Xu, Ziyang; Apostolakis, Sotiris; August, David I.; Levy, Amit (2021-10-15). ["Safer at any speed: automatic context-aware safety enhancement for Rust"](https://doi.org/10.1145%2F3485480) . _Proceedings of the ACM on Programming Languages _ . **5 ** (OOPSLA). Section 2. [doi](https://en.wikipedia.org/wiki/Doi_(identifier)) : [10.1145/3485480](https://doi.org/10.1145%2F3485480) . [S2CID](https://en.wikipedia.org/wiki/S2CID_(identifier)) [238212612](https://api.semanticscholar.org/CorpusID:238212612) . p. 5: We observe a large variance in the overheads of checked indexing: 23.6% of benchmarks do report significant performance hits from checked indexing, but 64.5% report little-to-no impact and, surprisingly, 11.8% report improved performance ... Ultimately, while unchecked indexing can improve performance, most of the time it does not.
154. **^ ** McNamara 2021 , p. 19, 27.
155. **^ ** Couprie, Geoffroy (2015). "Nom, A Byte oriented, streaming, Zero copy, Parser Combinators Library in Rust". _2015 IEEE Security and Privacy Workshops _ . pp. 142– 148. [doi](https://en.wikipedia.org/wiki/Doi_(identifier)) : [10.1109/SPW.2015.31](https://doi.org/10.1109%2FSPW.2015.31) . [ISBN](https://en.wikipedia.org/wiki/ISBN_(identifier)) [978-1-4799-9933-0](https://en.wikipedia.org/wiki/Special:BookSources/978-1-4799-9933-0) . [S2CID](https://en.wikipedia.org/wiki/S2CID_(identifier)) [16608844](https://api.semanticscholar.org/CorpusID:16608844) .
156. **^ ** McNamara 2021 , p. 20.
157. **^ ** ["Code generation"](https://doc.rust-lang.org/reference/attributes/codegen.html) . _The Rust Reference _ . [Archived](https://web.archive.org/web/20221009202615/https://doc.rust-lang.org/reference/attributes/codegen.html) from the original on 2022-10-09 . Retrieved 2022-10-09 .
158. **^ ** ["How Fast Is Rust?"](https://doc.rust-lang.org/1.0.0/complement-lang-faq.html#how-fast-is-rust?) . _The Rust Programming Language FAQ _ . [Archived](https://web.archive.org/web/20201028102013/https://doc.rust-lang.org/1.0.0/complement-lang-faq.html#how-fast-is-rust?) from the original on 2020-10-28 . Retrieved 2019-04-11 .
159. **^ ** Farshin, Alireza; Barbette, Tom; Roozbeh, Amir; Maguire, Gerald Q. Jr; Kostić, Dejan (2021). "PacketMill: Toward per-Core 100-GBPS networking". [Proceedings of the 26th ACM International Conference on Architectural Support for Programming Languages and Operating Systems](https://dlnext.acm.org/doi/abs/10.1145/3445814.3446724) . pp. 1– 17. [doi](https://en.wikipedia.org/wiki/Doi_(identifier)) : [10.1145/3445814.3446724](https://doi.org/10.1145%2F3445814.3446724) . [ISBN](https://en.wikipedia.org/wiki/ISBN_(identifier)) [9781450383172](https://en.wikipedia.org/wiki/Special:BookSources/9781450383172) . [S2CID](https://en.wikipedia.org/wiki/S2CID_(identifier)) [231949599](https://api.semanticscholar.org/CorpusID:231949599) . [Archived](https://web.archive.org/web/20220712060927/https://dlnext.acm.org/doi/abs/10.1145/3445814.3446724) from the original on 2022-07-12 . Retrieved 2022-07-12 . ... While some compilers (e.g., Rust) support structure reordering [82], C & C++ compilers are forbidden to reorder data structures (e.g., struct or class) [74] ...
160. **^ ** Gjengset 2021 , p. 22.
161. **^ ** Shankland, Stephen (2016-07-12). ["Firefox will get overhaul in bid to get you interested again"](https://www.cnet.com/tech/services-and-software/firefox-mozilla-gets-overhaul-in-a-bid-to-get-you-interested-again/) . [CNET](https://en.wikipedia.org/wiki/CNET) . [Archived](https://web.archive.org/web/20220714172029/https://www.cnet.com/tech/services-and-software/firefox-mozilla-gets-overhaul-in-a-bid-to-get-you-interested-again/) from the original on 2022-07-14 . Retrieved 2022-07-14 .
162. **^ ** Security Research Team (2013-10-04). ["ZeroMQ: Helping us Block Malicious Domains in Real Time"](https://web.archive.org/web/20230513161542/https://umbrella.cisco.com/blog/zeromq-helping-us-block-malicious-domains) . _Cisco Umbrella _ . Archived from [the original](https://umbrella.cisco.com/blog/zeromq-helping-us-block-malicious-domains) on 2023-05-13 . Retrieved 2023-05-13 .
163. **^ ** Cimpanu, Catalin (2019-10-15). ["AWS to sponsor Rust project"](https://www.zdnet.com/article/aws-to-sponsor-rust-project/) . _[ZDNET](https://en.wikipedia.org/wiki/ZDNET) _ . Retrieved 2024-07-17 .
164. **^ ** Nichols, Shaun (2018-06-27). ["Microsoft's next trick? Kicking things out of the cloud to Azure IoT Edge"](https://www.theregister.co.uk/2018/06/27/microsofts_next_cloud_trick_kicking_things_out_of_the_cloud_to_azure_iot_edge/) . _[The Register](https://en.wikipedia.org/wiki/The_Register) _ . [Archived](https://web.archive.org/web/20190927092433/https://www.theregister.co.uk/2018/06/27/microsofts_next_cloud_trick_kicking_things_out_of_the_cloud_to_azure_iot_edge/) from the original on 2019-09-27 . Retrieved 2019-09-27 .
165. **^ ** Tung, Liam (2020-04-30). ["Microsoft: Why we used programming language Rust over Go for WebAssembly on Kubernetes app"](https://www.zdnet.com/article/microsoft-why-we-used-programming-language-rust-over-go-for-webassembly-on-kubernetes-app/) . _[ZDNET](https://en.wikipedia.org/wiki/ZDNET) _ . [Archived](https://web.archive.org/web/20220421043549/https://www.zdnet.com/article/microsoft-why-we-used-programming-language-rust-over-go-for-webassembly-on-kubernetes-app/) from the original on 2022-04-21 . Retrieved 2022-04-21 .
166. **^ ** Claburn, Thomas (2022-09-20). ["In Rust We Trust: Microsoft Azure CTO shuns C and C++"](https://www.theregister.com/2022/09/20/rust_microsoft_c/) . _[The Register](https://en.wikipedia.org/wiki/The_Register) _ . Retrieved 2024-07-07 .
167. **^ ** Simone, Sergio De (2019-03-10). ["NPM Adopted Rust to Remove Performance Bottlenecks"](https://www.infoq.com/news/2019/03/rust-npm-performance/) . _InfoQ _ . [Archived](https://web.archive.org/web/20231119135434/https://www.infoq.com/news/2019/03/rust-npm-performance/) from the original on 2023-11-19 . Retrieved 2023-11-20 .
168. **^ ** Lyu, Shing (2020). ["Welcome to the World of Rust"](https://doi.org/10.1007/978-1-4842-5599-5_1) . In Lyu, Shing (ed.). _Practical Rust Projects: Building Game, Physical Computing, and Machine Learning Applications _ . Berkeley, CA: Apress. pp. 1– 8. [doi](https://en.wikipedia.org/wiki/Doi_(identifier)) : [10.1007/978-1-4842-5599-5_1](https://doi.org/10.1007%2F978-1-4842-5599-5_1) . [ISBN](https://en.wikipedia.org/wiki/ISBN_(identifier)) [978-1-4842-5599-5](https://en.wikipedia.org/wiki/Special:BookSources/978-1-4842-5599-5) . Retrieved 2023-11-29 .
169. **^ ** Lyu, Shing (2021). ["Rust in the Web World"](https://doi.org/10.1007/978-1-4842-6589-5_1) . In Lyu, Shing (ed.). _Practical Rust Web Projects: Building Cloud and Web-Based Applications _ . Berkeley, CA: Apress. pp. 1– 7. [doi](https://en.wikipedia.org/wiki/Doi_(identifier)) : [10.1007/978-1-4842-6589-5_1](https://doi.org/10.1007%2F978-1-4842-6589-5_1) . [ISBN](https://en.wikipedia.org/wiki/ISBN_(identifier)) [978-1-4842-6589-5](https://en.wikipedia.org/wiki/Special:BookSources/978-1-4842-6589-5) . Retrieved 2023-11-29 .
170. ^ a b Li, Hongyu; Guo, Liwei; Yang, Yexuan; Wang, Shangguang; Xu, Mengwei (2024-06-30). ["An Empirical Study of Rust-for-Linux: The Success, Dissatisfaction, and Compromise"](https://www.usenix.org/publications/loginonline/empirical-study-rust-linux-success-dissatisfaction-and-compromise) . _[USENIX](https://en.wikipedia.org/wiki/USENIX) _ . Retrieved 2024-11-28 .
171. **^ ** Corbet, Jonathan (2022-10-13). ["A first look at Rust in the 6.1 kernel"](https://lwn.net/Articles/910762/) . _[LWN.net](https://en.wikipedia.org/wiki/LWN.net) _ . [Archived](https://web.archive.org/web/20231117141103/https://lwn.net/Articles/910762/) from the original on 2023-11-17 . Retrieved 2023-11-11 .
172. **^ ** Vaughan-Nichols, Steven (2021-12-07). ["Rust takes a major step forward as Linux's second official language"](https://www.zdnet.com/article/rust-takes-a-major-step-forward-as-linuxs-second-official-language/) . _[ZDNET](https://en.wikipedia.org/wiki/ZDNET) _ . Retrieved 2024-11-27 .
173. **^ ** Corbet, Jonathan (2025-12-10). ["The (successful) end of the kernel Rust experiment"](https://lwn.net/Articles/1049831/) . _LWN.net _ . Retrieved 2025-12-10 .
174. **^ ** Amadeo, Ron (2021-04-07). ["Google is now writing low-level Android code in Rust"](https://arstechnica.com/gadgets/2021/04/google-is-now-writing-low-level-android-code-in-rust/) . _Ars Technica _ . [Archived](https://web.archive.org/web/20210408001446/https://arstechnica.com/gadgets/2021/04/google-is-now-writing-low-level-android-code-in-rust/) from the original on 2021-04-08 . Retrieved 2022-04-21 .
175. **^ ** Darkcrizt (2021-04-02). ["Google Develops New Bluetooth Stack for Android, Written in Rust"](https://web.archive.org/web/20210825165930/https://blog.desdelinux.net/en/google-develops-a-new-bluetooth-stack-for-android-written-in-rust/) . _Desde Linux _ . Archived from [the original](https://blog.desdelinux.net/en/google-develops-a-new-bluetooth-stack-for-android-written-in-rust/) on 2021-08-25 . Retrieved 2024-08-31 .
176. **^ ** Claburn, Thomas (2023-04-27). ["Microsoft is rewriting core Windows libraries in Rust"](https://www.theregister.com/2023/04/27/microsoft_windows_rust/) . _[The Register](https://en.wikipedia.org/wiki/The_Register) _ . [Archived](https://web.archive.org/web/20230513082735/https://www.theregister.com/2023/04/27/microsoft_windows_rust/) from the original on 2023-05-13 . Retrieved 2023-05-13 .
177. **^ ** Proven, Liam (2023-12-01). ["Small but mighty, 9Front's 'Humanbiologics' is here for the truly curious"](https://www.theregister.com/2023/12/01/9front_humanbiologics/) . _[The Register](https://en.wikipedia.org/wiki/The_Register) _ . Retrieved 2024-03-07 .
178. **^ ** Yegulalp, Serdar (2016-03-21). ["Rust's Redox OS could show Linux a few new tricks"](https://web.archive.org/web/20160321192838/http://www.infoworld.com/article/3046100/open-source-tools/rusts-redox-os-could-show-linux-a-few-new-tricks.html) . _[InfoWorld](https://en.wikipedia.org/wiki/InfoWorld) _ . Archived from [the original](http://www.infoworld.com/article/3046100/open-source-tools/rusts-redox-os-could-show-linux-a-few-new-tricks.html) on 2016-03-21 . Retrieved 2016-03-21 .
179. **^ ** Anderson, Tim (2021-01-14). ["Another Rust-y OS: Theseus joins Redox in pursuit of safer, more resilient systems"](https://www.theregister.com/2021/01/14/rust_os_theseus/) . _[The Register](https://en.wikipedia.org/wiki/The_Register) _ . [Archived](https://web.archive.org/web/20220714112619/https://www.theregister.com/2021/01/14/rust_os_theseus/) from the original on 2022-07-14 . Retrieved 2022-07-14 .
180. **^ ** Boos, Kevin; Liyanage, Namitha; Ijaz, Ramla; Zhong, Lin (2020). [Theseus: an Experiment in Operating System Structure and State Management](https://www.usenix.org/conference/osdi20/presentation/boos) . pp. 1– 19. [ISBN](https://en.wikipedia.org/wiki/ISBN_(identifier)) [978-1-939133-19-9](https://en.wikipedia.org/wiki/Special:BookSources/978-1-939133-19-9) . [Archived](https://web.archive.org/web/20230513164135/https://www.usenix.org/conference/osdi20/presentation/boos) from the original on 2023-05-13 . Retrieved 2023-05-13 .
181. **^ ** Zhang, HanDong (2023-01-31). ["2022 Review | The adoption of Rust in Business"](https://rustmagazine.org/issue-1/2022-review-the-adoption-of-rust-in-business/) . _Rust Magazine _ . Retrieved 2023-02-07 .
182. **^ ** Sei, Mark (2018-10-10). ["Fedora 29 new features: Startis now officially in Fedora"](https://www.marksei.com/fedora-29-new-features-startis/) . _Marksei, Weekly sysadmin pills _ . [Archived](https://web.archive.org/web/20190413075055/https://www.marksei.com/fedora-29-new-features-startis/) from the original on 2019-04-13 . Retrieved 2019-05-13 .
183. **^ ** Proven, Liam (2022-07-12). ["Oracle Linux 9 released, with some interesting additions"](https://www.theregister.com/2022/07/12/oracle_linux_9/) . _[The Register](https://en.wikipedia.org/wiki/The_Register) _ . [Archived](https://web.archive.org/web/20220714073400/https://www.theregister.com/2022/07/12/oracle_linux_9/) from the original on 2022-07-14 . Retrieved 2022-07-14 .
184. **^ ** Proven, Liam (2023-02-02). ["System76 teases features coming in homegrown Rust-based desktop COSMIC"](https://www.theregister.com/2023/02/02/system76_cosmic_xfce_updates/) . _[The Register](https://en.wikipedia.org/wiki/The_Register) _ . [Archived](https://web.archive.org/web/20240717145511/https://www.theregister.com/2023/02/02/system76_cosmic_xfce_updates/) from the original on 2024-07-17 . Retrieved 2024-07-17 .
185. **^ ** Hu, Vivian (2020-06-12). ["Deno Is Ready for Production"](https://www.infoq.com/news/2020/06/deno-1-ready-production/) . _InfoQ _ . [Archived](https://web.archive.org/web/20200701105007/https://www.infoq.com/news/2020/06/deno-1-ready-production/) from the original on 2020-07-01 . Retrieved 2022-07-14 .
186. **^ ** Abrams, Lawrence (2021-02-06). ["This Flash Player emulator lets you securely play your old games"](https://www.bleepingcomputer.com/news/software/this-flash-player-emulator-lets-you-securely-play-your-old-games/) . _[Bleeping Computer](https://en.wikipedia.org/wiki/Bleeping_Computer) _ . [Archived](https://web.archive.org/web/20211225124131/https://www.bleepingcomputer.com/news/software/this-flash-player-emulator-lets-you-securely-play-your-old-games/) from the original on 2021-12-25 . Retrieved 2021-12-25 .
187. **^ ** Kharif, Olga (2020-10-17). ["Ethereum Blockchain Killer Goes By Unassuming Name of Polkadot"](https://www.bloomberg.com/news/articles/2020-10-17/ethereum-blockchain-killer-goes-by-unassuming-name-of-polkadot) . _[Bloomberg News](https://en.wikipedia.org/wiki/Bloomberg_News) _ . [Bloomberg L.P.](https://en.wikipedia.org/wiki/Bloomberg_L.P.) [Archived](https://web.archive.org/web/20201017192915/https://www.bloomberg.com/news/articles/2020-10-17/ethereum-blockchain-killer-goes-by-unassuming-name-of-polkadot) from the original on 2020-10-17 . Retrieved 2021-07-14 .
188. **^ ** Keizer, Gregg (2016-10-31). ["Mozilla plans to rejuvenate Firefox in 2017"](https://www.computerworld.com/article/3137050/mozilla-plans-to-rejuvenate-firefox-in-2017.html) . _[Computerworld](https://en.wikipedia.org/wiki/Computerworld) _ . [Archived](https://web.archive.org/web/20230513020437/https://www.computerworld.com/article/3137050/mozilla-plans-to-rejuvenate-firefox-in-2017.html) from the original on 2023-05-13 . Retrieved 2023-05-13 .
189. **^ ** Claburn, Thomas (2023-01-12). ["Google polishes Chromium code with a layer of Rust"](https://www.theregister.com/2023/01/12/google_chromium_rust/) . _[The Register](https://en.wikipedia.org/wiki/The_Register) _ . Retrieved 2024-07-17 .
190. **^ ** Jansens, Dana (2023-01-12). ["Supporting the Use of Rust in the Chromium Project"](https://security.googleblog.com/2023/01/supporting-use-of-rust-in-chromium.html) . _Google Online Security Blog _ . [Archived](https://web.archive.org/web/20230113004438/https://security.googleblog.com/2023/01/supporting-use-of-rust-in-chromium.html) from the original on 2023-01-13 . Retrieved 2023-11-12 .
191. ^ a b c ["2025 Stack Overflow Developer Survey – Technology"](https://survey.stackoverflow.co/2025/technology) . _[Stack Overflow](https://en.wikipedia.org/wiki/Stack_Overflow) _ . Retrieved 2025-08-09 .
192. **^ ** Claburn, Thomas (2022-06-23). ["Linus Torvalds says Rust is coming to the Linux kernel"](https://www.theregister.com/2022/06/23/linus_torvalds_rust_linux_kernel/) . _[The Register](https://en.wikipedia.org/wiki/The_Register) _ . [Archived](https://web.archive.org/web/20220728221531/https://www.theregister.com/2022/06/23/linus_torvalds_rust_linux_kernel/) from the original on 2022-07-28 . Retrieved 2022-07-15 .
193. **^ ** Jung, Ralf; Jourdan, Jacques-Henri; Krebbers, Robbert; Dreyer, Derek (2017-12-27). ["RustBelt: securing the foundations of the Rust programming language"](https://dl.acm.org/doi/10.1145/3158154) . _Proceedings of the ACM on Programming Languages _ . **2 ** (POPL): 1– 34. [doi](https://en.wikipedia.org/wiki/Doi_(identifier)) : [10.1145/3158154](https://doi.org/10.1145%2F3158154) . [hdl](https://en.wikipedia.org/wiki/Hdl_(identifier)) : [21.11116/0000-0003-34C6-3](https://hdl.handle.net/21.11116%2F0000-0003-34C6-3) . [ISSN](https://en.wikipedia.org/wiki/ISSN_(identifier)) [2475-1421](https://search.worldcat.org/issn/2475-1421) .
194. **^ ** Popescu, Natalie; Xu, Ziyang; Apostolakis, Sotiris; August, David I.; Levy, Amit (2021-10-20). ["Safer at any speed: automatic context-aware safety enhancement for Rust"](https://doi.org/10.1145%2F3485480) . _Proceedings of the ACM on Programming Languages _ . **5 ** (OOPSLA): 1– 23. [doi](https://en.wikipedia.org/wiki/Doi_(identifier)) : [10.1145/3485480](https://doi.org/10.1145%2F3485480) . [ISSN](https://en.wikipedia.org/wiki/ISSN_(identifier)) [2475-1421](https://search.worldcat.org/issn/2475-1421) .
195. **^ ** Blanco-Cuaresma, Sergi; Bolmont, Emeline (2017-05-30). ["What can the programming language Rust do for astrophysics?"](https://www.cambridge.org/core/journals/proceedings-of-the-international-astronomical-union/article/what-can-the-programming-language-rust-do-for-astrophysics/B51B6DF72B7641F2352C05A502F3D881) . _[Proceedings of the International Astronomical Union](https://en.wikipedia.org/wiki/Proceedings_of_the_International_Astronomical_Union) _ . **12 ** (S325): 341– 344. [arXiv](https://en.wikipedia.org/wiki/ArXiv_(identifier)) : [1702.02951](https://arxiv.org/abs/1702.02951) . [Bibcode](https://en.wikipedia.org/wiki/Bibcode_(identifier)) : [2017IAUS..325..341B](https://ui.adsabs.harvard.edu/abs/2017IAUS..325..341B) . [doi](https://en.wikipedia.org/wiki/Doi_(identifier)) : [10.1017/S1743921316013168](https://doi.org/10.1017%2FS1743921316013168) . [ISSN](https://en.wikipedia.org/wiki/ISSN_(identifier)) [1743-9213](https://search.worldcat.org/issn/1743-9213) . [S2CID](https://en.wikipedia.org/wiki/S2CID_(identifier)) [7857871](https://api.semanticscholar.org/CorpusID:7857871) . [Archived](https://web.archive.org/web/20220625140128/https://www.cambridge.org/core/journals/proceedings-of-the-international-astronomical-union/article/what-can-the-programming-language-rust-do-for-astrophysics/B51B6DF72B7641F2352C05A502F3D881) from the original on 2022-06-25 . Retrieved 2022-06-25 .
196. **^ ** Wallach, Dan. ["TRACTOR: Translating All C to Rust"](https://www.darpa.mil/research/programs/translating-all-c-to-rust) . [DARPA](https://en.wikipedia.org/wiki/DARPA) . Retrieved 2025-08-03 .
197. **^ ** Klabnik & Nichols 2019 , p. 4.
198. **^ ** ["Getting Started"](https://www.rust-lang.org/learn/get-started#ferris) . _The Rust Programming Language _ . [Archived](https://web.archive.org/web/20201101145703/https://www.rust-lang.org/learn/get-started#ferris) from the original on 2020-11-01 . Retrieved 2020-10-11 .
199. **^ ** The Rust Survey Team (2025-02-13). ["2024 State of Rust Survey Results"](https://blog.rust-lang.org/2025/02/13/2024-State-Of-Rust-Survey-results.html) . _The Rust Programming Language _ . Retrieved 2025-09-07 .
200. **^ ** Tung, Liam (2021-02-08). ["The Rust programming language just took a huge step forwards"](https://www.zdnet.com/article/the-rust-programming-language-just-took-a-huge-step-forwards/) . _[ZDNET](https://en.wikipedia.org/wiki/ZDNET) _ . [Archived](https://web.archive.org/web/20220714105527/https://www.zdnet.com/article/the-rust-programming-language-just-took-a-huge-step-forwards/) from the original on 2022-07-14 . Retrieved 2022-07-14 .
201. **^ ** Krill, Paul (2021-02-09). ["Rust language moves to independent foundation"](https://www.infoworld.com/article/3606774/rust-language-moves-to-independent-foundation.html) . _[InfoWorld](https://en.wikipedia.org/wiki/InfoWorld) _ . [Archived](https://web.archive.org/web/20210410161528/https://www.infoworld.com/article/3606774/rust-language-moves-to-independent-foundation.html) from the original on 2021-04-10 . Retrieved 2021-04-10 .
202. **^ ** Vaughan-Nichols, Steven J. (2021-04-09). ["AWS's Shane Miller to head the newly created Rust Foundation"](https://www.zdnet.com/article/awss-shane-miller-to-head-the-newly-created-rust-foundation/) . _[ZDNET](https://en.wikipedia.org/wiki/ZDNET) _ . [Archived](https://web.archive.org/web/20210410031305/https://www.zdnet.com/article/awss-shane-miller-to-head-the-newly-created-rust-foundation/) from the original on 2021-04-10 . Retrieved 2021-04-10 .
203. **^ ** Vaughan-Nichols, Steven J. (2021-11-17). ["Rust Foundation appoints Rebecca Rumbul as executive director"](https://www.zdnet.com/article/rust-foundation-appoints-rebecca-rumbul-as-executive-director/) . _[ZDNET](https://en.wikipedia.org/wiki/ZDNET) _ . [Archived](https://web.archive.org/web/20211118062346/https://www.zdnet.com/article/rust-foundation-appoints-rebecca-rumbul-as-executive-director/) from the original on 2021-11-18 . Retrieved 2021-11-18 .
204. **^ ** ["Governance"](https://www.rust-lang.org/governance) . _The Rust Programming Language _ . [Archived](https://web.archive.org/web/20251002151554/https://rust-lang.org/governance/) from the original on 2025-10-02 . Retrieved 2025-11-19 .
205. **^ ** ["Introducing the Rust Leadership Council"](https://blog.rust-lang.org/2023/06/20/introducing-leadership-council.html) . _Rust Blog _ . 2023-06-20 . Retrieved 2024-01-12 .

## External links

[ [edit](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&action=edit&section=54) ]

Wikibooks has a book on the topic of: _**[Rust for the Novice Programmer](https://en.wikibooks.org/wiki/Rust_for_the_Novice_Programmer) ** _

- [Official website](https://rust-lang.org/)
- [Source code](https://github.com/rust-lang/rust) on [GitHub](https://en.wikipedia.org/wiki/GitHub)
- [Documentation](https://doc.rust-lang.org/stable/)

Retrieved from " [https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&oldid=1346393842](https://en.wikipedia.org/w/index.php?title=Rust_(programming_language)&oldid=1346393842) "

[Categories](https://en.wikipedia.org/wiki/Help:Category) :
- [Rust (programming language)](https://en.wikipedia.org/wiki/Category:Rust_(programming_language))
- [Compiled programming languages](https://en.wikipedia.org/wiki/Category:Compiled_programming_languages)
- [Concurrent programming languages](https://en.wikipedia.org/wiki/Category:Concurrent_programming_languages)
- [Free and open source compilers](https://en.wikipedia.org/wiki/Category:Free_and_open_source_compilers)
- [Free software projects](https://en.wikipedia.org/wiki/Category:Free_software_projects)
- [Functional languages](https://en.wikipedia.org/wiki/Category:Functional_languages)
- [High-level programming languages](https://en.wikipedia.org/wiki/Category:High-level_programming_languages)
- [Mozilla](https://en.wikipedia.org/wiki/Category:Mozilla)
- [Multi-paradigm programming languages](https://en.wikipedia.org/wiki/Category:Multi-paradigm_programming_languages)
- [Pattern matching programming languages](https://en.wikipedia.org/wiki/Category:Pattern_matching_programming_languages)
- [Procedural programming languages](https://en.wikipedia.org/wiki/Category:Procedural_programming_languages)
- [Programming languages created in 2015](https://en.wikipedia.org/wiki/Category:Programming_languages_created_in_2015)
- [Software using the Apache license](https://en.wikipedia.org/wiki/Category:Software_using_the_Apache_license)
- [Software using the MIT license](https://en.wikipedia.org/wiki/Category:Software_using_the_MIT_license)
- [Statically typed programming languages](https://en.wikipedia.org/wiki/Category:Statically_typed_programming_languages)
- [Systems programming languages](https://en.wikipedia.org/wiki/Category:Systems_programming_languages)

Hidden categories:
- [CS1 maint: deprecated archival service](https://en.wikipedia.org/wiki/Category:CS1_maint:_deprecated_archival_service)
- [Articles with short description](https://en.wikipedia.org/wiki/Category:Articles_with_short_description)
- [Short description is different from Wikidata](https://en.wikipedia.org/wiki/Category:Short_description_is_different_from_Wikidata)
- [Good articles](https://en.wikipedia.org/wiki/Category:Good_articles)
- [Use American English from February 2026](https://en.wikipedia.org/wiki/Category:Use_American_English_from_February_2026)
- [All Wikipedia articles written in American English](https://en.wikipedia.org/wiki/Category:All_Wikipedia_articles_written_in_American_English)
- [Use mdy dates from July 2022](https://en.wikipedia.org/wiki/Category:Use_mdy_dates_from_July_2022)
- [Articles with example C++ code](https://en.wikipedia.org/wiki/Category:Articles_with_example_C%2B%2B_code)
- [Articles with excerpts](https://en.wikipedia.org/wiki/Category:Articles_with_excerpts)
- [Articles containing potentially dated statements from 2026](https://en.wikipedia.org/wiki/Category:Articles_containing_potentially_dated_statements_from_2026)
- [All articles containing potentially dated statements](https://en.wikipedia.org/wiki/Category:All_articles_containing_potentially_dated_statements)
- [Pages using infobox mapframe with missing coordinates](https://en.wikipedia.org/wiki/Category:Pages_using_infobox_mapframe_with_missing_coordinates)
- [Articles containing potentially dated statements from November 2025](https://en.wikipedia.org/wiki/Category:Articles_containing_potentially_dated_statements_from_November_2025)
- [Pages using Sister project links with hidden wikidata](https://en.wikipedia.org/wiki/Category:Pages_using_Sister_project_links_with_hidden_wikidata)
- [Articles with example Rust code](https://en.wikipedia.org/wiki/Category:Articles_with_example_Rust_code)