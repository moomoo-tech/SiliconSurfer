Skip to content

You signed in with another tab or window. Reload to refresh your session.
You signed out in another tab or window. Reload to refresh your session.
You switched accounts on another tab or window. Reload to refresh your session.

Dismiss alert

{{ message }}

nickel-org

/
**
nickel.rs
**

Public

BranchesTags

## Folders and files

NameName
Last commit message

Last commit date

## Latest commit

## History

647 Commits

examples

examples

src

src

tests

tests

.appveyor.yml


.clog.toml

.clog.toml

.gitattributes


.gitignore

.gitignore

.travis.yml


Cargo.toml

Cargo.toml

LICENSE

LICENSE

README.md

README.md

changelog.md


contributing.md


## Repository files navigation

# [nickel.rs](http://nickel-org.github.io)

[nickel.rs](http://nickel-org.github.io) is a simple and lightweight foundation for web applications written in Rust. Its API is inspired by the popular express framework for JavaScript.

## Hello world

```
#[macro_use] extern crate nickel;

use nickel::{Nickel, HttpRouter};

fn main() {
let mut server = Nickel::new();
server.get("**", middleware!("Hello World"));
server.listen("127.0.0.1:6767");
}
```

### Dependencies

You'll need to create a _Cargo.toml_ that looks like this;

```
[package]

name = "my-nickel-app"
version = "0.0.1"
authors = ["yourname"]

[dependencies.nickel]
version = "*"
# If you are using the 'nightly' rust channel you can uncomment
# the line below to activate unstable features
# features = ["unstable"]

# Some examples require the `rustc_serialize` crate, which will
# require uncommenting the lines below
# [dependencies]
# rustc-serialize = "*"
```

You can then compile this using _Cargo build_ and run it using _Cargo run_. After it's running you should visit [http://localhost:6767](http://localhost:6767) to see your hello world!

## More examples

More examples can be found in the examples directory and the full documentation can be [found here](https://docs.rs/nickel/) .

## Contributing

[nickel.rs](http://nickel-org.github.io) is a community effort. We welcome new contributors with open arms. Please read the contributing guide here first.

If you're looking for inspiration, there's list of [open issues](https://github.com/nickel-org/nickel/issues?state=open) right here on github.

If you need a helping hand reach out to [@jolhoeft](https://github.com/jolhoeft) , [@cburgdorf](https://github.com/cburgdorf) , [@Ryman](https://github.com/Ryman) or [@SimonPersson](https://github.com/SimonPersson) .

And hey, did you know you can also contribute by just starring the project here on github :)

### Development Plan

Version
Branch
Description

0.11.x
maint-0.11.x
hyper-0.10.x (synchronous version), bug fixes only

0.12.x
master
hyper-0.14.x (asynchronous version)

0.13.x

new features, possibly will be 1.0 instead

## About

An expressjs inspired web framework for Rust

[nickel-org.github.io/](http://nickel-org.github.io/)

### Resources

Readme

### License

MIT license

### Contributing

Contributing

### Uh oh!

There was an error while loading. Please reload this page.

Activity

Custom properties

### Stars

**3.1k**
stars

### Watchers

**68**
watching

### Forks

**159**
forks

Report repository

##
Releases

12
tags

##
Packages
0

### Uh oh!


### Uh oh!


##
Contributors

-

-

-

### Uh oh!


## Languages

-

Rust
100.0%

You can’t perform that action at this time.