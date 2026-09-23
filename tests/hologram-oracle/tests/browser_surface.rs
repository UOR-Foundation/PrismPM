//! Pinned Live compatibility boundary, not PrismPM browser-runtime acceptance.
//! The wire fixture contributes only its decoded selector. Executable content
//! comes from the unchanged pinned Live example, never the synthetic wire blobs.

use hologram::archive::{HoloLoader, HoloWriter};
use hologram::space::{address_bytes, AppManifest, LayerKind, Realization};
use hologram_live::application_plan::{
    explain_application, PlanLimits, ProviderContext, ResolutionSource, ResolvedLayer,
};
use hologram_live::holo::{inspect_bytes, plan_bytes, HoloExecutor};
use hologram_live::holo_capability::{EffectiveGrant, RequestedCapabilities};
use hologram_live::holo_provider::{LayerPrepareContext, LayerProvider, ProviderTarget};
use hologram_live::holo_view_provider::ViewProvider;
use hologram_live::{holo_directory, holo_view, LiveError};
use hologram_view_surface::{
    IntentFuture, PortableViewAttachment, PortableViewIntentHandler, PortableViewSurface,
    SurfaceFuture, ViewAttachmentId, ViewIntentRequest, ViewSurfaceRegistry,
};
use std::path::Path;
use std::sync::{Arc, Mutex};

const REFUSAL: &str = "unsupported View surface \"prismpm-browser/1\"; expected \"portable\"";

#[derive(Default)]
struct RecordingSurface {
    events: Mutex<Vec<&'static str>>,
}

impl PortableViewSurface for RecordingSurface {
    fn attach(&self, view: PortableViewAttachment) -> SurfaceFuture<'_> {
        Box::pin(async move {
            assert_eq!(view.entry, "index.html");
            assert!(view.assets.iter().any(|asset| asset.path == "index.html"));
            self.events.lock().expect("events").push("attach");
            Ok(())
        })
    }

    fn detach<'a>(&'a self, _id: &'a ViewAttachmentId) -> SurfaceFuture<'a> {
        Box::pin(async move {
            self.events.lock().expect("events").push("detach");
            Ok(())
        })
    }
}

struct UnusedIntents;

impl PortableViewIntentHandler for UnusedIntents {
    fn handle<'a>(
        &'a self,
        _id: &'a ViewAttachmentId,
        _request: ViewIntentRequest,
    ) -> IntentFuture<'a> {
        Box::pin(async { panic!("provider-only control must not invoke an application") })
    }
}

fn registered_surface() -> (Arc<RecordingSurface>, Arc<ViewSurfaceRegistry>) {
    let surface = Arc::new(RecordingSurface::default());
    let registry = Arc::new(ViewSurfaceRegistry::new());
    registry
        .register_portable(surface.clone())
        .expect("register real portable surface");
    (surface, registry)
}

fn browser_selector() -> String {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../data/holo-browser-codec-v1.json"))
            .expect("wire fixture");
    let hex = fixture["manifest"]["bytes_hex"]
        .as_str()
        .expect("canonical manifest hex");
    assert_eq!(hex.len() % 2, 0);
    let bytes = (0..hex.len())
        .step_by(2)
        .map(|offset| u8::from_str_radix(&hex[offset..offset + 2], 16).expect("hex byte"))
        .collect::<Vec<_>>();
    let manifest = AppManifest::decode(&bytes).expect("upstream decodes browser wire manifest");
    manifest.validate().expect("valid browser wire manifest");
    assert_eq!(manifest.canonicalize(), bytes);
    assert_eq!(manifest.layers.len(), 2);
    assert_eq!(manifest.layers[1].kind, LayerKind::View);
    assert_eq!(manifest.layers[1].aux, "prismpm-browser/1");
    assert_eq!(
        fixture["manifest"]["layers"][1]["aux"],
        manifest.layers[1].aux
    );
    manifest.layers[1].aux.clone()
}

fn executable_archives() -> (Vec<u8>, Vec<u8>) {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../hologram-live/examples/wasm-view/hologram.json");
    let compiled = hologram_live::compile::compile_manifest(&source)
        .expect("compile genuine pinned upstream guest and portable View");
    let plan = HoloLoader::from_bytes(&compiled.bytes)
        .expect("physical archive")
        .into_plan()
        .expect("archive plan");
    let mut manifest =
        AppManifest::decode(plan.app_manifest().expect("manifest")).expect("manifest decode");
    assert_eq!(manifest.primary, Some(0));
    assert_eq!(manifest.layers.len(), 2);
    assert_eq!(manifest.layers[1].kind, LayerKind::View);
    assert_eq!(manifest.layers[1].aux, holo_view::PORTABLE_SURFACE);
    let blobs = plan.content_blobs().expect("addressed executable closure");

    // Only the selector changes. Regenerate its canonical manifest, directory,
    // and archive fingerprint; retain every genuine content blob unchanged.
    manifest.layers[1].aux = browser_selector();
    let directory = holo_directory::derive(&manifest, blobs.iter().copied())
        .expect("rederive browser-selector directory");
    let mut writer = HoloWriter::new();
    writer.set_app_manifest(manifest.canonicalize());
    writer.add_extension(
        holo_directory::DIRECTORY_EXTENSION_KEY,
        holo_directory::encode(&directory).expect("directory encoding"),
    );
    for (label, content) in blobs {
        writer.add_content_blob(label, content);
    }
    let browser = writer
        .finish()
        .expect("readdressed browser-selector archive");
    inspect_bytes("browser-selector", "upstream-example.holo", &browser)
        .expect("complete browser-selector archive passes upstream physical/directory inspection");
    (compiled.bytes, browser)
}

fn prepare_context(bytes: &[u8], target: ProviderTarget) -> LayerPrepareContext {
    let report = explain_application(bytes, PlanLimits::default(), |_| Ok(None))
        .expect("resolve genuine closed application");
    assert!(report.blockers.is_empty());
    let plan = HoloLoader::from_bytes(bytes)
        .expect("physical archive")
        .into_plan()
        .expect("archive plan");
    let manifest =
        AppManifest::decode(plan.app_manifest().expect("manifest")).expect("manifest decode");
    let blobs = plan.content_blobs().expect("content closure");
    let content = |label: &str| -> Arc<[u8]> {
        blobs
            .iter()
            .find(|(key, _)| *key == label.as_bytes())
            .map(|(_, bytes)| Arc::from(*bytes))
            .expect("embedded addressed object")
    };
    let view = &manifest.layers[1];
    let view_bytes = content(view.content.as_str());
    holo_view::decode(&view_bytes).expect("genuine upstream View bundle");
    assert_eq!(address_bytes(&view_bytes), view.content);
    LayerPrepareContext {
        identity: report.identity,
        effective_grant: EffectiveGrant::local_baseline(),
        requested_capabilities: RequestedCapabilities::decode(
            manifest.requires.as_str(),
            content(manifest.requires.as_str()),
        )
        .expect("real empty canonical capability set"),
        layer: ResolvedLayer {
            position: 1,
            kind: view.kind,
            content_kappa: view.content.to_string(),
            entry: view.entry.clone(),
            aux: view.aux.clone(),
            primary: false,
            content: view_bytes,
            resolution_source: ResolutionSource::Embedded,
            provider: "portable-view".to_owned(),
        },
        target,
        view_intents: Arc::new(UnusedIntents),
    }
}

#[tokio::test]
async fn pinned_provider_refuses_browser_selector_before_preparation_or_attachment() {
    let (portable, browser) = executable_archives();
    for target in [ProviderTarget::Direct, ProviderTarget::Resident] {
        let (control, registry) = registered_surface();
        let provider = ViewProvider::new(registry);
        let prepared = provider
            .prepare(prepare_context(&portable, target))
            .await
            .expect("supported selector prepares the identical valid View");
        assert!(control.events.lock().expect("events").is_empty());
        prepared.start().await.expect("positive attachment");
        prepared.stop().await.expect("positive detach");
        assert_eq!(
            *control.events.lock().expect("events"),
            ["attach", "detach"]
        );

        let (surface, registry) = registered_surface();
        let provider = ViewProvider::new(registry);
        let context = prepare_context(&browser, target);
        let availability = provider.availability(
            &ProviderContext {
                application_kappa: &context.identity.application_kappa,
                position: 1,
                kind: context.layer.kind,
                entry: &context.layer.entry,
                aux: &context.layer.aux,
                primary: false,
                layer_count: 2,
                content: &context.layer.content,
            },
            target,
        );
        assert_eq!(availability, Err(REFUSAL.to_owned()));
        match provider.prepare(context).await {
            Err(error @ LiveError::Config(_)) => {
                assert_eq!(error.code(), "LIVE_CONFIG_INVALID");
                assert_eq!(error.to_string(), REFUSAL);
            }
            Err(error) => panic!("wrong refusal: {error:?}"),
            Ok(_) => panic!("unsupported selector produced a prepared layer"),
        }
        assert!(surface.events.lock().expect("events").is_empty());
    }
}

#[tokio::test]
async fn pinned_executor_refuses_browser_selector_before_session_start() {
    let (portable, browser) = executable_archives();
    let (control, registry) = registered_surface();
    let session = HoloExecutor::with_view_surfaces(registry)
        .start_session(&portable)
        .await
        .expect("genuine pinned upstream application runs");
    let outcome = session
        .invoke(vec![b"portable control".to_vec()])
        .await
        .expect("execute real Wasm guest");
    assert_eq!(outcome.outputs, vec![b"PORTABLE CONTROL".to_vec()]);
    session.stop().await.expect("stop positive control");
    assert_eq!(
        *control.events.lock().expect("events"),
        ["attach", "detach"]
    );

    let plan = plan_bytes(&browser).expect("resolve the complete valid browser-selector archive");
    assert!(!plan.runnable);
    assert_eq!(plan.layers.len(), 2);
    assert_eq!(plan.layers[0].provider.status, "available");
    assert_eq!(plan.layers[1].provider.status, "unavailable");
    assert_eq!(plan.layers[1].provider.reason.as_deref(), Some(REFUSAL));
    assert_eq!(plan.blockers.len(), 1);
    assert_eq!(plan.blockers[0].kind, "provider_unavailable");
    assert_eq!(plan.blockers[0].error_code, "LIVE_CAPABILITY_MISSING");

    let (surface, registry) = registered_surface();
    match HoloExecutor::with_view_surfaces(registry)
        .start_session(&browser)
        .await
    {
        Err(error @ LiveError::Capability(_)) => {
            assert_eq!(error.code(), "LIVE_CAPABILITY_MISSING");
            assert_eq!(error.to_string(), plan.blockers[0].message);
        }
        Err(error) => panic!("wrong startup refusal: {error:?}"),
        Ok(_) => panic!("unsupported selector started a session"),
    }
    assert!(surface.events.lock().expect("events").is_empty());
}
