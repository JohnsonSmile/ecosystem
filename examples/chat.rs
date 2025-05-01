use anyhow::{anyhow, Result};
use dashmap::DashMap;
use futures::stream::SplitStream;
use futures::{SinkExt, StreamExt};
use std::fmt::Display;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_util::codec::{Framed, LinesCodec};
use tracing::level_filters::LevelFilter;
use tracing::{info, warn};
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

#[derive(Debug, Default)]
struct State {
    peers: DashMap<SocketAddr, mpsc::Sender<Arc<Message>>>,
}

const MAX_MESSAGE_SIZE: usize = 1024;

#[derive(Debug)]
enum SystemMessage {
    UserJoin(SocketAddr, String),
    UserLeave(SocketAddr, String),
}
impl State {
    async fn broadcast(&self, addr: SocketAddr, message: Arc<Message>) {
        for peer in self.peers.iter() {
            // if peer.key() == &addr {
            //     continue;
            // }
            if let Err(e) = peer.value().send(message.clone()).await {
                warn!("Failed to broadcast message to {} : {}", peer.key(), e);
            }
        }
    }

    async fn add(
        &self,
        addr: SocketAddr,
        mut stream: Framed<TcpStream, LinesCodec>,
    ) -> Result<Peer> {
        let (tx, mut rx) = mpsc::channel(MAX_MESSAGE_SIZE);
        self.peers.insert(addr, tx);
        let username = match stream.next().await {
            Some(Ok(line)) => line,
            Some(Err(e)) => return Err(e.into()),
            None => return Err(anyhow!("No message received")),
        };

        // receive message from others, and send them to the client
        let (mut stream_sender, stream_receiver) = stream.split();
        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if let Err(e) = stream_sender.send(message.to_string()).await {
                    warn!("Failed to send message to peer {}: {}", addr, e);
                }
            }
        });
        Ok(Peer {
            username,
            stream: stream_receiver,
        })
    }

    async fn broadcast_system(&self, system_message: SystemMessage) {
        match system_message {
            SystemMessage::UserJoin(addr, username) => {
                let message = Arc::new(Message {
                    sender: "Server".to_string(),
                    content: format!("Hello, {}!", username),
                });
                // state 广播数据
                self.broadcast(addr, message.clone()).await;
            }
            SystemMessage::UserLeave(addr, username) => {
                let message = Arc::new(Message {
                    sender: "Server".to_string(),
                    content: format!("{} has left the chat!", username),
                });
                self.peers.remove(&addr);
                self.broadcast(addr, message).await;
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Message {
    sender: String,
    content: String,
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.sender, self.content)
    }
}

struct Peer {
    username: String,
    // stream
    stream: SplitStream<Framed<TcpStream, LinesCodec>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let layer = fmt::Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = TcpListener::bind(addr).await?;
    info!("Listening on: {}", addr);

    let state = Arc::new(State::default());

    loop {
        let (stream, addr) = listener.accept().await?;
        info!("Got connection from: {}", addr);
        let state = state.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_client(state, addr, stream).await {
                warn!("Client error: {}", e);
            }
        });
    }

    #[allow(unreachable_code)]
    Ok(())
}

async fn handle_client(state: Arc<State>, addr: SocketAddr, stream: TcpStream) -> Result<()> {
    let mut stream = Framed::new(stream, LinesCodec::new());
    stream.send("Enter your username:").await?;

    if let Ok(mut peer) = state.add(addr, stream).await {
        state
            .broadcast_system(SystemMessage::UserJoin(addr, peer.username.clone()))
            .await;

        // 客户端收取消息
        while let Some(line) = peer.stream.next().await {
            let line = match line {
                Ok(line) => line,
                Err(e) => {
                    warn!("failed to read line from {}: {}", addr, e);
                    break;
                }
            };
            let message = Arc::new(Message {
                sender: peer.username.clone(),
                content: line.clone(),
            });
            state.broadcast(addr, message.clone()).await;
        }
        state
            .broadcast_system(SystemMessage::UserLeave(addr, peer.username))
            .await;
    } else {
        warn!("failed to read line from {}", addr)
    }
    Ok(())
}
