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

    fn run(&self) {
        let (is_ready, _cvar) = &*self.ready;
        let name = self.name.clone();

        println!("{name}: run() getting is_ready lock");
        let mut is_ready = is_ready.lock().unwrap();
        println!("{name}: run() got is_ready lock");

        println!("{name}: run() setting is_ready to true");
        *is_ready = true;
    }
}

trait Reducer {
    type State;

    fn reducer(&self, state: &mut Self::State);
}

fn main() {
    let dut_gro = Gro::new(String::from("dut"), String::from("abc"));
    let tb_gro = Gro::new(String::from("tb"), String::from("def"));

    let tb_handle = tb_gro.spawn(Arc::clone(&dut_gro.outputter));
    let dut_handle = dut_gro.spawn(Arc::clone(&tb_gro.outputter));

    tb_gro.run();
    // dut_gro.run();

    dut_handle.join().unwrap();
    tb_handle.join().unwrap();
}
