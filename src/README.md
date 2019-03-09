# Lang

A programming language. Someday it may have a name; for now it's just a little side project of mine.

## Dreams

Things I want to try to have at some point:
- Dependent types, sort of, maybe. I don't like how you always have to prove stuff in those other languages, so I'm not sure about that. But the idea of types as first-class values is something I'm drawn to.
- A "userspace" type system, where you can declare types using source code. That's for if the language is more rust-like and has sized and unsized types. It would probably have to be compiled for any of that to make sense.
- Whatever it is, some way to parse a graphql schema and turn it into types in the type system, so you can create a query and then the result of that query will have a type.
- Union types. This could be one of the types that is implementable in user code using a hashset. Maybe stuff like this could be implemented in Rust instead of in this language, that would still be pretty cool.
- Have structural types including records and tuples, and then have a uniform way to create newtypes: `newtype Foo of {bar: String}`. That would be pretty cool. And then there would be a uniform way of constructing and deconstructing values of that type, using (maybe) some implicit coercions if it doesn't totally negate the reason you would use a newtype in the first place.
- Having enums like Rust would be cool. Or maybe this:
    ```
    newtype Foo of i32;

    // this is a discriminated union. It might be possible to 
    // Question, should this be `newtype Bar of union { ... }`?
    union Bar {
        // existing primitive type
        String,
        // a type we declared above
        Foo,
        // declaring another variant, which will be its own type as Bar.Baz
        newtype Baz,
    }
    ```
- The enums/unions thing is a place where coercions could come in handy, especially if this ends up being more Rust-like and different types have different representations in memory instead of this Value enum we have. Then there would be difference between the representation of plain old `T` and a `T` in a `union {T, U}`.
- Not having semicolons and curly brace blocks would be nice. That would require writing my own tokenizer, which would be more work, and would make each new language construct more work to add.

## More Mid-Distance Goals

These are things that are more practical steps to take before the really pie-in-the-sky dreams in the above section. They're still far away though - there's probably a non-trivial amount of work to get there.
- A type system and a type checker. Probably just a simple type system, like how the simply-typed lambda calculus is a simple type system. It wouldn't support any kind of polymorphic functions, just simple builtin primitive types and records and tuples.
- Functions. If I can do cool stuff from first-year computer science, that would be cool. Stuff like lists and recursion.
