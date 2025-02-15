use common::run_app;
use runner::IndexerRunner;

mod config;
mod processing;
mod runner;
mod utils;

fn main() {
    run_app(IndexerRunner::default());
}
