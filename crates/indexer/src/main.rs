use common::run_app;
use runner::IndexerRunner;

mod runner;
mod config;


fn main() {
    run_app(IndexerRunner::default());
}
