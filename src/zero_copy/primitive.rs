use super::*;

#[derive(Copy, Clone)]
pub struct End<I: ?Sized>(PhantomData<I>);

pub fn end<I: Input + ?Sized>() -> End<I> {
    End(PhantomData)
}

impl<'a, I, E, S> Parser<'a, I, E, S> for End<I>
where
    I: Input + ?Sized,
    E: Error<I::Token>,
    S: 'a,
{
    type Output = ();

    fn go<M: Mode>(&self, inp: &mut InputRef<'a, '_, I, E, S>) -> PResult<M, Self::Output, E> {
        match inp.next() {
            (_, None) => Ok(M::bind(|| ())),
            (_, Some(_)) => Err(E::create()),
        }
    }

    go_extra!();
}

#[derive(Copy, Clone)]
pub struct Empty<I: ?Sized>(PhantomData<I>);

pub fn empty<I: Input + ?Sized>() -> Empty<I> {
    Empty(PhantomData)
}

impl<'a, I, E, S> Parser<'a, I, E, S> for Empty<I>
where
    I: Input + ?Sized,
    E: Error<I::Token>,
    S: 'a,
{
    type Output = ();

    fn go<M: Mode>(&self, inp: &mut InputRef<'a, '_, I, E, S>) -> PResult<M, Self::Output, E> {
        Ok(M::bind(|| ()))
    }

    go_extra!();
}

pub trait Seq<T> {
    type Iter<'a>: Iterator<Item = T>
    where
        Self: 'a;
    fn iter(&self) -> Self::Iter<'_>;
}

impl<T: Clone> Seq<T> for T {
    type Iter<'a>
    where
        Self: 'a,
    = core::iter::Once<T>;
    fn iter(&self) -> Self::Iter<'_> {
        core::iter::once(self.clone())
    }
}

impl<T: Clone, const N: usize> Seq<T> for [T; N] {
    type Iter<'a>
    where
        Self: 'a,
    = core::array::IntoIter<T, N>;
    fn iter(&self) -> Self::Iter<'_> {
        core::array::IntoIter::new(self.clone())
    }
}

impl<'b, T: Clone, const N: usize> Seq<T> for &'b [T; N] {
    type Iter<'a>
    where
        Self: 'a,
    = core::array::IntoIter<T, N>;
    fn iter(&self) -> Self::Iter<'_> {
        core::array::IntoIter::new((*self).clone())
    }
}

impl Seq<char> for str {
    type Iter<'a>
    where
        Self: 'a,
    = core::str::Chars<'a>;
    fn iter(&self) -> Self::Iter<'_> {
        self.chars()
    }
}

impl<'b> Seq<char> for &'b str {
    type Iter<'a>
    where
        Self: 'a,
    = core::str::Chars<'a>;
    fn iter(&self) -> Self::Iter<'_> {
        self.chars()
    }
}

impl Seq<char> for String {
    type Iter<'a>
    where
        Self: 'a,
    = core::str::Chars<'a>;
    fn iter(&self) -> Self::Iter<'_> {
        self.chars()
    }
}

// impl<'b, T, C: Container<T>> Container<T> for &'b C {
//     type Iter<'a> = C::Iter<'a>;
//     fn iter(&self) -> Self::Iter<'_> { (*self).iter() }
// }

pub struct Just<T, I: ?Sized, E = (), S = ()> {
    seq: T,
    phantom: PhantomData<(E, S, I)>,
}

impl<T: Copy, I: ?Sized, E, S> Copy for Just<T, I, E, S> {}
impl<T: Clone, I: ?Sized, E, S> Clone for Just<T, I, E, S> {
    fn clone(&self) -> Self {
        Self {
            seq: self.seq.clone(),
            phantom: PhantomData,
        }
    }
}

pub fn just<T, I, E, S>(seq: T) -> Just<T, I, E, S>
where
    I: Input + ?Sized,
    E: Error<I::Token>,
    I::Token: PartialEq,
    T: Seq<I::Token> + Clone,
{
    Just {
        seq,
        phantom: PhantomData,
    }
}

impl<'a, I, E, S, T> Parser<'a, I, E, S> for Just<T, I, E, S>
where
    I: Input + ?Sized,
    E: Error<I::Token>,
    S: 'a,
    I::Token: PartialEq,
    T: Seq<I::Token> + Clone,
{
    type Output = T;

    fn go<M: Mode>(&self, inp: &mut InputRef<'a, '_, I, E, S>) -> PResult<M, Self::Output, E> {
        let mut items = self.seq.iter();
        loop {
            match items.next() {
                Some(next) => match inp.next() {
                    (_, Some(tok)) if next == tok => {}
                    (_, Some(_) | None) => break Err(E::create()),
                },
                None => break Ok(M::bind(|| self.seq.clone())),
            }
        }
    }

    go_extra!();
}

pub struct OneOf<T, I: ?Sized, E = (), S = ()> {
    seq: T,
    phantom: PhantomData<(E, S, I)>,
}

impl<T: Copy, I: ?Sized, E, S> Copy for OneOf<T, I, E, S> {}
impl<T: Clone, I: ?Sized, E, S> Clone for OneOf<T, I, E, S> {
    fn clone(&self) -> Self {
        Self {
            seq: self.seq.clone(),
            phantom: PhantomData,
        }
    }
}

pub fn one_of<T, I, E, S>(seq: T) -> OneOf<T, I, E, S>
where
    I: Input + ?Sized,
    E: Error<I::Token>,
    I::Token: PartialEq,
    T: Seq<I::Token> + Clone,
{
    OneOf {
        seq,
        phantom: PhantomData,
    }
}

impl<'a, I, E, S, T> Parser<'a, I, E, S> for OneOf<T, I, E, S>
where
    I: Input + ?Sized,
    E: Error<I::Token>,
    S: 'a,
    I::Token: PartialEq,
    T: Seq<I::Token> + Clone,
{
    type Output = I::Token;

    fn go<M: Mode>(&self, inp: &mut InputRef<'a, '_, I, E, S>) -> PResult<M, Self::Output, E> {
        match inp.next() {
            (_, Some(tok)) if self.seq.iter().any(|not| not == tok) => Ok(M::bind(|| tok)),
            (at, found) => Err(E::create()),
        }
    }

    go_extra!();
}

pub struct NoneOf<T, I: ?Sized, E = (), S = ()> {
    seq: T,
    phantom: PhantomData<(E, S, I)>,
}

impl<T: Copy, I: ?Sized, E, S> Copy for NoneOf<T, I, E, S> {}
impl<T: Clone, I: ?Sized, E, S> Clone for NoneOf<T, I, E, S> {
    fn clone(&self) -> Self {
        Self {
            seq: self.seq.clone(),
            phantom: PhantomData,
        }
    }
}

pub fn none_of<T, I, E, S>(seq: T) -> NoneOf<T, I, E, S>
where
    I: Input + ?Sized,
    E: Error<I::Token>,
    I::Token: PartialEq,
    T: Seq<I::Token> + Clone,
{
    NoneOf {
        seq,
        phantom: PhantomData,
    }
}

impl<'a, I, E, S, T> Parser<'a, I, E, S> for NoneOf<T, I, E, S>
where
    I: Input + ?Sized,
    E: Error<I::Token>,
    S: 'a,
    I::Token: PartialEq,
    T: Seq<I::Token> + Clone,
{
    type Output = I::Token;

    fn go<M: Mode>(&self, inp: &mut InputRef<'a, '_, I, E, S>) -> PResult<M, Self::Output, E> {
        match inp.next() {
            (_, Some(tok)) if self.seq.iter().all(|not| not != tok) => Ok(M::bind(|| tok)),
            (at, found) => Err(E::create()),
        }
    }

    go_extra!();
}

pub struct Filter<F, I: ?Sized, E> {
    filter: F,
    phantom: PhantomData<(E, I)>,
}

impl<F: Copy, I: ?Sized, E> Copy for Filter<F, I, E> {}
impl<F: Clone, I: ?Sized, E> Clone for Filter<F, I, E> {
    fn clone(&self) -> Self {
        Self {
            filter: self.filter.clone(),
            phantom: PhantomData,
        }
    }
}

pub fn filter<F: Fn(&I::Token) -> bool, I: Input + ?Sized, E: Error<I::Token>>(
    filter: F,
) -> Filter<F, I, E> {
    Filter {
        filter,
        phantom: PhantomData,
    }
}

impl<'a, I, E, S, F> Parser<'a, I, E, S> for Filter<F, I, E>
where
    I: Input + ?Sized,
    E: Error<I::Token>,
    S: 'a,
    F: Fn(&I::Token) -> bool,
{
    type Output = I::Token;

    fn go<M: Mode>(&self, inp: &mut InputRef<'a, '_, I, E, S>) -> PResult<M, Self::Output, E> {
        match inp.next() {
            (_, Some(tok)) if (self.filter)(&tok) => Ok(M::bind(|| tok)),
            (_, Some(_) | None) => Err(E::create()),
        }
    }

    go_extra!();
}

pub type Any<I, E> = Filter<fn(&<I as Input>::Token) -> bool, I, E>;

pub fn any<I: Input + ?Sized, E: Error<I::Token>>() -> Any<I, E> {
    filter(|_| true)
}

#[derive(Copy, Clone)]
pub struct Choice<T, O> {
    parsers: T,
    phantom: PhantomData<O>,
}

pub fn choice<T, O>(parsers: T) -> Choice<T, O> {
    Choice {
        parsers,
        phantom: PhantomData,
    }
}

macro_rules! impl_for_tuple {
    () => {};
    ($head:ident $($X:ident)*) => {
        impl_for_tuple!($($X)*);
        impl_for_tuple!(~ $head $($X)*);
    };
    (~ $($X:ident)*) => {
        #[allow(unused_variables, non_snake_case)]
        impl<'a, I, E, S, $($X),*, O> Parser<'a, I, E, S> for Choice<($($X,)*), O>
        where
            I: Input + ?Sized,
            E: Error<I::Token>,
            S: 'a,
            $($X: Parser<'a, I, E, S, Output = O>),*
        {
            type Output = O;

            fn go<M: Mode>(&self, inp: &mut InputRef<'a, '_, I, E, S>) -> PResult<M, Self::Output, E> {
                let before = inp.save();

                let Choice { parsers: ($($X,)*), .. } = self;

                $(
                    match $X.go::<M>(inp) {
                        Ok(out) => return Ok(out),
                        Err(_) => inp.rewind(before),
                    };
                )*

                Err(E::create())
            }

            go_extra!();
        }
    };
}

impl_for_tuple!(A_ B_ C_ D_ E_ F_ G_ H_ I_ J_ K_ L_ M_ N_ O_ P_ Q_ S_ T_ U_ V_ W_ X_ Y_ Z_);