macro_rules! arm {
    ($pattern:pat => $block:block) => {
        $pattern => $block
        //~^ ERROR macro expansion ignores token `=>` and any following
        //~| NOTE the usage of `arm!` is likely invalid in pattern context
        //~| NOTE macros cannot expand to match arms
    };
}

fn main() {
    let x = Some(1);
    match x {
        Some(1) => {},
        arm!(None => {}),
        //~^ NOTE caused by the macro expansion here
        Some(2) => {},
        _ => {},
    };
}
