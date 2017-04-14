// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Building blocks for deserializing basic values using the `IntoDeserializer`
//! trait.
//!
//! ```rust
//! #[macro_use]
//! extern crate serde_derive;
//!
//! extern crate serde;
//!
//! use std::str::FromStr;
//! use serde::de::{value, Deserialize, IntoDeserializer};
//!
//! #[derive(Deserialize)]
//! enum Setting {
//!     On,
//!     Off,
//! }
//!
//! impl FromStr for Setting {
//!     type Err = value::Error;
//!
//!     fn from_str(s: &str) -> Result<Self, Self::Err> {
//!         Self::deserialize(s.into_deserializer())
//!     }
//! }
//! #
//! # fn main() {}
//! ```

use lib::*;

use de::{self, IntoDeserializer, Expected, SeqAccess};
use private::de::size_hint;
use self::private::{First, Second};

////////////////////////////////////////////////////////////////////////////////

/// A minimal representation of all possible errors that can occur using the
/// `IntoDeserializer` trait.
#[derive(Clone, Debug, PartialEq)]
pub struct Error {
    err: ErrorImpl,
}

#[cfg(any(feature = "std", feature = "collections"))]
type ErrorImpl = Box<str>;
#[cfg(not(any(feature = "std", feature = "collections")))]
type ErrorImpl = ();

impl de::Error for Error {
    #[cfg(any(feature = "std", feature = "collections"))]
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error { err: msg.to_string().into_boxed_str() }
    }

    #[cfg(not(any(feature = "std", feature = "collections")))]
    fn custom<T>(_msg: T) -> Self
    where
        T: Display,
    {
        Error { err: () }
    }
}

impl Display for Error {
    #[cfg(any(feature = "std", feature = "collections"))]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        formatter.write_str(&self.err)
    }

    #[cfg(not(any(feature = "std", feature = "collections")))]
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        formatter.write_str("Serde deserialization error")
    }
}

#[cfg(feature = "std")]
impl error::Error for Error {
    fn description(&self) -> &str {
        &self.err
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<'de, E> IntoDeserializer<'de, E> for ()
where
    E: de::Error,
{
    type Deserializer = UnitDeserializer<E>;

    fn into_deserializer(self) -> UnitDeserializer<E> {
        UnitDeserializer { marker: PhantomData }
    }
}

/// A helper deserializer that deserializes a `()`.
#[derive(Clone, Debug)]
pub struct UnitDeserializer<E> {
    marker: PhantomData<E>,
}

impl<'de, E> de::Deserializer<'de> for UnitDeserializer<E>
where
    E: de::Error,
{
    type Error = E;

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit seq
        seq_fixed_size bytes map unit_struct newtype_struct tuple_struct struct
        identifier tuple enum ignored_any byte_buf
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_none()
    }
}

////////////////////////////////////////////////////////////////////////////////

macro_rules! primitive_deserializer {
    ($ty:ty, $name:ident, $method:ident $($cast:tt)*) => {
        /// A helper deserializer that deserializes a number.
        #[derive(Clone, Debug)]
        pub struct $name<E> {
            value: $ty,
            marker: PhantomData<E>
        }

        impl<'de, E> IntoDeserializer<'de, E> for $ty
        where
            E: de::Error,
        {
            type Deserializer = $name<E>;

            fn into_deserializer(self) -> $name<E> {
                $name {
                    value: self,
                    marker: PhantomData,
                }
            }
        }

        impl<'de, E> de::Deserializer<'de> for $name<E>
        where
            E: de::Error,
        {
            type Error = E;

            forward_to_deserialize_any! {
                bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit
                option seq seq_fixed_size bytes map unit_struct newtype_struct
                tuple_struct struct identifier tuple enum ignored_any byte_buf
            }

            fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where
                V: de::Visitor<'de>,
            {
                visitor.$method(self.value $($cast)*)
            }
        }
    }
}

primitive_deserializer!(bool, BoolDeserializer, visit_bool);
primitive_deserializer!(i8, I8Deserializer, visit_i8);
primitive_deserializer!(i16, I16Deserializer, visit_i16);
primitive_deserializer!(i32, I32Deserializer, visit_i32);
primitive_deserializer!(i64, I64Deserializer, visit_i64);
primitive_deserializer!(isize, IsizeDeserializer, visit_i64 as i64);
primitive_deserializer!(u8, U8Deserializer, visit_u8);
primitive_deserializer!(u16, U16Deserializer, visit_u16);
primitive_deserializer!(u64, U64Deserializer, visit_u64);
primitive_deserializer!(usize, UsizeDeserializer, visit_u64 as u64);
primitive_deserializer!(f32, F32Deserializer, visit_f32);
primitive_deserializer!(f64, F64Deserializer, visit_f64);
primitive_deserializer!(char, CharDeserializer, visit_char);

/// A helper deserializer that deserializes a number.
#[derive(Clone, Debug)]
pub struct U32Deserializer<E> {
    value: u32,
    marker: PhantomData<E>,
}

impl<'de, E> IntoDeserializer<'de, E> for u32
where
    E: de::Error,
{
    type Deserializer = U32Deserializer<E>;

    fn into_deserializer(self) -> U32Deserializer<E> {
        U32Deserializer {
            value: self,
            marker: PhantomData,
        }
    }
}

impl<'de, E> de::Deserializer<'de> for U32Deserializer<E>
where
    E: de::Error,
{
    type Error = E;

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
        seq seq_fixed_size bytes map unit_struct newtype_struct tuple_struct
        struct identifier tuple ignored_any byte_buf
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u32(self.value)
    }

    fn deserialize_enum<V>(
        self,
        _name: &str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_enum(self)
    }
}

impl<'de, E> de::EnumAccess<'de> for U32Deserializer<E>
where
    E: de::Error,
{
    type Error = E;
    type Variant = private::UnitOnly<E>;

    fn variant_seed<T>(self, seed: T) -> Result<(T::Value, Self::Variant), Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self).map(private::unit_only)
    }
}

////////////////////////////////////////////////////////////////////////////////

/// A helper deserializer that deserializes a `&str`.
#[derive(Clone, Debug)]
pub struct StrDeserializer<'a, E> {
    value: &'a str,
    marker: PhantomData<E>,
}

impl<'de, 'a, E> IntoDeserializer<'de, E> for &'a str
where
    E: de::Error,
{
    type Deserializer = StrDeserializer<'a, E>;

    fn into_deserializer(self) -> StrDeserializer<'a, E> {
        StrDeserializer {
            value: self,
            marker: PhantomData,
        }
    }
}

impl<'de, 'a, E> de::Deserializer<'de> for StrDeserializer<'a, E>
where
    E: de::Error,
{
    type Error = E;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_str(self.value)
    }

    fn deserialize_enum<V>(
        self,
        _name: &str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_enum(self)
    }

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
        seq seq_fixed_size bytes map unit_struct newtype_struct tuple_struct
        struct identifier tuple ignored_any byte_buf
    }
}

impl<'de, 'a, E> de::EnumAccess<'de> for StrDeserializer<'a, E>
where
    E: de::Error,
{
    type Error = E;
    type Variant = private::UnitOnly<E>;

    fn variant_seed<T>(self, seed: T) -> Result<(T::Value, Self::Variant), Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self).map(private::unit_only)
    }
}

////////////////////////////////////////////////////////////////////////////////

/// A helper deserializer that deserializes a `String`.
#[cfg(any(feature = "std", feature = "collections"))]
#[derive(Clone, Debug)]
pub struct StringDeserializer<E> {
    value: String,
    marker: PhantomData<E>,
}

#[cfg(any(feature = "std", feature = "collections"))]
impl<'de, E> IntoDeserializer<'de, E> for String
where
    E: de::Error,
{
    type Deserializer = StringDeserializer<E>;

    fn into_deserializer(self) -> StringDeserializer<E> {
        StringDeserializer {
            value: self,
            marker: PhantomData,
        }
    }
}

#[cfg(any(feature = "std", feature = "collections"))]
impl<'de, E> de::Deserializer<'de> for StringDeserializer<E>
where
    E: de::Error,
{
    type Error = E;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_string(self.value)
    }

    fn deserialize_enum<V>(
        self,
        _name: &str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_enum(self)
    }

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
        seq seq_fixed_size bytes map unit_struct newtype_struct tuple_struct
        struct identifier tuple ignored_any byte_buf
    }
}

#[cfg(any(feature = "std", feature = "collections"))]
impl<'de, 'a, E> de::EnumAccess<'de> for StringDeserializer<E>
where
    E: de::Error,
{
    type Error = E;
    type Variant = private::UnitOnly<E>;

    fn variant_seed<T>(self, seed: T) -> Result<(T::Value, Self::Variant), Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self).map(private::unit_only)
    }
}

////////////////////////////////////////////////////////////////////////////////

/// A helper deserializer that deserializes a `String`.
#[cfg(any(feature = "std", feature = "collections"))]
#[derive(Clone, Debug)]
pub struct CowStrDeserializer<'a, E> {
    value: Cow<'a, str>,
    marker: PhantomData<E>,
}

#[cfg(any(feature = "std", feature = "collections"))]
impl<'de, 'a, E> IntoDeserializer<'de, E> for Cow<'a, str>
where
    E: de::Error,
{
    type Deserializer = CowStrDeserializer<'a, E>;

    fn into_deserializer(self) -> CowStrDeserializer<'a, E> {
        CowStrDeserializer {
            value: self,
            marker: PhantomData,
        }
    }
}

#[cfg(any(feature = "std", feature = "collections"))]
impl<'de, 'a, E> de::Deserializer<'de> for CowStrDeserializer<'a, E>
where
    E: de::Error,
{
    type Error = E;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Cow::Borrowed(string) => visitor.visit_str(string),
            Cow::Owned(string) => visitor.visit_string(string),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_enum(self)
    }

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
        seq seq_fixed_size bytes map unit_struct newtype_struct tuple_struct
        struct identifier tuple ignored_any byte_buf
    }
}

#[cfg(any(feature = "std", feature = "collections"))]
impl<'de, 'a, E> de::EnumAccess<'de> for CowStrDeserializer<'a, E>
where
    E: de::Error,
{
    type Error = E;
    type Variant = private::UnitOnly<E>;

    fn variant_seed<T>(self, seed: T) -> Result<(T::Value, Self::Variant), Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self).map(private::unit_only)
    }
}

////////////////////////////////////////////////////////////////////////////////

/// A helper deserializer that deserializes a sequence.
#[derive(Clone, Debug)]
pub struct SeqDeserializer<I, E> {
    iter: iter::Fuse<I>,
    count: usize,
    marker: PhantomData<E>,
}

impl<I, E> SeqDeserializer<I, E>
where
    I: Iterator,
    E: de::Error,
{
    /// Construct a new `SeqDeserializer<I>`.
    pub fn new(iter: I) -> Self {
        SeqDeserializer {
            iter: iter.fuse(),
            count: 0,
            marker: PhantomData,
        }
    }

    /// Check for remaining elements after passing a `SeqDeserializer` to
    /// `Visitor::visit_seq`.
    pub fn end(mut self) -> Result<(), E> {
        let mut remaining = 0;
        while self.iter.next().is_some() {
            remaining += 1;
        }
        if remaining == 0 {
            Ok(())
        } else {
            // First argument is the number of elements in the data, second
            // argument is the number of elements expected by the Deserialize.
            Err(de::Error::invalid_length(self.count + remaining, &ExpectedInSeq(self.count)),)
        }
    }
}

impl<'de, I, T, E> de::Deserializer<'de> for SeqDeserializer<I, E>
where
    I: Iterator<Item = T>,
    T: IntoDeserializer<'de, E>,
    E: de::Error,
{
    type Error = E;

    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let v = try!(visitor.visit_seq(&mut self));
        try!(self.end());
        Ok(v)
    }

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
        seq seq_fixed_size bytes map unit_struct newtype_struct tuple_struct
        struct identifier tuple enum ignored_any byte_buf
    }
}

impl<'de, I, T, E> de::SeqAccess<'de> for SeqDeserializer<I, E>
where
    I: Iterator<Item = T>,
    T: IntoDeserializer<'de, E>,
    E: de::Error,
{
    type Error = E;

    fn next_element_seed<V>(&mut self, seed: V) -> Result<Option<V::Value>, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(value) => {
                self.count += 1;
                seed.deserialize(value.into_deserializer()).map(Some)
            }
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        size_hint::from_bounds(&self.iter)
    }
}

struct ExpectedInSeq(usize);

impl Expected for ExpectedInSeq {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.0 == 1 {
            write!(formatter, "1 element in sequence")
        } else {
            write!(formatter, "{} elements in sequence", self.0)
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(any(feature = "std", feature = "collections"))]
impl<'de, T, E> IntoDeserializer<'de, E> for Vec<T>
where
    T: IntoDeserializer<'de, E>,
    E: de::Error,
{
    type Deserializer = SeqDeserializer<<Vec<T> as IntoIterator>::IntoIter, E>;

    fn into_deserializer(self) -> Self::Deserializer {
        SeqDeserializer::new(self.into_iter())
    }
}

#[cfg(any(feature = "std", feature = "collections"))]
impl<'de, T, E> IntoDeserializer<'de, E> for BTreeSet<T>
where
    T: IntoDeserializer<'de, E> + Eq + Ord,
    E: de::Error,
{
    type Deserializer = SeqDeserializer<<BTreeSet<T> as IntoIterator>::IntoIter, E>;

    fn into_deserializer(self) -> Self::Deserializer {
        SeqDeserializer::new(self.into_iter())
    }
}

#[cfg(feature = "std")]
impl<'de, T, E> IntoDeserializer<'de, E> for HashSet<T>
where
    T: IntoDeserializer<'de, E> + Eq + Hash,
    E: de::Error,
{
    type Deserializer = SeqDeserializer<<HashSet<T> as IntoIterator>::IntoIter, E>;

    fn into_deserializer(self) -> Self::Deserializer {
        SeqDeserializer::new(self.into_iter())
    }
}

////////////////////////////////////////////////////////////////////////////////

/// A deserializer holding a `SeqAccess`.
#[derive(Clone, Debug)]
pub struct SeqAccessDeserializer<A> {
    seq: A,
}

impl<A> SeqAccessDeserializer<A> {
    /// Construct a new `SeqAccessDeserializer<A>`.
    pub fn new(seq: A) -> Self {
        SeqAccessDeserializer { seq: seq }
    }
}

impl<'de, A> de::Deserializer<'de> for SeqAccessDeserializer<A>
where
    A: de::SeqAccess<'de>,
{
    type Error = A::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_seq(self.seq)
    }

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
        seq seq_fixed_size bytes map unit_struct newtype_struct tuple_struct
        struct identifier tuple enum ignored_any byte_buf
    }
}

////////////////////////////////////////////////////////////////////////////////

/// A helper deserializer that deserializes a map.
pub struct MapDeserializer<'de, I, E>
where
    I: Iterator,
    I::Item: private::Pair,
    First<I::Item>: IntoDeserializer<'de, E>,
    Second<I::Item>: IntoDeserializer<'de, E>,
    E: de::Error,
{
    iter: iter::Fuse<I>,
    value: Option<Second<I::Item>>,
    count: usize,
    lifetime: PhantomData<&'de ()>,
    error: PhantomData<E>,
}

impl<'de, I, E> MapDeserializer<'de, I, E>
where
    I: Iterator,
    I::Item: private::Pair,
    First<I::Item>: IntoDeserializer<'de, E>,
    Second<I::Item>: IntoDeserializer<'de, E>,
    E: de::Error,
{
    /// Construct a new `MapDeserializer<I, K, V, E>`.
    pub fn new(iter: I) -> Self {
        MapDeserializer {
            iter: iter.fuse(),
            value: None,
            count: 0,
            lifetime: PhantomData,
            error: PhantomData,
        }
    }

    /// Check for remaining elements after passing a `MapDeserializer` to
    /// `Visitor::visit_map`.
    pub fn end(mut self) -> Result<(), E> {
        let mut remaining = 0;
        while self.iter.next().is_some() {
            remaining += 1;
        }
        if remaining == 0 {
            Ok(())
        } else {
            // First argument is the number of elements in the data, second
            // argument is the number of elements expected by the Deserialize.
            Err(de::Error::invalid_length(self.count + remaining, &ExpectedInMap(self.count)),)
        }
    }

    fn next_pair(&mut self) -> Option<(First<I::Item>, Second<I::Item>)> {
        match self.iter.next() {
            Some(kv) => {
                self.count += 1;
                Some(private::Pair::split(kv))
            }
            None => None,
        }
    }
}

impl<'de, I, E> de::Deserializer<'de> for MapDeserializer<'de, I, E>
where
    I: Iterator,
    I::Item: private::Pair,
    First<I::Item>: IntoDeserializer<'de, E>,
    Second<I::Item>: IntoDeserializer<'de, E>,
    E: de::Error,
{
    type Error = E;

    fn deserialize_any<V_>(mut self, visitor: V_) -> Result<V_::Value, Self::Error>
    where
        V_: de::Visitor<'de>,
    {
        let value = try!(visitor.visit_map(&mut self));
        try!(self.end());
        Ok(value)
    }

    fn deserialize_seq<V_>(mut self, visitor: V_) -> Result<V_::Value, Self::Error>
    where
        V_: de::Visitor<'de>,
    {
        let value = try!(visitor.visit_seq(&mut self));
        try!(self.end());
        Ok(value)
    }

    fn deserialize_seq_fixed_size<V_>(self,
                                      _len: usize,
                                      visitor: V_)
                                      -> Result<V_::Value, Self::Error>
    where
        V_: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
        bytes map unit_struct newtype_struct tuple_struct struct identifier
        tuple enum ignored_any byte_buf
    }
}

impl<'de, I, E> de::MapAccess<'de> for MapDeserializer<'de, I, E>
where
    I: Iterator,
    I::Item: private::Pair,
    First<I::Item>: IntoDeserializer<'de, E>,
    Second<I::Item>: IntoDeserializer<'de, E>,
    E: de::Error,
{
    type Error = E;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.next_pair() {
            Some((key, value)) => {
                self.value = Some(value);
                seed.deserialize(key.into_deserializer()).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        let value = self.value.take();
        // Panic because this indicates a bug in the program rather than an
        // expected failure.
        let value = value.expect("MapAccess::visit_value called before visit_key");
        seed.deserialize(value.into_deserializer())
    }

    fn next_entry_seed<TK, TV>(&mut self,
                          kseed: TK,
                          vseed: TV)
                          -> Result<Option<(TK::Value, TV::Value)>, Self::Error>
    where
        TK: de::DeserializeSeed<'de>,
        TV: de::DeserializeSeed<'de>,
    {
        match self.next_pair() {
            Some((key, value)) => {
                let key = try!(kseed.deserialize(key.into_deserializer()));
                let value = try!(vseed.deserialize(value.into_deserializer()));
                Ok(Some((key, value)))
            }
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        size_hint::from_bounds(&self.iter)
    }
}

impl<'de, I, E> de::SeqAccess<'de> for MapDeserializer<'de, I, E>
where
    I: Iterator,
    I::Item: private::Pair,
    First<I::Item>: IntoDeserializer<'de, E>,
    Second<I::Item>: IntoDeserializer<'de, E>,
    E: de::Error,
{
    type Error = E;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.next_pair() {
            Some((k, v)) => {
                let de = PairDeserializer(k, v, PhantomData);
                seed.deserialize(de).map(Some)
            }
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        size_hint::from_bounds(&self.iter)
    }
}

// Cannot #[derive(Clone)] because of the bound:
//
//    <I::Item as private::Pair>::Second: Clone
impl<'de, I, E> Clone for MapDeserializer<'de, I, E>
where
    I: Iterator + Clone,
    I::Item: private::Pair,
    First<I::Item>: IntoDeserializer<'de, E>,
    Second<I::Item>: IntoDeserializer<'de, E> + Clone,
    E: de::Error,
{
    fn clone(&self) -> Self {
        MapDeserializer {
            iter: self.iter.clone(),
            value: self.value.clone(),
            count: self.count,
            lifetime: self.lifetime,
            error: self.error,
        }
    }
}

// Cannot #[derive(Debug)] because of the bound:
//
//    <I::Item as private::Pair>::Second: Debug
impl<'de, I, E> Debug for MapDeserializer<'de, I, E>
where
    I: Iterator + Debug,
    I::Item: private::Pair,
    First<I::Item>: IntoDeserializer<'de, E>,
    Second<I::Item>: IntoDeserializer<'de, E> + Debug,
    E: de::Error,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.debug_struct("MapDeserializer")
            .field("iter", &self.iter)
            .field("value", &self.value)
            .field("count", &self.count)
            .field("lifetime", &self.lifetime)
            .field("error", &self.error)
            .finish()
    }
}

// Used in the `impl SeqAccess for MapDeserializer` to visit the map as a
// sequence of pairs.
struct PairDeserializer<A, B, E>(A, B, PhantomData<E>);

impl<'de, A, B, E> de::Deserializer<'de> for PairDeserializer<A, B, E>
where
    A: IntoDeserializer<'de, E>,
    B: IntoDeserializer<'de, E>,
    E: de::Error,
{
    type Error = E;

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
        bytes map unit_struct newtype_struct tuple_struct struct identifier
        tuple enum ignored_any byte_buf
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let mut pair_visitor = PairVisitor(Some(self.0), Some(self.1), PhantomData);
        let pair = try!(visitor.visit_seq(&mut pair_visitor));
        if pair_visitor.1.is_none() {
            Ok(pair)
        } else {
            let remaining = pair_visitor.size_hint().unwrap();
            // First argument is the number of elements in the data, second
            // argument is the number of elements expected by the Deserialize.
            Err(de::Error::invalid_length(2, &ExpectedInSeq(2 - remaining)))
        }
    }

    fn deserialize_seq_fixed_size<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if len == 2 {
            self.deserialize_seq(visitor)
        } else {
            // First argument is the number of elements in the data, second
            // argument is the number of elements expected by the Deserialize.
            Err(de::Error::invalid_length(2, &ExpectedInSeq(len)))
        }
    }
}

struct PairVisitor<A, B, E>(Option<A>, Option<B>, PhantomData<E>);

impl<'de, A, B, E> de::SeqAccess<'de> for PairVisitor<A, B, E>
where
    A: IntoDeserializer<'de, E>,
    B: IntoDeserializer<'de, E>,
    E: de::Error,
{
    type Error = E;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        if let Some(k) = self.0.take() {
            seed.deserialize(k.into_deserializer()).map(Some)
        } else if let Some(v) = self.1.take() {
            seed.deserialize(v.into_deserializer()).map(Some)
        } else {
            Ok(None)
        }
    }

    fn size_hint(&self) -> Option<usize> {
        if self.0.is_some() {
            Some(2)
        } else if self.1.is_some() {
            Some(1)
        } else {
            Some(0)
        }
    }
}

struct ExpectedInMap(usize);

impl Expected for ExpectedInMap {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.0 == 1 {
            write!(formatter, "1 element in map")
        } else {
            write!(formatter, "{} elements in map", self.0)
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(any(feature = "std", feature = "collections"))]
impl<'de, K, V, E> IntoDeserializer<'de, E> for BTreeMap<K, V>
where
    K: IntoDeserializer<'de, E> + Eq + Ord,
    V: IntoDeserializer<'de, E>,
    E: de::Error,
{
    type Deserializer = MapDeserializer<'de, <BTreeMap<K, V> as IntoIterator>::IntoIter, E>;

    fn into_deserializer(self) -> Self::Deserializer {
        MapDeserializer::new(self.into_iter())
    }
}

#[cfg(feature = "std")]
impl<'de, K, V, E> IntoDeserializer<'de, E> for HashMap<K, V>
where
    K: IntoDeserializer<'de, E> + Eq + Hash,
    V: IntoDeserializer<'de, E>,
    E: de::Error,
{
    type Deserializer = MapDeserializer<'de, <HashMap<K, V> as IntoIterator>::IntoIter, E>;

    fn into_deserializer(self) -> Self::Deserializer {
        MapDeserializer::new(self.into_iter())
    }
}

////////////////////////////////////////////////////////////////////////////////

/// A deserializer holding a `MapAccess`.
#[derive(Clone, Debug)]
pub struct MapAccessDeserializer<A> {
    map: A,
}

impl<A> MapAccessDeserializer<A> {
    /// Construct a new `MapAccessDeserializer<A>`.
    pub fn new(map: A) -> Self {
        MapAccessDeserializer { map: map }
    }
}

impl<'de, A> de::Deserializer<'de> for MapAccessDeserializer<A>
where
    A: de::MapAccess<'de>,
{
    type Error = A::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(self.map)
    }

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
        seq seq_fixed_size bytes map unit_struct newtype_struct tuple_struct
        struct identifier tuple enum ignored_any byte_buf
    }
}

////////////////////////////////////////////////////////////////////////////////

mod private {
    use lib::*;

    use de::{self, Unexpected};

    #[derive(Clone, Debug)]
    pub struct UnitOnly<E> {
        marker: PhantomData<E>,
    }

    pub fn unit_only<T, E>(t: T) -> (T, UnitOnly<E>) {
        (t, UnitOnly { marker: PhantomData })
    }

    impl<'de, E> de::VariantAccess<'de> for UnitOnly<E>
    where
        E: de::Error,
    {
        type Error = E;

        fn deserialize_unit(self) -> Result<(), Self::Error> {
            Ok(())
        }

        fn deserialize_newtype_seed<T>(self, _seed: T) -> Result<T::Value, Self::Error>
        where
            T: de::DeserializeSeed<'de>,
        {
            Err(de::Error::invalid_type(Unexpected::UnitVariant, &"newtype variant"),)
        }

        fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
        {
            Err(de::Error::invalid_type(Unexpected::UnitVariant, &"tuple variant"),)
        }

        fn deserialize_struct<V>(
            self,
            _fields: &'static [&'static str],
            _visitor: V,
        ) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
        {
            Err(de::Error::invalid_type(Unexpected::UnitVariant, &"struct variant"),)
        }
    }

    /// Avoid having to restate the generic types on `MapDeserializer`. The
    /// `Iterator::Item` contains enough information to figure out K and V.
    pub trait Pair {
        type First;
        type Second;
        fn split(self) -> (Self::First, Self::Second);
    }

    impl<A, B> Pair for (A, B) {
        type First = A;
        type Second = B;
        fn split(self) -> (A, B) {
            self
        }
    }

    pub type First<T> = <T as Pair>::First;
    pub type Second<T> = <T as Pair>::Second;
}
