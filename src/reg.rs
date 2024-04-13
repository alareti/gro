use crate::ross;

struct Builder<S> {
    init: S,
}

impl<S> Builder<S> {
    fn new(init: S) -> Self {
        Builder { init }
    }

    fn input(sender: usize) {}
}
