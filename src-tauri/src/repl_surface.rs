//! my-lisp's own human-language surfaces (uk/en/sa/core), pulled directly from
//! the `external/my-lisp` git submodule via `include_str!` — the same
//! `lib/surface/{uk,sa}.lisp` files the native `my-lisp` CLI REPL loads
//! (see `crates/my-lisp-cli/src/repl.rs` and `docs/repl-surfaces.md` there).
//!
//! This module intentionally does not reimplement or fork that content: it
//! compiles in the submodule's real files, so a submodule bump automatically
//! carries any surface fix or extension into my-idea without a manual sync.
//!
//! Мовні поверхні самого my-lisp (uk/en/sa/core), взяті напряму з git
//! підмодуля `external/my-lisp` через `include_str!` — ті самі файли
//! `lib/surface/{uk,sa}.lisp`, які завантажує нативний CLI REPL my-lisp.
//! Модуль свідомо не дублює цей вміст: він вкомпільовує реальні файли
//! підмодуля, тож оновлення підмодуля саме переносить будь-яке виправлення
//! чи розширення поверхні в my-idea без ручної синхронізації.

use my_lisp::{eval_program, Environment, PresentationLanguage, Session};

const SURFACE_PREREQUISITES: &[(&str, &str)] = &[
    ("unify.lisp", include_str!("../../external/my-lisp/lib/unify.lisp")),
    ("reason.lisp", include_str!("../../external/my-lisp/lib/reason.lisp")),
    ("forward.lisp", include_str!("../../external/my-lisp/lib/forward.lisp")),
    ("knowledge.lisp", include_str!("../../external/my-lisp/lib/knowledge.lisp")),
    (
        "persistent-map.lisp",
        include_str!("../../external/my-lisp/lib/persistent-map.lisp"),
    ),
    (
        "persistent-vector.lisp",
        include_str!("../../external/my-lisp/lib/persistent-vector.lisp"),
    ),
    ("time.lisp", include_str!("../../external/my-lisp/lib/time.lisp")),
    ("epistemic.lisp", include_str!("../../external/my-lisp/lib/epistemic.lisp")),
];
const UK_SURFACE: &str = include_str!("../../external/my-lisp/lib/surface/uk.lisp");
const SA_SURFACE: &str = include_str!("../../external/my-lisp/lib/surface/sa.lisp");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplSurface {
    Core,
    English,
    Ukrainian,
    Sanskrit,
}

impl Default for ReplSurface {
    fn default() -> Self {
        Self::Core
    }
}

impl ReplSurface {
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_lowercase().as_str() {
            "core" | "ядро" => Some(Self::Core),
            "en" | "english" | "англійська" => Some(Self::English),
            "uk" | "ук" | "українська" => Some(Self::Ukrainian),
            "sa" | "sanskrit" | "санскрит" => Some(Self::Sanskrit),
            _ => None,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Self::Core => "core",
            Self::English => "en",
            Self::Ukrainian => "ук",
            Self::Sanskrit => "sa",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::Core => "ядро",
            Self::English => "англійська",
            Self::Ukrainian => "українська",
            Self::Sanskrit => "санскрит",
        }
    }

    pub fn presentation(self) -> PresentationLanguage {
        match self {
            Self::Core => PresentationLanguage::Canonical,
            Self::English => PresentationLanguage::English,
            Self::Ukrainian => PresentationLanguage::Ukrainian,
            Self::Sanskrit => PresentationLanguage::Sanskrit,
        }
    }
}

/// Builds a child environment carrying the requested surface's names on top
/// of `base`, matching `my-lisp-cli`'s `build_surface_layer` exactly.
pub fn build_surface_layer(base: &Environment, surface: ReplSurface) -> Result<Environment, String> {
    let layer = base.child();
    if matches!(surface, ReplSurface::Ukrainian | ReplSurface::Sanskrit) {
        let mut session = Session {
            environment: layer.clone(),
        };
        for (name, source) in SURFACE_PREREQUISITES {
            eval_program(source, &mut session)
                .map_err(|error| format!("не вдалося завантажити {name}: {}", error.render(source)))?;
        }
        let (name, source) = match surface {
            ReplSurface::Ukrainian => ("uk.lisp", UK_SURFACE),
            ReplSurface::Sanskrit => ("sa.lisp", SA_SURFACE),
            ReplSurface::Core | ReplSurface::English => unreachable!(),
        };
        eval_program(source, &mut session)
            .map_err(|error| format!("не вдалося завантажити {name}: {}", error.render(source)))?;
    }
    Ok(layer)
}
