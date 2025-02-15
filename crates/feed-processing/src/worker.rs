use common::{AppError, Broadcaster, Context, MpSc, SharedRef, SpawnResult, Worker};

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
pub struct FeedProcessingWorker<I, O, A>
where
    A: FeedProcessor<I, O>,
{
    /// Application context containing configuration and runtime information
    context: Context,

    /// Multi-producer, single-consumer channel that receives input data
    receiver: MpSc<I>,

    /// Broadcaster that sends processed output data to downstream consumers
    sender: Broadcaster<O>,

    /// The implementation that processes input data into output data
    processor: SharedRef<A>,
}

impl<I, O, A> FeedProcessingWorker<I, O, A>
where
    O: Clone,
    I: Clone,
    A: FeedProcessor<I, O> + Clone,
{
    /// Creates a new FeedProcessingWorker with the given components
    pub fn new(context: Context, receiver: MpSc<I>, sender: Broadcaster<O>, processor: A) -> Self {
        Self {
            context,
            receiver,
            sender,
            processor: SharedRef::new(processor),
        }
    }

    /// Clones the worker with a new receiver
    pub fn clone_with_receiver(&mut self) -> Self {
        Self {
            context: self.context.clone(),
            receiver: self.receiver.clone_with_receiver(),
            sender: self.sender.clone(),
            processor: self.processor.clone(),
        }
    }

    pub fn process(&mut self, input: &I) -> Option<O> {
        self.processor.lock().process(input)
    }
}

impl<I, O, A> Worker for FeedProcessingWorker<I, O, A>
where
    I: Clone + Send + Sync + 'static,
    O: Clone + Send + Sync + std::fmt::Debug + 'static,
    A: FeedProcessor<I, O> + Clone + Send + Sync + 'static,
{
    fn spawn(&mut self) -> SpawnResult {
        let mut worker = self.clone_with_receiver();

        tokio::spawn(async move {
            let receiver = worker.receiver.receiver();
            if receiver.is_none() {
                return Err(AppError::Unrecoverable("receiver is not set".to_string()));
            }
            let mut receiver = receiver.unwrap();

            loop {
                tokio::select! {
                    input = receiver.recv() => {
                        let input = input.unwrap();
                        let output = worker.process(&input);
                        if let Some(output) = output {
                            //TODO: determine if this error should cause the application to exit
                            // if let Err(e) = worker.sender.try_send(output) {
                            //     log::error!("error sending output to broadcaster: {}", e);
                            // }
                            log::info!("output: {:?}", output);
                        }
                    }
                }
            }
        })
    }
}
