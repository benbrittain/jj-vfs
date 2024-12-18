use std::path::PathBuf;

use anyhow::anyhow;
use serde::Deserialize;
use tonic::transport::Server as GrpcServer;
use tracing::info;

mod hash;
mod service;
mod store;
mod ty;
mod vfs;
mod vfs_mgr;

use clap::Parser;
use vfs_mgr::*;

/// JJ Daemon
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Configuration
    #[arg(short, long)]
    config: PathBuf,
}

#[derive(Deserialize, Debug)]
struct Config {
    /// Address the jj CLI connects over
    pub grpc_addr: String,
    /// local cache
    pub cache: PathBuf,
    /// NFS configuration
    pub nfs: NfsConfig,
}

#[derive(Deserialize, Debug)]
struct NfsConfig {
    /// Minimum of the port range an NFS mount can be served over
    pub min_port: usize,
    /// Maximum of the port range an NFS mount can be served over
    pub max_port: usize,
}

async fn run_with_config(config: Config) -> Result<(), anyhow::Error> {
    info!("Starting daemon with configuration: {config:#?}");

    let addr = config.grpc_addr.parse()?;

    let mut vfs_mgr = VfsManager::new(VfsManagerConfig {
        min_nfs_port: config.nfs.min_port,
        max_nfs_port: config.nfs.max_port,
    });

    let jj_svc = service::JujutsuService::new();

    let reflection_svc = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build()?;

    info!("Serving jj gRPC interface");
    let grpc_fut = GrpcServer::builder()
        .add_service(reflection_svc)
        .add_service(jj_svc)
        .serve(addr);

    let nfs_fut = vfs_mgr.serve();
    tokio::select! {
        ret = nfs_fut => {
            panic!("NFS: {:?}", ret );
        }
        ret = grpc_fut => {
            panic!("GRPC: {:?}", ret );
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    let contents = std::fs::read_to_string(&args.config)
        .map_err(|e| anyhow!("Could not read {}: {}", args.config.display(), e))?;

    let config: Config = toml::from_str(&contents)?;

    tracing_log::LogTracer::init()?;

    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .finish();

    // use that subscriber to process traces emitted after this point
    tracing::subscriber::set_global_default(subscriber)?;

    Ok(run_with_config(config).await?)
}
