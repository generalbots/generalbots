use super::{ParamKind, ParamSpec};

pub(crate) const fn s(name: &'static str) -> ParamSpec {
    ParamSpec {
        name,
        kind: ParamKind::Str,
        required: false,
    }
}

pub(crate) const fn s_req(name: &'static str) -> ParamSpec {
    ParamSpec {
        name,
        kind: ParamKind::Str,
        required: true,
    }
}

pub(crate) const fn json(name: &'static str, required: bool) -> ParamSpec {
    ParamSpec {
        name,
        kind: ParamKind::Json,
        required,
    }
}

pub(crate) const fn json_req(name: &'static str) -> ParamSpec {
    ParamSpec {
        name,
        kind: ParamKind::Json,
        required: true,
    }
}

pub(crate) const fn resource_id() -> &'static [ParamSpec] {
    &[ParamSpec {
        name: "resource_id",
        kind: ParamKind::Str,
        required: true,
    }]
}
