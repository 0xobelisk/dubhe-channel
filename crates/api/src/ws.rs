//! WebSocket 服务器
//! 
//! 事件推送服务，利用 tokio-broadcast 多订阅者模型

use anyhow::Result;
use futures::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, RwLock};
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::types::WsEvent;

/// WebSocket 服务器
pub struct WsServer {
    connections: Arc<RwLock<HashMap<Uuid, broadcast::Sender<String>>>>,
    event_sender: broadcast::Sender<WsEvent>,
}

impl WsServer {
    pub fn new() -> Self {
        let (event_sender, _) = broadcast::channel(1000);
        
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
        }
    }

    pub async fn start(&self, bind_addr: &str) -> Result<()> {
        let listener = TcpListener::bind(bind_addr).await?;
        info!("WebSocket server listening on {}", bind_addr);

        // 启动事件广播任务
        self.start_event_broadcaster().await;

        // 处理连接
        while let Ok((stream, addr)) = listener.accept().await {
            info!("New WebSocket connection from {}", addr);
            
            let connections = self.connections.clone();
            let event_receiver = self.event_sender.subscribe();
            
            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(stream, connections, event_receiver).await {
                    error!("WebSocket connection error: {}", e);
                }
            });
        }

        Ok(())
    }

    async fn start_event_broadcaster(&self) {
        let event_sender = self.event_sender.clone();
        
        // 模拟事件发送（实际应该从其他模块接收事件）
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
            
            loop {
                interval.tick().await;
                
                // 发送示例事件
                let event = WsEvent::ParallelStats {
                    efficiency: 0.95,
                    conflicts: 12,
                };
                
                if let Err(e) = event_sender.send(event) {
                    warn!("Failed to send event: {}", e);
                }
            }
        });
    }

    async fn handle_connection(
        stream: TcpStream,
        connections: Arc<RwLock<HashMap<Uuid, broadcast::Sender<String>>>>,
        mut event_receiver: broadcast::Receiver<WsEvent>,
    ) -> Result<()> {
        let connection_id = Uuid::new_v4();
        let (reader, writer) = stream.into_split();
        
        let mut lines = FramedRead::new(reader, LinesCodec::new());
        let mut sink = FramedWrite::new(writer, LinesCodec::new());
        
        // 创建连接专用的广播通道
        let (tx, mut rx) = broadcast::channel(100);
        connections.write().await.insert(connection_id, tx);

        // 处理输出消息
        let output_task = tokio::spawn(async move {
            while let Ok(message) = rx.recv().await {
                if let Err(e) = sink.send(message).await {
                    error!("Failed to send message: {}", e);
                    break;
                }
            }
        });

        // 处理事件广播
        let connections_clone = connections.clone();
        let event_task = tokio::spawn(async move {
            while let Ok(event) = event_receiver.recv().await {
                let message = match serde_json::to_string(&event) {
                    Ok(msg) => msg,
                    Err(e) => {
                        error!("Failed to serialize event: {}", e);
                        continue;
                    }
                };

                let connections = connections_clone.read().await;
                for (_, sender) in connections.iter() {
                    if let Err(e) = sender.send(message.clone()) {
                        warn!("Failed to broadcast message: {}", e);
                    }
                }
            }
        });

        // 处理输入消息（目前只是回显）
        while let Some(line) = lines.next().await {
            match line {
                Ok(msg) => {
                    info!("Received message: {}", msg);
                    // TODO: 处理客户端消息
                }
                Err(e) => {
                    error!("Error reading line: {}", e);
                    break;
                }
            }
        }

        // 清理连接
        connections.write().await.remove(&connection_id);
        output_task.abort();
        event_task.abort();
        
        info!("WebSocket connection {} closed", connection_id);
        Ok(())
    }

    /// 发送事件到所有连接的客户端
    pub async fn broadcast_event(&self, event: WsEvent) -> Result<()> {
        self.event_sender.send(event)?;
        Ok(())
    }
} 