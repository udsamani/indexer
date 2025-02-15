use tokio::sync::broadcast::{Receiver, Sender};

use crate::{AppError, AppResult};

/// A broadcaster is many to many message dispatcher.
///
/// It can be used to dispatch messagess to multiple workers.
pub struct Broadcaster<M> {
    /// To send messages to the broadcaster.
    sender: Sender<M>,

    /// Capacity of the channel.
    _capacity: usize,

    /// This receiver is never used, it is here to keep the sender alive.
    _receiver: Option<Receiver<M>>,
}

impl<M> Clone for Broadcaster<M>
where
    M: Clone,
{
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            _capacity: self._capacity,
            _receiver: None,
        }
    }
}

impl<M> Default for Broadcaster<M>
where
    M: Default + Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        let (sender, receiver) = tokio::sync::broadcast::channel(1000);
        Self {
            sender,
            _capacity: 1000,
            _receiver: Some(receiver),
        }
    }
}

impl<M> Broadcaster<M>
where
    M: Clone,
{
    pub fn new(capacity: usize) -> Self {
        let (sender, receiver) = tokio::sync::broadcast::channel(capacity);
        Self {
            sender,
            _capacity: capacity,
            _receiver: Some(receiver),
        }
    }

    pub fn sender(&self) -> Sender<M> {
        self.sender.clone()
    }

    pub fn try_send(&self, message: M) -> AppResult<()> {
        if let Err(e) = self.sender.send(message) {
            return Err(AppError::ChannelSendError(e.to_string()));
        }
        Ok(())
    }
}
