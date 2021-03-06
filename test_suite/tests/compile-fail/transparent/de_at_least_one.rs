// Copyright 2018 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_use]
extern crate serde_derive;

#[derive(Deserialize)] //~ ERROR: 12:10: 12:21: #[serde(transparent)] requires at least one field that is neither skipped nor has a default
#[serde(transparent)]
struct S {
    #[serde(skip)]
    a: u8,
    #[serde(default)]
    b: u8,
}

fn main() {}
