use std::{collections::HashMap, path::PathBuf};

use bollard_buildkit_proto::moby::{
    buildkit::v1::{control_client::ControlClient, CacheOptions, SolveRequest},
    filesync::v1::{auth_server::AuthServer, file_send_server::FileSendServer},
    upload::v1::upload_server::UploadServer,
};
use log::debug;
// use tonic::service::Interceptor;
use tonic::{
    codegen::InterceptedService, metadata::MetadataValue, service::Interceptor, transport::Channel,
};

use crate::{auth::DockerCredentials, grpc::build::ImageBuildFrontendOptionsIngest};

use super::{
    build::{ImageBuildFrontendOptions, ImageBuildLoadInput},
    error::GrpcError,
    export::ImageExporterRequest,
    registry::ImageRegistryOutput,
    GrpcServer,
};

/// The Docker Container driver opens a GRPC connection by instantiating a Buildkit container over
/// the traditional docker socket, and communicating over a docker execution Stdin/Stdout pipe.
pub mod docker_container;
/// The Moby driver opens a bi-directional GRPC connection by upgrading HTTP `/session` and `/grpc`
/// endpoints over the traditional docker socket.
pub mod moby;

/// TODO
pub trait Driver {
    /// TODO
    async fn grpc_handle(
        self,
        session_id: &str,
        services: Vec<GrpcServer>,
    ) -> Result<ControlClient<InterceptedService<Channel, DriverInterceptor>>, GrpcError>;
    /// TODO
    fn get_tear_down_handler(&self) -> Box<dyn DriverTearDownHandler>;
}

/// TODO
pub trait DriverTearDownHandler {
    /// TODO
    fn tear_down<'a>(
        &'a self,
    ) -> std::pin::Pin<Box<dyn futures_core::Future<Output = Result<(), GrpcError>> + 'a>>;
}

/// TODO
#[derive(Debug, Clone)]
pub struct DriverInterceptor {
    session_id: String,
    metadata_grpc_method: Vec<String>,
}

impl Interceptor for DriverInterceptor {
    fn call(&mut self, mut req: tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status> {
        let metadata = req.metadata_mut();

        metadata.insert(
            "x-docker-expose-session-uuid",
            self.session_id
                .parse()
                .map_err(|_| tonic::Status::invalid_argument("invalid 'session_id' argument"))?,
        );

        debug!("grpc-method: {:?}", self.metadata_grpc_method.join(","));
        for metadata_grpc_method_value in &self.metadata_grpc_method {
            let metadata_value = metadata_grpc_method_value
                .parse::<MetadataValue<tonic::metadata::Ascii>>()
                .map_err(|_| tonic::Status::invalid_argument("invalid grpc method name"))?;
            metadata.append("x-docker-expose-session-grpc-method", metadata_value);
        }

        Ok(req)
    }
}

/// TODO
#[derive(Debug, Clone)]
pub enum ImageExporterEnum {
    /// TODO
    OCI(ImageExporterRequest),
    /// TODO
    Docker(ImageExporterRequest),
}

/// TODO
pub trait Export {
    /// TODO
    async fn export(
        self,
        exporter_request: ImageExporterEnum,
        frontend_opts: ImageBuildFrontendOptions,
        load_input: ImageBuildLoadInput,
        credentials: Option<HashMap<&str, DockerCredentials>>,
    ) -> Result<(), GrpcError>;
}

/// TODO
pub trait Build {
    /// TODO
    async fn docker_build(
        self,
        name: &str,
        frontend_opts: ImageBuildFrontendOptions,
        load_input: ImageBuildLoadInput,
        credentials: Option<HashMap<&str, DockerCredentials>>,
    ) -> Result<(), GrpcError>;
}

/// TODO
pub trait Image {
    /// TODO
    async fn registry(
        self,
        output: ImageRegistryOutput,
        frontend_opts: ImageBuildFrontendOptions,
        load_input: ImageBuildLoadInput,
        credentials: Option<HashMap<&str, DockerCredentials>>,
    ) -> Result<(), GrpcError>;
}

pub(crate) async fn solve(
    driver: impl Driver,
    exporter: &str,
    exporter_attrs: HashMap<String, String>,
    path: Option<PathBuf>,
    frontend_opts: ImageBuildFrontendOptions,
    load_input: ImageBuildLoadInput,
    credentials: Option<HashMap<&str, DockerCredentials>>,
) -> Result<(), GrpcError> {
    let session_id = crate::grpc::new_id();

    let ImageBuildLoadInput::Upload(payload) = load_input;

    let mut upload_provider = super::UploadProvider::new();
    let context = upload_provider.add(payload.to_vec());

    let ImageBuildFrontendOptionsIngest {
        cache_to,
        cache_from,
        mut frontend_attrs,
    } = frontend_opts.consume();

    frontend_attrs.insert(String::from("context"), context);

    let mut auth_provider = super::AuthProvider::new();
    if let Some(creds) = credentials {
        for (host, docker_credentials) in creds {
            auth_provider.set_docker_credentials(host, docker_credentials);
        }
    }
    let auth = AuthServer::new(auth_provider);

    let upload = UploadServer::new(upload_provider);
    let mut services: Vec<GrpcServer> = vec![
        super::GrpcServer::Auth(auth),
        super::GrpcServer::Upload(upload),
    ];
    if let Some(path) = path {
        let filesend = FileSendServer::new(super::FileSendImpl::new(path.as_path()));

        services.push(super::GrpcServer::FileSend(filesend));
    }

    let tear_down_handler = driver.get_tear_down_handler();
    let mut control_client = driver.grpc_handle(&session_id, services).await?;

    let id = super::new_id();

    let solve_request = SolveRequest {
        r#ref: id,
        cache: Some(CacheOptions {
            export_ref_deprecated: String::new(),
            import_refs_deprecated: Vec::new(),
            export_attrs_deprecated: HashMap::new(),
            exports: cache_to,
            imports: cache_from,
        }),
        definition: None,
        entitlements: vec![],
        exporter: String::from(exporter),
        exporter_attrs,
        frontend: String::from("dockerfile.v0"),
        frontend_attrs,
        frontend_inputs: HashMap::new(),
        session: session_id,
    };

    debug!("sending solve request: {:#?}", solve_request);
    let res = control_client.solve(solve_request).await;
    debug!("solve res: {:#?}", res);

    // clean up

    tear_down_handler.tear_down().await?;
    // tear_down?;
    res?;

    Ok(())
}
