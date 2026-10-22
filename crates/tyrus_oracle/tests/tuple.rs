#![allow(clippy::expect_used, clippy::unwrap_used)]

use tyrus_oracle::tuple::{NodeRole, NodeTuple, ENV_NODE_REFERENCE, ENV_NODE_SEED};

fn reference_tuple() -> NodeTuple {
    NodeTuple {
        node: "26.11.1".into(),
        v8: "14.6.202.34-node.37".into(),
        icu: "78.3".into(),
        unicode: "17.0".into(),
        tz: "2026e".into(),
        cldr: "48.0".into(),
    }
}

#[test]
fn identical_tuples_have_no_diff() {
    assert_eq!(reference_tuple().diff(&reference_tuple()), Vec::new());
}

#[test]
fn diff_names_every_differing_field_in_declaration_order() {
    let mut actual = reference_tuple();
    actual.tz = "2026c".into();
    actual.v8 = "14.7.173.7".into();
    let diffs = reference_tuple().diff(&actual);
    let fields: Vec<&str> = diffs.iter().map(|d| d.field).collect();
    assert_eq!(fields, ["v8", "tz"]);
    let tz = diffs.last().expect("duas diferenças");
    assert_eq!(
        (tz.expected.as_str(), tz.actual.as_str()),
        ("2026e", "2026c")
    );
}

#[test]
fn diff_is_sensitive_to_every_single_field() {
    let base = reference_tuple();
    let variants = [
        NodeTuple {
            node: "26.11.2".into(),
            ..base.clone()
        },
        NodeTuple {
            v8: "x".into(),
            ..base.clone()
        },
        NodeTuple {
            icu: "x".into(),
            ..base.clone()
        },
        NodeTuple {
            unicode: "x".into(),
            ..base.clone()
        },
        NodeTuple {
            tz: "x".into(),
            ..base.clone()
        },
        NodeTuple {
            cldr: "x".into(),
            ..base.clone()
        },
    ];
    for variant in variants {
        assert_eq!(base.diff(&variant).len(), 1, "{variant:?}");
    }
}

#[test]
fn roles_map_to_their_own_env_vars() {
    assert_eq!(NodeRole::Reference.env_var(), ENV_NODE_REFERENCE);
    assert_eq!(NodeRole::Seed.env_var(), ENV_NODE_SEED);
    assert_eq!(ENV_NODE_REFERENCE, "TYRUS_NODE_REFERENCE");
    assert_eq!(ENV_NODE_SEED, "TYRUS_NODE_SEED");
    assert_eq!(NodeRole::Reference.to_string(), "reference");
    assert_eq!(NodeRole::Seed.to_string(), "seed");
}

#[test]
fn role_parses_from_cli_name() {
    assert_eq!(
        "reference".parse::<NodeRole>().unwrap(),
        NodeRole::Reference
    );
    assert_eq!("seed".parse::<NodeRole>().unwrap(), NodeRole::Seed);
    assert!("Reference".parse::<NodeRole>().is_err());
}
