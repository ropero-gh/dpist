#[derive(Debug, Clone, Copy)]
pub enum FieldKind {
    Path,
    U64,
    Bool,
}

pub struct FieldSpec {
    pub name: &'static str,
    pub kind: FieldKind,
}

pub struct ModifierSpec {
    pub name: &'static str,
    pub fields: &'static [FieldSpec],
}

pub struct UiSchema;

impl UiSchema {
    pub const GENERAL: &'static [FieldSpec] = &[
        FieldSpec {
            name: "input",
            kind: FieldKind::Path,
        },
        FieldSpec {
            name: "output",
            kind: FieldKind::Path,
        },
    ];

    pub const MODIFIERS: &'static [ModifierSpec] = &[
        ModifierSpec {
            name: "DropEveryNth",
            fields: &[FieldSpec {
                name: "n",
                kind: FieldKind::U64,
            }],
        },
        ModifierSpec {
            name: "Delay::Fixed",
            fields: &[FieldSpec {
                name: "millis",
                kind: FieldKind::U64,
            }],
        },
        ModifierSpec {
            name: "Delay::Jitter",
            fields: &[
                FieldSpec {
                    name: "min_ms",
                    kind: FieldKind::U64,
                },
                FieldSpec {
                    name: "max_ms",
                    kind: FieldKind::U64,
                },
            ],
        },
        ModifierSpec {
            name: "Delay::PerFlow",
            fields: &[FieldSpec {
                name: "millis",
                kind: FieldKind::U64,
            }],
        },
        ModifierSpec {
            name: "Delay::Burst",
            fields: &[
                FieldSpec {
                    name: "active_ms",
                    kind: FieldKind::U64,
                },
                FieldSpec {
                    name: "pause_ms",
                    kind: FieldKind::U64,
                },
            ],
        },
    ];
}
