// Copyright 2018 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_use]
extern crate serde_derive;

#[derive(Serialize)] //~ ERROR: 12:10: 12:19: #[serde(flatten] can not be combined with #[serde(skip_serializing)]
struct Foo {
    #[serde(flatten, skip_serializing)]
    other: Other,
}

#[derive(Serialize)]
struct Other {
    x: u32
}

fn main() {}
