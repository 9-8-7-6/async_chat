use async_std::{task, FromServer};
use crate::connection::Outbound;
use std::sync::Arc;
use tokio::sync::broadcast::{self, error::RecvError};

pub struct Group {
    name: Arc<String>,
    Sender: broadcast::Sender<Arc<String>>
}

impl Group{
    pub fn new(name: Arc<String>) -> Group{
        let (sender, _receiver) = broadcast::channel(1000);
        Group { name, sender}
    }

    pub fn join(&self, outbound: Arc<Outbound>) {
        let receiver = self.sender.subscribe();

        task::spawn(handle_subscriber(self.name.clone(), receiver, outbound));
    }

    pub fn post(&self, message: Arc<String>) {
        let _ignored = self.sender.send(message);
    }
    
}

async fn handle_subscriber(group_name: Arc<String>, mut receiver: broadcast::Receiver<Arc<String>>, outbound: Arc<Outbound>)
{
    loop {
        let pack = match receiver.recv().await {
            Ok(message) => FromServer::Message {
                group_name: group_name.clone(),
                message: message.clone(),
            },

            Err(RecvError::Lagged(n)) => FromSerrver::Error(
                format!("Dropped {} messages from {}.", n, group_name)
            ),

            Err(RecvError::Closed) => break,
        };

        if outbound.send(packet).await.is_err() {
            break;
        }
    }
}
