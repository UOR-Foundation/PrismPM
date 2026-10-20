use axum::body::to_bytes;
use axum::extract::{Request, State};
use axum::http::{header, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Router;
use hologram::archive::HoloWriter;
use hologram::space::{address_bytes, AppManifest, Layer, Realization};
use hologram_live::holo::{inspect_bytes, plan_bytes, HoloCatalog, HoloExecutor, HoloRuntime};
use hologram_live::store::ObjectStore;
use hologram_view_surface::{
    intent_limits, PortableViewAttachment, PortableViewSurface, SurfaceFuture, ViewAttachmentId,
    ViewIntentRequest, APPLICATION_INVOKE_INTENT, VIEW_INTENT_VERSION, VIEW_INTENT_VERSION_2,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::process::Stdio;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[derive(Default)]
struct RecordingSurface {
    attachment: Mutex<Option<PortableViewAttachment>>,
    attached: AtomicUsize,
    detached: AtomicUsize,
}

// An acceptance adapter, never an application server or deployment target.
// Axum/Hyper own HTTP parsing. Immutable assets and invocations come exclusively
// from the pinned upstream attachment, not a reimplementation of the guest.
#[derive(Clone)]
struct BrowserSurface {
    surface: Arc<RecordingSurface>,
    authority: String,
    intent_version: u16,
    model_id: Option<String>,
    binding_path: Option<String>,
}

fn intent_version(application: &Value) -> u16 {
    if application["profile"].as_str() == Some("prismpm/text-application/4") {
        VIEW_INTENT_VERSION_2
    } else {
        VIEW_INTENT_VERSION
    }
}

fn response(status: StatusCode, kind: &'static str, bytes: Vec<u8>) -> Response {
    (
        status,
        [
            (header::CONTENT_TYPE, kind),
            (header::CACHE_CONTROL, "no-store"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        bytes,
    )
        .into_response()
}

fn asset_query_allowed(
    host: &BrowserSurface,
    method: &Method,
    path: &str,
    query: Option<&str>,
) -> bool {
    let Some(query) = query else {
        return true;
    };
    host.intent_version == VIEW_INTENT_VERSION_2
        && method == Method::GET
        && (matches!(path, "/app.js" | "/app.css") || host.binding_path.as_deref() == Some(path))
        && host
            .model_id
            .as_ref()
            .is_some_and(|id| query == format!("model_id={id}"))
}

#[cfg(test)]
mod research_asset_query_tests {
    use super::*;

    #[tokio::test]
    async fn research_asset_query_is_exact_current_model_and_asset() {
        let model_id = "a".repeat(64);
        let host = BrowserSurface {
            surface: Arc::new(RecordingSurface::default()),
            authority: "127.0.0.1:4321".to_owned(),
            intent_version: VIEW_INTENT_VERSION_2,
            model_id: Some(model_id.clone()),
            binding_path: Some("/fixture.js".to_owned()),
        };
        let query = format!("model_id={model_id}");
        for path in ["/app.js", "/app.css", "/fixture.js"] {
            let request = Request::builder()
                .uri(format!("{path}?{query}"))
                .header(header::HOST, &host.authority)
                .body(axum::body::Body::empty())
                .unwrap();
            assert_eq!(
                portable_http(State(host.clone()), request).await.status(),
                StatusCode::GONE,
                "exact model query passes admission and reaches the absent attachment"
            );
        }
        for (method, path, query) in [
            (
                Method::GET,
                "/app.js",
                format!("model_id={}", "b".repeat(64)),
            ),
            (Method::GET, "/app.js", format!("{query}&extra=1")),
            (Method::GET, "/app.js", format!("{query}&{query}")),
            (
                Method::GET,
                "/app.js",
                format!("model_id=%61{}", "a".repeat(63)),
            ),
            (Method::GET, "/fixture_bg.wasm", query.clone()),
            (Method::GET, "/other.js", query.clone()),
            (Method::POST, "/app.js", query.clone()),
            (Method::HEAD, "/app.css", query.clone()),
        ] {
            let request = Request::builder()
                .method(method)
                .uri(format!("{path}?{query}"))
                .header(header::HOST, &host.authority)
                .body(axum::body::Body::empty())
                .unwrap();
            assert_eq!(
                portable_http(State(host.clone()), request).await.status(),
                StatusCode::BAD_REQUEST
            );
        }
        let legacy = BrowserSurface {
            intent_version: VIEW_INTENT_VERSION,
            ..host.clone()
        };
        assert!(!asset_query_allowed(
            &legacy,
            &Method::GET,
            "/app.js",
            Some(&query)
        ));
        assert!(asset_query_allowed(&legacy, &Method::GET, "/app.js", None));
        assert!(asset_query_allowed(
            &host,
            &Method::GET,
            "/fixture_bg.wasm",
            None
        ));
    }
}

async fn portable_http(State(host): State<BrowserSurface>, request: Request) -> Response {
    let reject = |status| response(status, "text/plain; charset=utf-8", Vec::new());
    let origin = format!("http://{}", host.authority);
    if request
        .headers()
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        != Some(host.authority.as_str())
        || !asset_query_allowed(
            &host,
            request.method(),
            request.uri().path(),
            request.uri().query(),
        )
    {
        return reject(StatusCode::BAD_REQUEST);
    }
    if request.headers().contains_key(header::ORIGIN)
        && request
            .headers()
            .get(header::ORIGIN)
            .and_then(|v| v.to_str().ok())
            != Some(origin.as_str())
    {
        return reject(StatusCode::FORBIDDEN);
    }
    let attachment = match host.surface.attachment.lock() {
        Ok(value) => value.clone(),
        Err(_) => return reject(StatusCode::INTERNAL_SERVER_ERROR),
    };
    let Some(attachment) = attachment else {
        return reject(StatusCode::GONE);
    };
    if request.uri().path() == "/_hologram/intent" {
        if request.method() != Method::POST {
            return reject(StatusCode::METHOD_NOT_ALLOWED);
        }
        if request
            .headers()
            .get(header::ORIGIN)
            .and_then(|v| v.to_str().ok())
            != Some(origin.as_str())
        {
            return reject(StatusCode::FORBIDDEN);
        }
        if request
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            != Some("application/json")
        {
            return reject(StatusCode::UNSUPPORTED_MEDIA_TYPE);
        }
        // JSON may represent a UTF-8 payload byte using six ASCII bytes. The
        // authoritative DTO separately enforces this version's decoded bound.
        let limits = match intent_limits(host.intent_version) {
            Ok(limits) => limits,
            Err(_) => return reject(StatusCode::INTERNAL_SERVER_ERROR),
        };
        let maximum = limits.payload_bytes * 6 + 256;
        let body = match to_bytes(request.into_body(), maximum).await {
            Ok(body) => body,
            Err(_) => return reject(StatusCode::PAYLOAD_TOO_LARGE),
        };
        let intent: ViewIntentRequest = match serde_json::from_slice(&body) {
            Ok(value) => value,
            Err(_) => return reject(StatusCode::BAD_REQUEST),
        };
        if intent.version != host.intent_version || intent.validate().is_err() {
            return reject(StatusCode::BAD_REQUEST);
        }
        match attachment.intents.handle(&attachment.id, intent).await {
            Ok(result) if result.version == host.intent_version => {
                match serde_json::to_vec(&result) {
                    Ok(bytes) => response(StatusCode::OK, "application/json", bytes),
                    Err(_) => reject(StatusCode::INTERNAL_SERVER_ERROR),
                }
            }
            Ok(_) => reject(StatusCode::INTERNAL_SERVER_ERROR),
            Err(_) => reject(StatusCode::BAD_REQUEST),
        }
    } else {
        if request.method() != Method::GET && request.method() != Method::HEAD {
            return reject(StatusCode::METHOD_NOT_ALLOWED);
        }
        let path = if request.uri().path() == "/" {
            attachment.entry.as_str()
        } else {
            request.uri().path().strip_prefix('/').unwrap_or("")
        };
        let Some(asset) = attachment.assets.iter().find(|asset| asset.path == path) else {
            return reject(StatusCode::NOT_FOUND);
        };
        let mime = match asset.path.as_str() {
            "index.html" => "text/html; charset=utf-8",
            "app.js" => "text/javascript; charset=utf-8",
            "app.css" => "text/css; charset=utf-8",
            _ => return reject(StatusCode::NOT_FOUND),
        };
        response(
            StatusCode::OK,
            mime,
            if request.method() == Method::HEAD {
                Vec::new()
            } else {
                asset.bytes.to_vec()
            },
        )
    }
}

async fn browser_line(
    reader: &mut BufReader<tokio::process::ChildStdout>,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut line = Vec::new();
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(120);
    // Do not permit a child to allocate an unbounded report before validation.
    loop {
        let available = tokio::time::timeout_at(deadline, reader.fill_buf()).await??;
        if available.is_empty() {
            return Err("browser oracle closed stdout before evidence".into());
        }
        let count = available
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(available.len(), |n| n + 1);
        if line.len() + count > 65_536 {
            return Err("browser evidence exceeds 64KiB".into());
        }
        line.extend_from_slice(&available[..count]);
        reader.consume(count);
        if line.last() == Some(&b'\n') {
            return Ok(serde_json::from_slice(&line)?);
        }
    }
}

async fn browser_start(
    surface: Arc<RecordingSurface>,
    application: &Value,
    model_id: &str,
    script: &std::path::Path,
    node: &std::path::Path,
    browser: &std::path::Path,
) -> Result<
    (
        tokio::process::Child,
        BufReader<tokio::process::ChildStdout>,
        tokio::task::JoinHandle<std::io::Result<()>>,
        Value,
    ),
    Box<dyn std::error::Error>,
> {
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).await?;
    let authority = listener.local_addr()?.to_string();
    let host = BrowserSurface {
        surface,
        authority: authority.clone(),
        intent_version: intent_version(application),
        model_id: (intent_version(application) == VIEW_INTENT_VERSION_2)
            .then(|| model_id.to_owned()),
        binding_path: (intent_version(application) == VIEW_INTENT_VERSION_2).then(|| {
            format!(
                "/{}.js",
                application["cargo_name"]
                    .as_str()
                    .unwrap_or("")
                    .replace('-', "_")
            )
        }),
    };
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            Router::new().fallback(portable_http).with_state(host),
        )
        .await
    });
    if !node.is_absolute() || !node.is_file() || !browser.is_absolute() || !browser.is_file() {
        return Err("controller must supply verified absolute Node and browser executables".into());
    }
    let mut child = tokio::process::Command::new(node)
        .arg(script)
        // Both the SDK and source devcontainer install the same pinned image
        // payload here. Never inherit a caller-selected browser cache path.
        .env("PLAYWRIGHT_BROWSERS_PATH", "/ms-playwright")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .kill_on_drop(true)
        .spawn()?;
    let mut browser_input = json!({"origin":format!("http://{authority}"), "application":application, "browser_executable":browser});
    if intent_version(application) == VIEW_INTENT_VERSION_2 {
        browser_input["model_id"] = json!(model_id);
    }
    child
        .stdin
        .as_mut()
        .ok_or("browser stdin missing")?
        .write_all(format!("{}\n", browser_input).as_bytes())
        .await?;
    let mut reader = BufReader::new(child.stdout.take().ok_or("browser stdout missing")?);
    let live = browser_line(&mut reader).await?;
    if live != json!({"stage":"live"}) {
        return Err("browser did not finish live journeys".into());
    }
    Ok((child, reader, server, json!({"stage":"stopped"})))
}

impl PortableViewSurface for RecordingSurface {
    fn attach(&self, view: PortableViewAttachment) -> SurfaceFuture<'_> {
        Box::pin(async move {
            *self
                .attachment
                .lock()
                .map_err(|_| "attachment lock poisoned".to_owned())? = Some(view);
            self.attached.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
    }

    fn detach<'a>(&'a self, _id: &'a ViewAttachmentId) -> SurfaceFuture<'a> {
        Box::pin(async move {
            *self
                .attachment
                .lock()
                .map_err(|_| "attachment lock poisoned".to_owned())? = None;
            self.detached.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
    }
}

fn bytes(value: &Value, field: &str) -> Result<Vec<u8>, String> {
    value[field]
        .as_array()
        .ok_or_else(|| format!("acceptance vector has no {field}"))?
        .iter()
        .map(|byte| {
            byte.as_u64()
                .and_then(|byte| u8::try_from(byte).ok())
                .ok_or_else(|| format!("acceptance {field} contains a non-byte"))
        })
        .collect()
}

fn allocation_boundary(
    profile: Option<&str>,
    allocation_cap: usize,
    vectors: &[(Vec<u8>, Vec<u8>)],
) -> Result<(Vec<u8>, Vec<u8>), String> {
    if matches!(
        profile,
        Some(
            "prismpm/text-application/1"
                | "prismpm/text-application/2"
                | "prismpm/text-application/3"
                | "prismpm/text-application/4"
        )
    ) {
        vectors
            .iter()
            .find(|(request, _)| request.len() == allocation_cap)
            .cloned()
            .ok_or_else(|| "text model has no explicit guest-allocation-boundary vector".to_owned())
    } else {
        let (_, response) = vectors
            .iter()
            .find(|(request, _)| request.is_empty())
            .ok_or("model has no empty malformed-request vector")?;
        Ok((vec![b'x'; allocation_cap], response.clone()))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = std::env::args_os().skip(1);
    let archive_path = arguments.next().ok_or("archive path is required")?;
    let model_path = arguments.next().ok_or("model path is required")?;
    let wasm_path = arguments.next().ok_or("Core-Wasm path is required")?;
    let browser_script = arguments
        .next()
        .ok_or("portable browser oracle script is required")?;
    let node = arguments
        .next()
        .ok_or("verified Node executable is required")?;
    let browser = arguments
        .next()
        .ok_or("verified browser executable is required")?;
    if arguments.next().is_some() {
        return Err("unexpected argument".into());
    }
    let archive = std::fs::read(archive_path)?;
    let wasm = std::fs::read(wasm_path)?;
    let model_bytes = std::fs::read(model_path)?;
    let model_id = format!("{:x}", Sha256::digest(&model_bytes));
    let model: Value = serde_json::from_slice(&model_bytes)?;
    let application = &model["application"];
    let intent_version = intent_version(application);
    let research = application["profile"].as_str() == Some("prismpm/text-application/4");
    // Match the production fail-fast check to authoritative upstream constants,
    // before guest invocation or any model-sized negative-case allocation.
    if VIEW_INTENT_VERSION != 1
        || VIEW_INTENT_VERSION_2 != 2
        || hologram_view_surface::MAX_INTENT_PAYLOAD_BYTES != 65_536
        || hologram_view_surface::MAX_INTENT_OUTPUT_BYTES != 1_048_576
        || hologram_view_surface::MAX_INTENT_V2_PAYLOAD_BYTES != 2_097_152
        || hologram_view_surface::MAX_INTENT_V2_OUTPUT_BYTES != 4_194_304
        || intent_limits(VIEW_INTENT_VERSION)?.payload_bytes != 65_536
        || intent_limits(VIEW_INTENT_VERSION)?.output_bytes != 1_048_576
        || intent_limits(VIEW_INTENT_VERSION_2)?.payload_bytes != 2_097_152
        || intent_limits(VIEW_INTENT_VERSION_2)?.output_bytes != 4_194_304
    {
        return Err("pinned portable View transport limits changed".into());
    }
    let limits = intent_limits(intent_version)?;
    for (field, maximum) in [
        ("request_maximum", limits.payload_bytes),
        ("response_maximum", limits.output_bytes),
    ] {
        if application[field]
            .as_u64()
            .is_none_or(|value| value == 0 || value > maximum as u64)
        {
            return Err("application bounds exceed pinned portable View transport limits".into());
        }
    }
    let vectors = application["acceptance_vectors"]
        .as_array()
        .ok_or("model has no acceptance vectors")?;
    if vectors.is_empty() {
        return Err("model acceptance corpus is empty".into());
    }

    let name = application["name"]
        .as_str()
        .ok_or("application name is required")?;
    let inspection = inspect_bytes("oracle", &format!("{name}.holo"), &archive)?;
    if inspection.format_version != 4 || !inspection.footer_verified {
        return Err("upstream inspection did not verify Hologram v4/footer".into());
    }
    let directory = inspection
        .directory
        .as_ref()
        .ok_or("upstream inspection found no application directory")?;
    if !inspection.directory_embedded
        || directory.primary_layer != Some(0)
        || directory.layers.len() != 2
        || directory.layers[0].kind != "wasm"
        || directory.layers[0].entry != "holo_run"
        || directory.layers[0].contract.as_deref() != Some("hologram:guest/core-wasm@1")
        || directory.layers[1].kind != "view"
        || directory.layers[1].surface.as_deref() != Some("portable")
        || directory.blobs.len() != 4
    {
        return Err("upstream application directory disagrees with the application profile".into());
    }

    let plan = serde_json::to_value(plan_bytes(&archive)?)?;
    if plan["execution_target"] != "direct"
        || plan["runnable"] != false
        || plan["layers"][0]["provider"]["name"] != "wasmtime-direct"
        || plan["layers"][1]["provider"]["status"] != "unavailable"
        || plan["blockers"][0]["error_code"] != "LIVE_CAPABILITY_MISSING"
    {
        return Err(
            "upstream headless plan did not report the exact portable-surface blocker".into(),
        );
    }

    let registry = Arc::new(hologram_view_surface::ViewSurfaceRegistry::new());
    let surface = Arc::new(RecordingSurface::default());
    registry.register_portable(surface.clone())?;
    let session = HoloExecutor::with_view_surfaces(registry)
        .start_session(&archive)
        .await?;
    if surface.attached.load(Ordering::SeqCst) != 1 {
        return Err("portable View did not attach exactly once".into());
    }
    let attachment = surface
        .attachment
        .lock()
        .map_err(|_| "attachment lock poisoned")?
        .clone()
        .ok_or("portable View attachment was not retained")?;
    let paths = attachment
        .assets
        .iter()
        .map(|asset| asset.path.as_str())
        .collect::<Vec<_>>();
    if attachment.entry != "index.html" || paths != ["app.css", "app.js", "index.html"] {
        return Err("attached portable View asset closure/order is not canonical".into());
    }

    let mut intent_count = 0usize;
    let mut intent_transport_rejections = 0usize;
    let mut lifecycle_request = None;
    for vector in vectors {
        let request = bytes(vector, "request")?;
        let response = bytes(vector, "response")?;
        let direct = session.invoke(vec![request.clone()]).await?;
        if direct.outputs != vec![response.clone()] {
            return Err("upstream direct execution disagrees with a modeled vector".into());
        }
        if let (Ok(payload), Ok(expected)) =
            (String::from_utf8(request), String::from_utf8(response))
        {
            let request = ViewIntentRequest {
                version: intent_version,
                name: APPLICATION_INVOKE_INTENT.to_owned(),
                payload,
            };
            let oversized = request.payload.len() > limits.payload_bytes;
            let intent = attachment
                .intents
                .handle(&attachment.id, request.clone())
                .await;
            if research && oversized {
                if intent.is_ok() {
                    return Err("oversized modeled UTF-8 intent was admitted".into());
                }
                intent_transport_rejections += 1;
            } else {
                let intent = intent?;
                if intent.version != intent_version || intent.outputs != vec![expected.clone()] {
                    return Err("portable View intent disagrees with a modeled vector".into());
                }
                lifecycle_request.get_or_insert((request, expected));
                intent_count += 1;
            }
        }
    }
    if research {
        // This is one byte beyond the decoded budget, despite containing fewer
        // Unicode scalars than that budget. A character-count mutant must fail.
        let mut payload = "é".repeat(limits.payload_bytes / 2);
        payload.push('x');
        if payload.len() != limits.payload_bytes + 1 {
            return Err("UTF-8 transport-boundary fixture has the wrong byte length".into());
        }
        let request = ViewIntentRequest {
            version: intent_version,
            name: APPLICATION_INVOKE_INTENT.to_owned(),
            payload,
        };
        if attachment
            .intents
            .handle(&attachment.id, request)
            .await
            .is_ok()
        {
            return Err("first over-bound multibyte UTF-8 intent was admitted".into());
        }
        intent_transport_rejections += 1;
    }
    let allocation_cap = usize::try_from(
        application["guest_allocation_maximum"]
            .as_u64()
            .ok_or("model has no guest allocation cap")?,
    )?;
    let boundary_vectors = vectors
        .iter()
        .map(|vector| Ok((bytes(vector, "request")?, bytes(vector, "response")?)))
        .collect::<Result<Vec<_>, String>>()?;
    let (boundary_request, boundary_response) = allocation_boundary(
        application["profile"].as_str(),
        allocation_cap,
        &boundary_vectors,
    )?;
    let at_cap = session.invoke(vec![boundary_request]).await?;
    if at_cap.outputs != vec![boundary_response] {
        return Err("guest allocation cap did not return the modeled boundary response".into());
    }
    if session
        .invoke(vec![vec![b'x'; allocation_cap + 1]])
        .await
        .is_ok()
    {
        return Err("first over-cap Core-Wasm request did not fail".into());
    }

    let (mut browser, mut browser_output, server, stopped) = browser_start(
        surface.clone(),
        application,
        &model_id,
        std::path::Path::new(&browser_script),
        std::path::Path::new(&node),
        std::path::Path::new(&browser),
    )
    .await?;
    // Reuse an actually successful modeled intent, not a numeric payload that
    // a different application could reject equally before and after stop.
    let (lifecycle_request, expected) = lifecycle_request.ok_or("no live lifecycle intent")?;
    let live = attachment
        .intents
        .handle(&attachment.id, lifecycle_request.clone())
        .await?;
    if live.version != intent_version || live.outputs != vec![expected] {
        return Err("lifecycle intent no longer succeeds before stop".into());
    }
    session.stop().await?;
    session.stop().await?;
    if surface.detached.load(Ordering::SeqCst) != 1 {
        return Err("portable View did not detach exactly once".into());
    }
    if attachment
        .intents
        .handle(&attachment.id, lifecycle_request)
        .await
        .is_ok()
    {
        return Err("stale portable View intent remained usable after stop".into());
    }
    browser
        .stdin
        .as_mut()
        .ok_or("browser stdin missing")?
        .write_all(format!("{stopped}\n").as_bytes())
        .await?;
    let portable_browser = browser_line(&mut browser_output).await?;
    browser.stdin.take();
    let status = tokio::time::timeout(std::time::Duration::from_secs(15), browser.wait()).await??;
    server.abort();
    if !status.success() {
        return Err("portable browser oracle failed".into());
    }

    // Resident Hologram deliberately has a headless View provider, so use the
    // exact modeled primary bytes in an upstream-written primary-only archive.
    // The composed archive itself was already checked above, including its
    // mandatory unavailable-surface result under headless planning.
    let capabilities = hologram_live::holo_capability::empty_canonical();
    let manifest = AppManifest {
        primary: Some(0),
        requires: address_bytes(&capabilities),
        layers: vec![Layer::wasm_with_contract(
            address_bytes(&wasm),
            "holo_run",
            "hologram:guest/core-wasm@1",
        )],
        children: Vec::new(),
    };
    let addressed = [
        (address_bytes(&capabilities), capabilities.as_slice()),
        (address_bytes(&wasm), wasm.as_slice()),
    ];
    let directory = hologram_live::holo_directory::derive(
        &manifest,
        addressed
            .iter()
            .map(|(kappa, content)| (kappa.as_bytes(), *content)),
    )?;
    let mut writer = HoloWriter::new();
    writer.set_app_manifest(manifest.canonicalize());
    writer.add_extension(
        hologram_live::holo_directory::DIRECTORY_EXTENSION_KEY,
        hologram_live::holo_directory::encode(&directory)?,
    );
    for (kappa, content) in addressed {
        writer.add_content_blob(kappa.as_bytes(), content);
    }
    let resident_archive = writer.finish()?;
    let store_root = tempfile::tempdir()?;
    let store = Arc::new(ObjectStore::open(store_root.path())?);
    let catalog = Arc::new(HoloCatalog::new(store));
    let resident_kappa = catalog
        .import(format!("{name}-primary.holo"), resident_archive)?
        .kappa;
    let runtime = HoloRuntime::new(catalog, 8);
    runtime.load(&resident_kappa).await?;
    for vector in vectors {
        let response = runtime
            .run(&resident_kappa, vec![bytes(vector, "request")?])
            .await?;
        if response.outputs != vec![bytes(vector, "response")?] {
            return Err("upstream resident execution disagrees with a modeled vector".into());
        }
    }
    runtime.unload(&resident_kappa).await?;
    runtime.unload(&resident_kappa).await?;

    let mut report = json!({
        "application_kappa": inspection.application_kappa,
        "archive_fingerprint": inspection.archive_fingerprint,
        "archive_kappa": session.archive_kappa(),
        "direct_vectors": vectors.len(),
        "footer_verified": true,
        "guest_allocation_boundary": "verified",
        "intent_vectors": intent_count,
        "resident_vectors": vectors.len(),
        "portable_browser": portable_browser,
        "schema": "prismpm/hologram-oracle/2",
        "view_attached": 1,
        "view_detached": 1
    });
    if research {
        report["intent_transport_rejections"] = json!(intent_transport_rejections);
    }
    println!("{}", serde_json::to_string(&report)?);
    Ok(())
}
