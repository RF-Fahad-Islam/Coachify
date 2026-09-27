use coachify_core::seam::{command_descriptors, register_all_types};
use specta::Types;
use specta_typescript::Typescript;
use std::borrow::Cow;

use specta::datatype::{DataType, Fields, NamedReferenceType, Primitive, Reference};

fn sanitize_datatype(dt: &mut DataType) {
    if let DataType::Reference(Reference::Named(named_ref)) = dt {
        let dbg = format!("{:?}", named_ref);
        if dbg.contains("serde_json") {
            *dt = <specta_typescript::Unknown as specta::Type>::definition(&mut Types::default());
            return;
        }
    }

    match dt {
        DataType::Primitive(
            Primitive::i64
            | Primitive::u64
            | Primitive::isize
            | Primitive::usize
            | Primitive::i128
            | Primitive::u128
            | Primitive::f32
            | Primitive::f64,
        ) => {
            *dt = DataType::Primitive(Primitive::i32);
        }
        DataType::Primitive(_) => {}
        DataType::List(list) => {
            sanitize_datatype(&mut list.ty);
        }
        DataType::Map(map) => {
            sanitize_datatype(map.key_ty_mut());
            sanitize_datatype(map.value_ty_mut());
        }
        DataType::Nullable(inner) => {
            sanitize_datatype(inner);
        }
        DataType::Struct(strct) => {
            sanitize_fields(&mut strct.fields);
        }
        DataType::Enum(enm) => {
            for (_variant_name, variant) in &mut enm.variants {
                sanitize_fields(&mut variant.fields);
            }
        }
        DataType::Tuple(tuple) => {
            for elem in &mut tuple.elements {
                sanitize_datatype(elem);
            }
        }
        DataType::Intersection(types) => {
            for ty in types {
                sanitize_datatype(ty);
            }
        }
        DataType::Generic(_) => {}
        DataType::Reference(Reference::Named(named_ref)) => match &mut named_ref.inner {
            NamedReferenceType::Inline { dt, .. } => {
                sanitize_datatype(dt);
            }
            NamedReferenceType::Reference { generics, .. } => {
                for (_generic, gen_dt) in generics {
                    sanitize_datatype(gen_dt);
                }
            }
            NamedReferenceType::Recursive(_) => {}
        },
        DataType::Reference(Reference::Opaque(_)) => {}
    }
}

fn sanitize_fields(fields: &mut Fields) {
    match fields {
        Fields::Unit => {}
        Fields::Unnamed(unnamed) => {
            for field in &mut unnamed.fields {
                if let Some(ty) = field.ty.as_mut() {
                    if matches!(ty, DataType::Nullable(_)) {
                        field.optional = true;
                    }
                    sanitize_datatype(ty);
                }
            }
        }
        Fields::Named(named) => {
            for (_name, field) in &mut named.fields {
                if let Some(ty) = field.ty.as_mut() {
                    if matches!(ty, DataType::Nullable(_)) {
                        field.optional = true;
                    }
                    sanitize_datatype(ty);
                }
            }
        }
    }
}

fn lower_enum_variants(name: &str, ty: &mut DataType) {
    match (name, &mut *ty) {
        ("RowIssueSeverity" | "DemoProfile", DataType::Enum(enm)) => {
            for (variant_name, _) in &mut enm.variants {
                *variant_name = Cow::Owned(variant_name.to_lowercase());
            }
        }
        ("WorkstationConnectionState", DataType::Enum(enm)) => {
            for (variant_name, _) in &mut enm.variants {
                if variant_name == "Offline" {
                    *variant_name = Cow::Borrowed("disconnected");
                } else {
                    *variant_name = Cow::Owned(variant_name.to_lowercase());
                }
            }
        }
        ("WorkstationPrinterSettings", DataType::Struct(strct)) => {
            if let Fields::Named(named) = &mut strct.fields {
                for (field_name, field) in &mut named.fields {
                    if field_name == "page_size" {
                        let mut enm = specta::datatype::Enum::default();
                        enm.variants = vec![
                            (
                                Cow::Borrowed("thermal_80mm"),
                                specta::datatype::Variant::unit(),
                            ),
                            (Cow::Borrowed("a4_dual"), specta::datatype::Variant::unit()),
                            (
                                Cow::Borrowed("a5_single"),
                                specta::datatype::Variant::unit(),
                            ),
                        ];
                        field.ty = Some(DataType::Enum(enm));
                    }
                }
            }
        }
        _ => {}
    }

    if let DataType::Struct(strct) = ty {
        if let Fields::Named(named) = &mut strct.fields {
            for (field_name, field) in &mut named.fields {
                let is_opt = matches!(
                    (name, field_name.as_ref()),
                    (
                        "WorkstationClientStatus",
                        "mode" | "connection_state" | "is_reconnecting" | "reconnect_attempts",
                    ) | (
                        "StatementChargeLine",
                        "charge_id"
                            | "gross_amount_poisha"
                            | "concession_amount_poisha"
                            | "net_amount_poisha"
                            | "paid_amount_poisha"
                            | "remaining_due_poisha",
                    ) | (
                        "VendorDashboardOverview",
                        "channel_heads"
                            | "key_recovery_drills"
                            | "pricing_tiers"
                            | "pending_bank_transfers",
                    )
                );
                if is_opt {
                    field.optional = true;
                }
            }
        }
    }
}

pub struct SimpleFormat;

impl specta::Format for SimpleFormat {
    fn map_types(&'_ self, types: &Types) -> Result<Cow<'_, Types>, specta::FormatError> {
        let mut cloned = types.clone();
        cloned.iter_mut(|ndt| {
            let is_serde_json = ndt.module_path == "serde_json"
                || ndt.module_path.starts_with("serde_json::")
                || (ndt.name == "Value" && ndt.module_path.is_empty());
            if is_serde_json {
                ndt.ty = None;
            } else if let Some(ty) = ndt.ty.as_mut() {
                lower_enum_variants(&ndt.name, ty);
                sanitize_datatype(ty);
            }
        });
        Ok(Cow::Owned(cloned))
    }

    fn map_type(
        &'_ self,
        _types: &Types,
        dt: &specta::datatype::DataType,
    ) -> Result<Cow<'_, specta::datatype::DataType>, specta::FormatError> {
        let mut cloned = dt.clone();
        sanitize_datatype(&mut cloned);
        Ok(Cow::Owned(cloned))
    }
}

pub fn generate_ts_contract() -> String {
    let mut types = Types::default();
    register_all_types(&mut types);

    types.iter_mut(|ndt| {
        let is_serde_json = ndt.module_path == "serde_json"
            || ndt.module_path.starts_with("serde_json::")
            || (ndt.name == "Value" && ndt.module_path.is_empty());
        if is_serde_json {
            ndt.ty = None;
        } else if let Some(ty) = ndt.ty.as_mut() {
            lower_enum_variants(&ndt.name, ty);
            sanitize_datatype(ty);
        }
    });

    let ts_exporter = Typescript::default();
    let exported_types = ts_exporter
        .export(&types, SimpleFormat)
        .expect("Failed to export specta types");

    let descriptors = command_descriptors();

    let mut command_map_entries = String::new();
    for cmd in &descriptors {
        let req = if cmd.is_empty_req {
            "EmptyPayload"
        } else {
            cmd.request_type
        };
        command_map_entries.push_str(&format!(
            "  \"{}\": {{\n    request: {};\n    response: {};\n  }};\n",
            cmd.name, req, cmd.response_type
        ));
    }

    format!(
        r#"/**
 * Auto-generated by Coachify xtask export-types via Specta.
 * Command Seam: UI <-> Core boundary contract. Do not edit manually.
 */

import {{ invoke }} from "@tauri-apps/api/core";

export interface RpcEnvelope<T> {{
  ok: boolean;
  data?: T;
  error?: RpcError;
}}

export type EmptyPayload = Record<string, never> | Record<string, unknown>;
export type ValidatePathResponse = PathValidationResult | BackupValidationResult;
export type PromoteReleaseInput = PromoteReleaseChannelInput;
export type PortalLoginChallenge = PortalLoginResponse;
export type PortalSessionResponse = PortalVerifyOtpResponse;
export type UpdateTemplateConfigPayload = UpdatePrintTemplateConfigPayload;
export type CreateExamSubjectInput = ExamSubjectConfigInput;
export type MonthlyGenerationResult = GenerationResult;

export interface RpcOptions {{
  sessionToken?: string | null;
  idempotencyKey?: string | null;
}}

{}

export interface CommandMap {{
{}
}}

export type CommandName = keyof CommandMap;

/**
 * Single typed RPC entry point to the Rust core.
 */
export async function rpc<C extends CommandName>(
  command: C,
  payload: CommandMap[C]["request"],
  options?: RpcOptions | string | null
): Promise<RpcEnvelope<CommandMap[C]["response"]>> {{
  const isTauri =
    typeof window !== "undefined" &&
    Boolean(
      (window as unknown as {{ __TAURI_INTERNALS__?: {{ invoke?: unknown }} }})
        .__TAURI_INTERNALS__?.invoke
    );

  if (!isTauri) {{
    return {{
      ok: false,
      error: {{
        code: "tauri_unavailable",
        message:
          "Tauri desktop bridge is not available. Please open Coachify using the native desktop app window rather than an external web browser.",
      }},
    }};
  }}

  const sessionToken =
    typeof options === "object" && options !== null ? options.sessionToken : null;
  const idempotencyKey =
    typeof options === "string"
      ? options
      : typeof options === "object" && options !== null
        ? options.idempotencyKey
        : null;

  try {{
    const raw = await invoke<string | RpcEnvelope<CommandMap[C]["response"]>>("rpc", {{
      command,
      payload,
      sessionToken: sessionToken ?? null,
      idempotencyKey: idempotencyKey ?? null,
    }});
    if (typeof raw === "string") {{
      return JSON.parse(raw);
    }}
    return raw;
  }} catch (err: unknown) {{
    return {{
      ok: false,
      error: {{
        code: "transport_error",
        message: err instanceof Error ? err.message : String(err),
      }},
    }};
  }}
}}
"#,
        exported_types.trim(),
        command_map_entries.trim_end()
    )
}
