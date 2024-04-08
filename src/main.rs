use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::thread;

type State = String;
type Input = String;

// const INIT_STATE: &str = "abc";
fn reducer(input: &Input, state: &mut State) {
    *state = input.clone();
}

type Flag = Arc<(Mutex<bool>, Condvar)>;
type StateUpdater = Arc<dyn Fn(&Input) + Send + Sync>;
type StateOutputter = Arc<dyn Fn(&StateUpdater) + Send + Sync>;

struct Gro {
    name: String,
    state: Arc<RwLock<State>>,
    ready: Flag,
    outputter: StateOutputter,
    updater: StateUpdater,
}

impl Gro {
    fn new(name: String, init: State) -> Gro {
        let state = Arc::new(RwLock::new(init));

        // Interpret as 'ready to transmit'
        let ready: Flag = Arc::new((Mutex::new(false), Condvar::new()));

        let c_state = Arc::clone(&state);
        let c_ready = Arc::clone(&ready);
        let c_name = name.clone();
        let outputter: StateOutputter = Arc::new(move |updater: &StateUpdater| {
            let (is_ready, cvar) = &*c_ready;

            println!("{c_name}: Outputter getting is_ready lock");
            let mut is_ready = is_ready.lock().unwrap();
            println!("{c_name}: Outputter got is_ready lock");
            println!("{c_name}: Outputter is_ready: {}", is_ready);

            if !*is_ready {
                println!("{c_name}: Outputter waiting...");
                is_ready = cvar.wait(is_ready).unwrap();
            }

            println!("{c_name}: Outputter calling connected updater");
            updater(&c_state.read().unwrap());

            println!("{c_name}: Outputter setting ready to false");
            *is_ready = false;

            println!("{c_name}: Outputter notifying and exiting");
            cvar.notify_all();
        });

        let c_state = Arc::clone(&state);
        let c_ready = Arc::clone(&ready);
        let c_name = name.clone();
        let updater: StateUpdater = Arc::new(move |input: &Input| {
            let (is_ready, cvar) = &*c_ready;

            println!("{c_name}: Updater getting is_ready lock");
            let mut is_ready = is_ready.lock().unwrap();
            println!("{c_name}: Updater got is_ready lock");

            if *is_ready {
                println!("{c_name}: Updater waiting...");
                is_ready = cvar.wait(is_ready).unwrap();
            }

            println!("{c_name}: Updater calling reducer");
            reducer(input, &mut c_state.write().expect("Couldn't write state"));

            println!("{c_name}: Updater setting ready to true");
            *is_ready = true;

            println!("{c_name}: Updater notifying and exiting");
            cvar.notify_all();
        });

        Gro {
            name,
            state,
            ready,
            outputter,
            updater,
        }
    }

    fn spawn(&self, other_tx: StateOutputter) -> thread::JoinHandle<()> {
        let c_rx = Arc::clone(&self.updater);
        let c_state = Arc::clone(&self.state);
        let c_ready = Arc::clone(&self.ready);
        let c_name = self.name.clone();

        thread::spawn(move || {
            let mut iter: u64 = 0;
            loop {
                iter += 1;
                println!("{} iter {}: {:?}", c_name, iter, c_state);
                println!("{} iter {}: {:?}", c_name, iter, c_ready);

                other_tx(&c_rx);
            }
        })
    }

    fn reset(&self) {
        let (is_ready, _cvar) = &*self.ready;
        let name = self.name.clone();

        println!("{name}: reset() getting is_ready lock");
        let mut is_ready = is_ready.lock().unwrap();
        println!("{name}: reset() got is_ready lock");

        println!("{name}: reset() setting is_ready to true");
        *is_ready = true;
    }
}

fn main() {
    let dut_gro = Gro::new(String::from("dut"), String::from("abc"));
    let tb_gro = Gro::new(String::from("tb"), String::from("def"));

    let tb_handle = tb_gro.spawn(Arc::clone(&dut_gro.outputter));
    let dut_handle = dut_gro.spawn(Arc::clone(&tb_gro.outputter));

    tb_gro.reset();
    // dut_gro.reset();

    dut_handle.join().unwrap();
    tb_handle.join().unwrap();
}

// fn main() {
//     let groat_builder_a = Builder::new(StateA(String::from(""))).name(String::from("GroatA"));
//     let groat_builder_b = Builder::new(StateB(String::from(""))).name(String::from("GroatB"));
//
//     let groat_a = groat_builder_a.spawn();
//     let groat_b = groat_builder_b.spawn();
//
//     let _ = groat_a.join();
//     let _ = groat_b.join();
// }

// struct Groat<S> {
//     handle: thread::JoinHandle<()>,
//     tx: Box<dyn FnOnce() -> S>,
// }
//
// // Connecting one groat to another should be
// // as simple as something like
// // groat_a.on_tx(|&state_a| groat_b.drive(&state_a))
// // Because of that, Groat needs to know the various
// // reducer trait objects it will need to support.
// // Or at least the builder should know.
// impl<S> Groat<S> {
//     // reset essentially puts the groat in a
//     // known good state (initial_state) and
//     // causes it to transmit its state,
//     // essentially starting the actual state
//     // reducing processes once everything has
//     // been hooked up properly
//     fn reset(self) {}
//
//     fn drive(self, _input: Box<dyn Reducer<State = S>>) {}
//
//     // Fn should have Groat state type as input
//     // on_tx will call proc after every state
//     // update
//     fn on_tx(self, _proc: Box<dyn Fn()>) {}
//
//     fn join(self) -> thread::Result<()> {
//         self.handle.join()
//     }
// }
//
// struct Builder<S> {
//     initial_state: S,
//     name: Option<String>,
// }
//
// impl<S> Builder<S>
// where
//     S: Send + 'static + std::fmt::Debug,
// {
//     fn new(initial_state: S) -> Self {
//         Builder {
//             initial_state,
//             name: None,
//         }
//     }
//
//     fn name(self, name: String) -> Self {
//         Builder {
//             initial_state: self.initial_state,
//             name: Some(name),
//         }
//     }
//
//     fn spawn(self) -> Groat<S> {
//         let thread_builder = thread::Builder::new();
//
//         let thread_builder = match self.name {
//             Some(name) => thread_builder.name(name),
//             None => thread_builder,
//         };
//
//         let handle = thread_builder
//             .spawn(move || {
//                 let groat_name = match thread::current().name() {
//                     Some(name) => String::from(name),
//                     None => format!("{:?}", thread::current().id()),
//                 };
//                 println!("Spawned new groat named {groat_name}");
//
//                 // let state = self.initial_state;
//                 // println!("{groat_name}\tInitial state: {state:#?}");
//             })
//             .unwrap();
//
//         Groat {
//             handle,
//             tx: Box::new(|| self.initial_state),
//         }
//     }
// }
//
// trait Reducer {
//     type State;
//
//     fn reducer(&self, state: &mut Self::State);
// }
//
// #[derive(Debug)]
// struct StateA(String);
// impl Reducer for StateB {
//     type State = StateA;
//
//     fn reducer(&self, state: &mut Self::State) {
//         state.0.push_str(&self.0.to_string());
//     }
// }
//
// #[derive(Debug)]
// struct StateB(String);
// impl Reducer for StateA {
//     type State = StateB;
//
//     fn reducer(&self, state: &mut Self::State) {
//         state.0.push_str(&self.0.to_string());
//     }
// }
