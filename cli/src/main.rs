#![deny(warnings)]

use jj_cli::{
    cli_util::{CliRunner, CommandHelper},
    command_error::{cli_error, user_error_with_message, CommandError},
    ui::Ui,
};
use jj_lib::{
    file_util,
    op_store::WorkspaceId,
    repo::{ReadonlyRepo, StoreFactories},
    signing::Signer,
    workspace::{WorkingCopyFactories, Workspace, WorkspaceInitError},
};

mod backend;
mod blocking_client;
mod working_copy;

use backend::YakBackend;
use jj_lib::{local_working_copy::LocalWorkingCopyFactory, working_copy::WorkingCopyFactory};
use working_copy::{YakWorkingCopy, YakWorkingCopyFactory};

/// Create a new repo in the given directory                                                                                                                  
///                                                                                                                                                           
/// If the given directory does not exist, it will be created. If no directory                                                                                
/// is given, the current directory is used.                                                                                                                  
#[derive(clap::Args, Clone, Debug)]
pub(crate) struct InitArgs {
    /// The destination directory
    #[arg(default_value = ".", value_hint = clap::ValueHint::DirPath)]
    destination: String,
}

#[derive(Debug, Clone, clap::Subcommand)]
enum YakCommands {
    Init(InitArgs),
    Status,
}

#[derive(Debug, Clone, clap::Args)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
struct YakArgs {
    #[command(subcommand)]
    command: YakCommands,
}

#[derive(clap::Parser, Clone, Debug)]
enum YakSubcommand {
    /// Commands for working with the yak daemon
    Yak(YakArgs),
}

fn create_store_factories() -> StoreFactories {
    let mut store_factories = StoreFactories::empty();
    // Register the backend so it can be loaded when the repo is loaded. The name
    // must match `Backend::name()`.
    store_factories.add_backend(
        "yak",
        Box::new(|settings, store_path| {
            Ok(Box::new(YakBackend::new(settings, store_path).unwrap()))
        }),
    );
    store_factories
}

pub fn default_working_copy_factory() -> Box<dyn WorkingCopyFactory> {
    Box::new(LocalWorkingCopyFactory {})
}

fn run_yak_command(
    ui: &mut Ui,
    command_helper: &CommandHelper,
    command: YakSubcommand,
) -> Result<(), CommandError> {
    let YakSubcommand::Yak(YakArgs { command }) = command;
    match command {
        YakCommands::Status => todo!(),
        YakCommands::Init(args) => {
            if command_helper.global_args().ignore_working_copy {
                return Err(cli_error("--ignore-working-copy is not respected"));
            }
            if command_helper.global_args().at_operation.is_some() {
                return Err(cli_error("--at-op is not respected"));
            }
            let cwd = command_helper.cwd();
            let wc_path = cwd.join(&args.destination);
            let wc_path = file_util::create_or_reuse_dir(&wc_path)
                .and_then(|_| wc_path.canonicalize())
                .map_err(|e| user_error_with_message("Failed to create workspace", e))?;

            let grpc_port = command_helper.settings().get::<usize>("grpc_port").unwrap();

            // NOTE: We need to tell the daemon to mount the filesystem BEFORE we
            // initalize the core jj internals or we'll have writes on-disk and on
            // vfs.
            let client = crate::blocking_client::BlockingJujutsuInterfaceClient::connect(format!(
                "http://[::1]:{grpc_port}"
            ))
            .unwrap();
            client
                .initialize(proto::jj_interface::InitializeReq {
                    path: wc_path.as_os_str().to_str().unwrap().to_string(),
                })
                .unwrap();

            Workspace::init_with_factories(
                command_helper.settings(),
                &wc_path,
                &|settings, store_path| {
                    let backend = YakBackend::new(settings, store_path)?;
                    Ok(Box::new(backend))
                },
                Signer::from_settings(command_helper.settings())
                    .map_err(WorkspaceInitError::SignInit)?,
                ReadonlyRepo::default_op_store_initializer(),
                ReadonlyRepo::default_op_heads_store_initializer(),
                ReadonlyRepo::default_index_store_initializer(),
                ReadonlyRepo::default_submodule_store_initializer(),
                //&YakWorkingCopyFactory {},
                &*default_working_copy_factory(),
                WorkspaceId::default(),
            )?;

            let relative_wc_path = file_util::relative_path(cwd, &wc_path);
            writeln!(
                ui.status(),
                "Initialized repo in \"{}\"",
                relative_wc_path.display()
            )?;

            Ok(())
        }
    }
}

fn main() -> std::process::ExitCode {
    let mut working_copy_factories = WorkingCopyFactories::new();
    working_copy_factories.insert(
        YakWorkingCopy::name().to_owned(),
        Box::new(YakWorkingCopyFactory {}),
    );
    // NOTE: logging before this point will not work since it is
    // initialized by CliRunner.
    CliRunner::init()
        .add_store_factories(create_store_factories())
        .add_working_copy_factories(working_copy_factories)
        .add_subcommand(run_yak_command)
        .run()
}
