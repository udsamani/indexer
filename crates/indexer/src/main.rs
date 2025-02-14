use common::run_app;
use runner::IndexerRunner;

mod runner;


fn main() {
    run_app(IndexerRunner::default());
}
