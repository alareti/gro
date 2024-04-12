mod gro;

// use std::thread;

// type Flag = Arc<(Mutex<bool>, Condvar)>;
// type Updater = Arc<dyn Fn(&Input) + Send + Sync>;
// type Outputter = Arc<dyn Fn(&Updater) + Send + Sync>;

// struct DutState {
//     s: String,
// }
//
// struct SomeInput {
//     s: String,
//     b: bool,
// }
//
// impl gro::Reducer for SomeInput {
//     type State = DutState;
//
//     fn reduce(&self, state: &mut Self::State) {
//         if self.b {
//             state.s += &self.s;
//         }
//     }
// }

fn main() {
    // let mut dut_gro = gro::Gro::new(DutState {
    //     s: String::from(""),
    // });

    // let i = SomeInput {
    //     s: String::from("Some input"),
    //     b: true,
    // };

    // dut_gro.drive(Box::new(i));
}
