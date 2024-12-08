use nfsserve::tcp::{NFSTcp, NFSTcpListener};
use rand::Rng;
use tokio::sync::mpsc;

use crate::vfs::VirtualFileSystem;

pub struct VfsManager {
    config: VfsManagerConfig,
    tx: mpsc::UnboundedSender<VfsManagerMessage>,
    rx: mpsc::UnboundedReceiver<VfsManagerMessage>,
}

pub struct VfsManagerConfig {
    pub min_nfs_port: usize,
    pub max_nfs_port: usize,
}

enum VfsManagerMessage {
    Bind,
}

/// Handle to the VFS Manager service
pub struct VfsManagerHandle(mpsc::UnboundedSender<VfsManagerMessage>);

impl VfsManagerHandle {
    pub fn bind(&self) -> anyhow::Result<()> {
        self.0.send(VfsManagerMessage::Bind)?;
        Ok(())
    }
}

impl VfsManager {
    pub fn new(config: VfsManagerConfig) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        VfsManager { config, tx, rx }
    }

    pub fn handle(&self) -> VfsManagerHandle {
        VfsManagerHandle(self.tx.clone())
    }

    pub async fn serve(&mut self) -> Result<(), std::io::Error> {
        while let Some(msg) = self.rx.recv().await {
            match msg {
                VfsManagerMessage::Bind => {
                    let port = rand::thread_rng()
                        .gen_range(self.config.min_nfs_port..self.config.max_nfs_port);
                    let _join_handle = tokio::spawn(async move {
                        let listener = NFSTcpListener::bind(
                            &format!("127.0.0.1:{port}"),
                            VirtualFileSystem::default(),
                        )
                        .await
                        .unwrap();
                        listener.handle_forever().await
                    });
                }
            }
        }
        unreachable!();
    }
}
