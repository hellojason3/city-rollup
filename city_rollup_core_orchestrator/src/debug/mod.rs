use city_common::cli::args::L2WorkerArgs;
use city_common::cli::args::OrchestratorArgs;
pub mod pubsub1;
pub mod scenario;
pub fn run_debug_demo(args: OrchestratorArgs) {
    pubsub1::run_pub_sub_demo_1(args)
}

pub fn run_debug_demo_client(args: L2WorkerArgs) {
    pubsub1::run_pub_sub_demo_1_client(args)
}
