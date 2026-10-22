// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! OpenAPI document of the web service.
//!
//! The operations are documented by annotating the request handlers with
//! `#[utoipa::path]`. The handlers are routed with [`utoipa_axum::routes!`],
//! i.e. the documented paths are the actual routes.
//!
//! The schemas of request and response bodies are not defined twice. They are
//! generated from the existing [JSON Schema](schemars::JsonSchema) definitions
//! of the corresponding types. The annotations reference these schemas through
//! the marker types that are declared in this module.

use std::borrow::Cow;

use schemars::{
    JsonSchema,
    generate::{SchemaGenerator, SchemaSettings},
};
use serde_json::{Map, Value};
use utoipa::{
    IntoParams, OpenApi, PartialSchema, ToResponse, ToSchema,
    openapi::{
        ContentBuilder, RefOr, Required, Response, ResponseBuilder,
        path::{Parameter, ParameterBuilder, ParameterIn},
        schema::{Ref, Schema},
    },
};

use aoide_backend_webapi_json as api;

use crate::routing::api::storage;

pub(crate) const COLLECTIONS_TAG: &str = "Collections";
pub(crate) const COLLECTED_MEDIA_SOURCES_TAG: &str = "Collections: Media Sources";
pub(crate) const COLLECTED_MEDIA_TRACKER_TAG: &str = "Collections: Media Tracker";
pub(crate) const COLLECTED_PLAYLISTS_TAG: &str = "Collections: Playlists";
pub(crate) const COLLECTED_TRACKS_TAG: &str = "Collections: Tracks";
pub(crate) const MEDIA_TRACKER_TAG: &str = "Media Tracker";
pub(crate) const TRACKS_TAG: &str = "Tracks";
pub(crate) const PLAYLISTS_TAG: &str = "Playlists";
pub(crate) const STORAGE_TAG: &str = "Storage";
pub(crate) const ADMINISTRATION_TAG: &str = "Administration";
pub(crate) const SCHEMAS_TAG: &str = "Schemas";

#[derive(OpenApi)]
#[openapi(
    servers((url = "/", description = "This server")),
    info(
        title = "aoide",
        description = "Music library management",
        contact(url = "https://gitlab.com/uklotzde/aoide-rs"),
        license(
            name = "AGPL-3.0-or-later",
            url = "https://www.gnu.org/licenses/agpl-3.0.html"
        ),
    ),
    tags(
        (name = COLLECTIONS_TAG),
        (name = COLLECTED_MEDIA_SOURCES_TAG),
        (name = COLLECTED_MEDIA_TRACKER_TAG),
        (name = COLLECTED_PLAYLISTS_TAG),
        (name = COLLECTED_TRACKS_TAG),
        (name = MEDIA_TRACKER_TAG),
        (name = TRACKS_TAG),
        (name = PLAYLISTS_TAG),
        (name = STORAGE_TAG),
        (name = ADMINISTRATION_TAG),
        (
            name = SCHEMAS_TAG,
            description = "JSON schemas of requests and responses, generated from the server code."
        ),
    ),
    components(responses(
        BadRequest,
        NotFound,
        Conflict,
        UnsupportedMediaType,
        UnprocessableEntity,
        InternalServerError,
    ))
)]
pub(crate) struct ApiDoc;

/// The location of all named schemas in the OpenAPI document.
const COMPONENT_SCHEMAS_PATH: &str = "/components/schemas";

/// Declares named schema components that are generated from JSON schemas.
///
/// The `request` schemas describe how the types are deserialized and the
/// `response` schemas describe how they are serialized.
///
/// If the type itself is a named JSON schema then the component must have
/// the same name. This is checked when generating the components.
macro_rules! json_schema_components {
    ($($contract:ident $name:ident = $ty:ty;)+) => {
        $(
            pub(crate) struct $name;

            impl PartialSchema for $name {
                fn schema() -> RefOr<Schema> {
                    // Placeholder that is replaced by the generated JSON schema.
                    RefOr::T(Schema::default())
                }
            }

            impl ToSchema for $name {
                fn name() -> Cow<'static, str> {
                    Cow::Borrowed(stringify!($name))
                }
            }
        )+

        fn generate_json_schema_components() -> Map<String, Value> {
            let mut generators = ComponentGenerators::new();
            $(
                generators.$contract.add_component::<$ty>(stringify!($name));
            )+
            generators.finish()
        }
    };
}

json_schema_components! {
    request Collection = api::collection::create::RequestBody;
    response CollectionEntity = api::collection::create::ResponseBody;
    response LoadCollectionsResponseBody = api::collection::load_all::ResponseBody;
    response LoadCollectionKindsResponseBody = api::collection::load_all_kinds::ResponseBody;
    response CollectionWithSummaryEntity = api::collection::load_one::ResponseBody;

    request ResolveTracksRequestBody = api::track::resolve::RequestBody;
    response ResolveTracksResponseBody = api::track::resolve::ResponseBody;
    request TrackSearchParams = api::track::search::RequestBody;
    response SearchTracksResponseBody = api::track::search::ResponseBody;
    request ReplaceTracksRequestBody = api::track::replace::RequestBody;
    response ReplaceTracksSummary = api::track::replace::ResponseBody;
    request ImportAndReplaceTracksRequestBody = api::track::import_and_replace::RequestBody;
    response ImportAndReplaceTracksOutcome = api::track::import_and_replace::ResponseBody;
    request FindUnsynchronizedTracksRequestBody = api::track::find_unsynchronized::RequestBody;
    response FindUnsynchronizedTracksResponseBody = api::track::find_unsynchronized::ResponseBody;
    request ExportTrackFilesRequestBody = api::track::vfs::export_files::RequestBody;
    response ExportTrackFilesResponseBody = api::track::vfs::export_files::ResponseBody;
    request LoadTracksRequestBody = api::track::load_many::RequestBody;
    response LoadTracksResponseBody = api::track::load_many::ResponseBody;
    response TrackBodyEntity = api::track::load_one::ResponseBody;

    request PurgeOrphanedMediaSourcesParams = api::media::source::purge_orphaned::RequestBody;
    response PurgeOrphanedMediaSourcesOutcome = api::media::source::purge_orphaned::ResponseBody;
    request PurgeUntrackedMediaSourcesParams = api::media::source::purge_untracked::RequestBody;
    response PurgeUntrackedMediaSourcesOutcome = api::media::source::purge_untracked::ResponseBody;
    request RelocateMediaSourcesRequestBody = api::media::source::relocate::RequestBody;
    response RelocateMediaSourcesResponseBody = api::media::source::relocate::ResponseBody;

    request MediaTrackerQueryStatusParams = api::media::tracker::query_status::RequestBody;
    response MediaTrackerStatus = api::media::tracker::query_status::ResponseBody;
    request MediaTrackerFsTraversalParams = api::media::tracker::scan_directories::RequestBody;
    response MediaTrackerScanDirectoriesOutcome = api::media::tracker::scan_directories::ResponseBody;
    request MediaTrackerUntrackDirectoriesParams = api::media::tracker::untrack_directories::RequestBody;
    response MediaTrackerUntrackDirectoriesOutcome = api::media::tracker::untrack_directories::ResponseBody;
    request MediaTrackerImportFilesParams = api::media::tracker::import_files::RequestBody;
    response MediaTrackerImportFilesOutcome = api::media::tracker::import_files::ResponseBody;
    response MediaTrackerFindUntrackedFilesOutcome = api::media::tracker::find_untracked_files::ResponseBody;
    response MediaTrackerProgress = api::media::tracker::Progress;

    request Playlist = api::playlist::create::RequestBody;
    response PlaylistEntity = api::playlist::create::ResponseBody;
    response LoadPlaylistsResponseBody = api::playlist::load_all::ResponseBody;
    response PlaylistWithEntriesEntity = api::playlist::load_one::ResponseBody;
    request PatchPlaylistEntriesRequestBody = api::playlist::entries::patch::RequestBody;
    response PlaylistWithEntriesSummaryEntity = api::playlist::entries::patch::ResponseBody;

    response StoragePendingTasks = storage::PendingTasksResponseBody;

    response ErrorResponse = aoide_websrv_axum_sqlite::ErrorResponseBody;
}

/// Declares query parameters that are generated from the JSON schema of a type.
///
/// The type must be represented by a JSON object. Each of its properties
/// becomes a query parameter.
macro_rules! query_params {
    ($($name:ident = $ty:ty;)+) => {
        $(
            pub(crate) struct $name;

            impl IntoParams for $name {
                fn into_params(_: impl Fn() -> Option<ParameterIn>) -> Vec<Parameter> {
                    query_parameters::<$ty>()
                }
            }
        )+
    };
}

query_params! {
    LoadCollectionsQuery = api::collection::load_all::QueryParams;
    UpdateCollectionQuery = api::collection::update::QueryParams;
    SearchTracksQuery = api::track::search::QueryParams;
    ReplaceTracksQuery = api::track::replace::QueryParams;
    ImportAndReplaceTracksQuery = api::track::import_and_replace::QueryParams;
    FindUnsynchronizedTracksQuery = api::track::find_unsynchronized::QueryParams;
    ExportTrackMetadataQuery = api::track::export_metadata::QueryParams;
    LoadPlaylistsQuery = api::playlist::load_all::QueryParams;
    EntityRevQuery = api::EntityRevQueryParams;
    CleanseStorageQuery = storage::CleanseQueryParams;
}

/// Declares a single, required path parameter with the JSON schema of a type.
macro_rules! path_params {
    ($($name:ident = $param_name:literal: $ty:ty, $description:literal;)+) => {
        $(
            pub(crate) struct $name;

            impl IntoParams for $name {
                fn into_params(_: impl Fn() -> Option<ParameterIn>) -> Vec<Parameter> {
                    vec![path_parameter::<$ty>($param_name, $description)]
                }
            }
        )+
    };
}

path_params! {
    CollectionUidPath = "collectionUid": aoide_core::EntityUid, "The UID of the collection.";
    PlaylistUidPath = "playlistUid": aoide_core::EntityUid, "The UID of the playlist.";
    TrackUidPath = "trackUid": aoide_core::EntityUid, "The UID of the track.";
}

/// Declares reusable error responses with an [`ErrorResponse`] body.
macro_rules! error_responses {
    ($($name:ident = $description:literal;)+) => {
        $(
            pub(crate) struct $name;

            impl<'r> ToResponse<'r> for $name {
                fn response() -> (&'r str, RefOr<Response>) {
                    (stringify!($name), error_response($description).into())
                }
            }
        )+
    };
}

error_responses! {
    BadRequest = "The request with the provided parameters was invalid.";
    NotFound = "The specified resource was not found.";
    Conflict = "The specified resource could not be updated or patched due to a conflict.";
    UnsupportedMediaType = "The request body is not declared as JSON, i.e. the header \
        `Content-Type: application/json` is missing.";
    UnprocessableEntity = "The request body is valid JSON but does not match the expected schema.";
    InternalServerError = "The server failed to process the request.";
}

fn error_response(description: &str) -> Response {
    ResponseBuilder::new()
        .description(description)
        .content(
            "application/json",
            ContentBuilder::new()
                .schema(Some(Ref::from_schema_name(ErrorResponse::name())))
                .build(),
        )
        .build()
}

fn schema_settings() -> SchemaSettings {
    // OpenAPI 3.1 is compatible with JSON Schema 2020-12.
    SchemaSettings::draft2020_12().with(|settings| {
        settings.meta_schema = None;
    })
}

/// Generates the schema components for requests and responses separately.
struct ComponentGenerators {
    request: ComponentGenerator,
    response: ComponentGenerator,
}

impl ComponentGenerators {
    fn new() -> Self {
        Self {
            request: ComponentGenerator::new(schema_settings().for_deserialize()),
            response: ComponentGenerator::new(schema_settings().for_serialize()),
        }
    }

    fn finish(self) -> Map<String, Value> {
        let Self { request, response } = self;
        let mut components = request.finish();
        for (name, schema) in response.finish() {
            if let Some(request_schema) = components.get(&name) {
                // Types that are used both in requests and responses need distinct
                // names if they are serialized differently than they are deserialized.
                assert_eq!(
                    *request_schema, schema,
                    "request and response schemas of {name} differ"
                );
                continue;
            }
            components.insert(name, schema);
        }
        components
    }
}

struct ComponentGenerator {
    generator: SchemaGenerator,
    components: Vec<(&'static str, Value)>,
}

impl ComponentGenerator {
    fn new(settings: SchemaSettings) -> Self {
        let generator = settings
            .with(|settings| settings.definitions_path = COMPONENT_SCHEMAS_PATH.into())
            .into_generator();
        Self {
            generator,
            components: Vec::new(),
        }
    }

    fn add_component<T: JsonSchema>(&mut self, name: &'static str) {
        let schema = self.generator.subschema_for::<T>().to_value();
        if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
            // Named JSON schemas are added by the generator.
            let schema_name = reference.trim_start_matches(&format!("#{COMPONENT_SCHEMAS_PATH}/"));
            assert_eq!(
                name, schema_name,
                "schema component {name} must be named after the JSON schema"
            );
            return;
        }
        self.components.push((name, schema));
    }

    fn finish(self) -> Map<String, Value> {
        let Self {
            mut generator,
            components,
        } = self;
        let mut definitions = generator.take_definitions(true);
        for (name, schema) in components {
            // The named JSON schemas are only known after all components have been generated.
            let conflicting = definitions.insert(name.to_owned(), schema);
            assert!(
                conflicting.is_none(),
                "schema component {name} conflicts with a JSON schema"
            );
        }
        definitions
    }
}

fn query_parameters<T: JsonSchema>() -> Vec<Parameter> {
    let schema = inline_json_schema::<T>();
    let required = schema
        .get("required")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let Some(properties) = schema.get("properties").and_then(Value::as_object) else {
        assert_eq!(
            schema.get("type").and_then(Value::as_str),
            Some("object"),
            "query parameters must be represented by an object"
        );
        return Vec::new();
    };
    properties
        .iter()
        .map(|(name, schema)| {
            let required = required.iter().any(|required| required == name);
            parameter(name, ParameterIn::Query, required, schema.clone(), None)
        })
        .collect()
}

fn path_parameter<T: JsonSchema>(name: &str, description: &str) -> Parameter {
    let mut schema = inline_json_schema::<T>();
    if let Some(schema) = schema.as_object_mut() {
        // The title of the root schema is the name of the type.
        schema.remove("title");
    }
    parameter(name, ParameterIn::Path, true, schema, Some(description))
}

/// Generates a self-contained JSON schema without references.
fn inline_json_schema<T: JsonSchema>() -> Value {
    schema_settings()
        .for_deserialize()
        .with(|settings| settings.inline_subschemas = true)
        .into_generator()
        .into_root_schema_for::<T>()
        .to_value()
}

fn parameter(
    name: &str,
    parameter_in: ParameterIn,
    required: bool,
    mut schema: Value,
    description: Option<&str>,
) -> Parameter {
    // The description of the schema, e.g. from a doc comment of the field,
    // documents the parameter unless a description is given explicitly.
    let schema_description = schema
        .as_object_mut()
        .and_then(|schema| schema.remove("description"));
    let description = description.map(ToOwned::to_owned).or_else(|| {
        schema_description.and_then(|description| description.as_str().map(ToOwned::to_owned))
    });
    let schema = serde_json::from_value::<RefOr<Schema>>(non_nullable(schema))
        .unwrap_or_else(|err| panic!("unsupported schema of parameter {name}: {err}"));
    ParameterBuilder::new()
        .name(name)
        .parameter_in(parameter_in)
        .required(if required {
            Required::True
        } else {
            Required::False
        })
        .description(description)
        .schema(Some(schema))
        .build()
}

/// Strips `null` from the schema of an optional value.
///
/// Missing parameters are represented by omitting them, not by `null`.
fn non_nullable(mut schema: Value) -> Value {
    let Some(object) = schema.as_object_mut() else {
        return schema;
    };
    if let Some(Value::Array(types)) = object.get_mut("type") {
        types.retain(|t| t != "null");
        if let [single_type] = types.as_slice() {
            let single_type = single_type.clone();
            object.insert("type".to_owned(), single_type);
        }
    }
    if let Some(Value::Array(values)) = object.get_mut("enum") {
        values.retain(|value| !value.is_null());
    }
    if let Some(Value::Array(subschemas)) = object.get("anyOf") {
        let is_null = |schema: &Value| schema.get("type").and_then(Value::as_str) == Some("null");
        if let [subschema] = subschemas
            .iter()
            .filter(|schema| !is_null(schema))
            .collect::<Vec<_>>()
            .as_slice()
        {
            let mut subschema = (*subschema).clone();
            object.remove("anyOf");
            if let Some(subschema) = subschema.as_object_mut() {
                // Keep the remaining keywords, e.g. a default value.
                subschema.extend(std::mem::take(object));
            }
            return non_nullable(subschema);
        }
    }
    schema
}

/// Finishes the OpenAPI document that has been collected from the routes.
///
/// Replaces the placeholders of the [JSON schema components](json_schema_components)
/// with the generated JSON schemas.
pub(crate) fn finish_document(openapi: utoipa::openapi::OpenApi) -> Value {
    let mut document = serde_json::to_value(openapi).expect("OpenAPI document");
    let schemas = document
        .as_object_mut()
        .expect("OpenAPI document")
        .entry("components")
        .or_insert_with(|| Value::Object(Map::new()))
        .as_object_mut()
        .expect("components")
        .entry("schemas")
        .or_insert_with(|| Value::Object(Map::new()))
        .as_object_mut()
        .expect("schemas");
    schemas.extend(generate_json_schema_components());
    document
}

#[cfg(test)]
mod tests;
