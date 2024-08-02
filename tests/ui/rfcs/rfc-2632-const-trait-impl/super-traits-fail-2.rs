//@ compile-flags: -Znext-solver
#![allow(incomplete_features)]
#![feature(const_trait_impl, effects)]
//@ revisions: yy yn ny nn

#[cfg_attr(any(yy, yn), const_trait)]
trait Foo {
    fn a(&self);
}

#[cfg_attr(any(yy, ny), const_trait)]
trait Bar: ~const Foo {}
//[ny,nn]~^ ERROR: `~const` can only be applied to `#[const_trait]`
//[ny,nn]~| ERROR: `~const` can only be applied to `#[const_trait]`
//[ny,nn]~| ERROR: `~const` can only be applied to `#[const_trait]`
//[yn,nn]~^^^^ ERROR: `~const` is not allowed here

const fn foo<T: Bar>(x: &T) {
    x.a();
    //[yy,yn]~^ ERROR the trait bound
    // FIXME(effects) diagnostic
}

fn main() {}
