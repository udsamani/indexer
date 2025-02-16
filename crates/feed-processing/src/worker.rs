use common::{Broadcaster, Context, SpawnResult, Worker};

use crate::FeedProcessor;

/// A worker that processes feed data using a specified processor implementation
///
/// # Type Parameters
/// * `I` - The input type that will be received from the broadcaster
/// * `O` - The output type that will be sent to the producer
/// * `A` - The processor implementation type that transforms `I` into `O`
///
/// # Generic Constraints
/// * `A: FeedProcessor<I, O>` - The processor must implement the FeedProcessor trait
///   for the given input and output types
///
/// This worker acts as a bridge between:
/// 1. A broadcaster that sends input data of type `I`
/// 2. A processor that transforms the input
/// 3. A producer that broadcasts the processed output of type `O`
#[derive(Clone)]
pub struct FeedProcessingWorker<I, O, A>
where
    A: FeedProcessor<I, O>,
{
    /// Application context containing configuration and runtime information
    _context: Context,

    /// Multi-producer, single-consumer channel that receives input data
    receiver: Broadcaster<I>,

    /// Broadcaster that sends processed output data to downstream consumers
    sender: Broadcaster<O>,

    /// The implementation that processes input data into output data
    processor: A,
}

impl<I, O, A> FeedProcessingWorker<I, O, A>
where
    O: Clone,
    I: Clone,
    A: FeedProcessor<I, O> + Clone,
{
    /// Creates a new FeedProcessingWorker with the given components
    pub fn new(
        context: Context,
        receiver: Broadcaster<I>,
        sender: Broadcaster<O>,
        processor: A,
    ) -> Self {
        Self {
            _context: context,
            receiver,
            sender,
            processor,
        }
    }

    pub fn process(&mut self, input: &I) -> Option<O> {
        self.processor.process(input)
    }
}

impl<I, O, A> Worker for FeedProcessingWorker<I, O, A>
where
    I: Clone + Send + Sync + 'static,
    O: Clone + Send + Sync + std::fmt::Debug + 'static,
    A: FeedProcessor<I, O> + Clone + Send + Sync + 'static,
{
    fn spawn(&mut self) -> SpawnResult {
        let mut worker = self.clone();
        let mut app = worker._context.app.subscribe();

        tokio::spawn(async move {
            let mut receiver = worker.receiver.receiver();

            loop {
                tokio::select! {
                    _ = app.recv() => {
                        log::info!("{} received exit message", worker._context.name);
                        return Ok(format!("{} received exit message", worker._context.name));
                    }
                    input = receiver.recv() => {
                        let input = input.unwrap();
                        let output = worker.process(&input);
                        if let Some(output) = output {
                            if let Err(e) = worker.sender.try_send(output) {
                                log::error!("error sending output to broadcaster: {}", e);
                            }
                        }
                    }
                }
            }
        })
    }
}
