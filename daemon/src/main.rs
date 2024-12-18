use tonic::transport::Server as GrpcServer;
use tracing::info;

mod hash;
mod service;
mod store;
mod ty;
mod vfs;
mod vfs_mgr;

use vfs_mgr::*;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let addr = "[::1]:10000".parse()?;

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

    info!("daemon started");

    let mut vfs_mgr = VfsManager::new(VfsManagerConfig {
        min_nfs_port: 12000,
        max_nfs_port: 12010,
    });

    let jj_svc = service::JujutsuService::new();

    let _store = store::Store::new();

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
