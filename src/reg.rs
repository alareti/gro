use crate::comms;

use std::{
    cell::{OnceCell, RefCell},
    rc::Rc,
};

trait Reducer<S> {
    fn reduce(&self, state: &mut S);
}

// Input is some type which implements Reducer on self's state
type Input<S> = Option<Box<dyn Reducer<S>>>;

// Updater takes an Input and mutates self's state
type Updater<S> = Box<dyn Fn(Input<S>)>;

// Self's outputter takes another's Updater as input
// type Outputter<S> = Box<dyn Fn(Box<dyn Fn(S)>)>;

struct Register<S> {
    state: Rc<RefCell<S>>,
    delta: Option<Input<S>>,
    data_tx: OnceCell<comms::Port<S>>,
    data_rx: Option<comms::Port<Input<S>>>,
    update_done_tx: OnceCell<comms::Port<()>>,
    update_start_rx: Option<comms::Port<()>>,
    reset_rx: Option<comms::Port<()>>,
}

impl<S: 'static> Register<S> {
    fn new(init: S) -> Self {
        let state = Rc::new(RefCell::new(init));
        let c_state = Rc::clone(&state);

        let data_on_recv: Updater<S> = Box::new(move |input| match input {
            Some(data) => {
                data.reduce(&mut RefCell::borrow_mut(&c_state));
            }
            None => {}
        });

        let data_rx = Some(comms::Port::new(data_on_recv));
        let data_tx = OnceCell::new();

        Register {
            state,
            data_tx,
            data_rx,
        }
    }
}

// struct Builder<S> {
//     init: S,
//     input: Option<Rc<Driver<Input<S>>>>,
//     output: Option<Rc<Driver<Box<S>>>>,
// }

// impl<S> Builder<S> {
//     fn new(init: S) -> Self {
//         Builder {
//             init,
//             input: None,
//             output: None,
//         }
//     }

//     // This will define an external driver's on_recv field
//     fn input(mut self, driver: Rc<Driver<Input<S>>>) -> Self {
//         self.input = Some(driver);
//         self
//     }

//     // This will come to own the driver as its own
//     // Boxed for symmetry with input
//     fn output(mut self, driver: Rc<Driver<Box<S>>>) -> Self {
//         self.output = Some(driver);
//         self
//     }
// }
