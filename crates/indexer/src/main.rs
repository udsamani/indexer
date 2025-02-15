use common::run_app;
use runner::IndexerRunner;

mod config;
mod processing;
mod runner;

fn main() {
    run_app(IndexerRunner::default());
}
