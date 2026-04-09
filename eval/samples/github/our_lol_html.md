Skip to content

You signed in with another tab or window. Reload to refresh your session.
You signed out in another tab or window. Reload to refresh your session.
You switched accounts on another tab or window. Reload to refresh your session.

Dismiss alert

{{ message }}

[nickel-org](https://github.com/nickel-org)
/
**
[nickel.rs](https://github.com/nickel-org/nickel.rs)
**

Public

[Branches](https://github.com/nickel-org/nickel.rs/branches) [Tags](https://github.com/nickel-org/nickel.rs/tags)

## Folders and files

NameName
Last commit message

Last commit date

## Latest commit

## History

[647 Commits](https://github.com/nickel-org/nickel.rs/commits/master/)

[examples](https://github.com/nickel-org/nickel.rs/tree/master/examples)


[src](https://github.com/nickel-org/nickel.rs/tree/master/src)


[tests](https://github.com/nickel-org/nickel.rs/tree/master/tests)


[.appveyor.yml](https://github.com/nickel-org/nickel.rs/blob/master/.appveyor.yml)


[.clog.toml](https://github.com/nickel-org/nickel.rs/blob/master/.clog.toml)


[.gitattributes](https://github.com/nickel-org/nickel.rs/blob/master/.gitattributes)


[.gitignore](https://github.com/nickel-org/nickel.rs/blob/master/.gitignore)


[.travis.yml](https://github.com/nickel-org/nickel.rs/blob/master/.travis.yml)


[Cargo.toml](https://github.com/nickel-org/nickel.rs/blob/master/Cargo.toml)


[LICENSE](https://github.com/nickel-org/nickel.rs/blob/master/LICENSE)


[README.md](https://github.com/nickel-org/nickel.rs/blob/master/README.md)


[changelog.md](https://github.com/nickel-org/nickel.rs/blob/master/changelog.md)


[contributing.md](https://github.com/nickel-org/nickel.rs/blob/master/contributing.md)


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

More examples can be found [in the examples directory](https://github.com/nickel-org/nickel.rs/blob/master/examples) and the full documentation can be [found here](https://docs.rs/nickel/) .

## Contributing

[nickel.rs](http://nickel-org.github.io) is a community effort. We welcome new contributors with open arms. Please read the [contributing guide here](https://github.com/nickel-org/nickel.rs/blob/master/contributing.md) first.

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

[Activity](https://github.com/nickel-org/nickel.rs/activity)

[Custom properties](https://github.com/nickel-org/nickel.rs/custom-properties)

### Stars

[**3.1k**
stars](https://github.com/nickel-org/nickel.rs/stargazers)

### Watchers

[**68**
watching](https://github.com/nickel-org/nickel.rs/watchers)

### Forks

[**159**
forks](https://github.com/nickel-org/nickel.rs/forks)

[Report repository](https://github.com/contact/report-content?content_url=https%3A%2F%2Fgithub.com%2Fnickel-org%2Fnickel.rs&amp;amp;report=nickel-org+%28user%29)

##
[Releases](https://github.com/nickel-org/nickel.rs/releases)

[12
tags](https://github.com/nickel-org/nickel.rs/tags)

##
[Packages
0](https://github.com/orgs/nickel-org/packages?repo_name=nickel.rs)

### Uh oh!


### Uh oh!


##
[Contributors](https://github.com/nickel-org/nickel.rs/graphs/contributors)

-

-

-

### Uh oh!


## Languages

-
[Rust
100.0%](https://github.com/nickel-org/nickel.rs/search?l=rust)

You can’t perform that action at this time.