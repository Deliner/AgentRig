use super::descriptor::{Descriptor, TEST_CODE};
use super::parameters::Parameters;
use crate::shared::diagnostics::lint::Level as Severity;
use serde::Serialize;

// DECISION: D133
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Signature {
    ParameterCount,
    BooleanParameter,
    TupleAsRecord,
    OrdEqMismatch,
    SecretDebug,
}
impl Signature {
    pub(super) fn descriptor(self) -> &'static Descriptor {
        match self {
            Self::ParameterCount => &PARAMETERCOUNT,
            Self::BooleanParameter => &BOOLEANPARAMETER,
            Self::TupleAsRecord => &TUPLEASRECORD,
            Self::OrdEqMismatch => &ORDEQMISMATCH,
            Self::SecretDebug => &SECRETDEBUG,
        }
    }
}
const PARAMETERCOUNT: Descriptor = Descriptor {
    default_id: "parameters",
    exclude: &[],
    target: "file",
    metric: "declared input parameters, excluding method receivers; constructors included; variadics count as one",
    skill: "reduce-parameters",
    parameters: Parameters::Thresholds {
        warning: None,
        error: 4,
    },
};
// DECISION: D062
const BOOLEANPARAMETER: Descriptor = Descriptor {
    default_id: "boolean-parameter",
    exclude: &[],
    target: "file",
    metric: "parameters typed as the Rust bool primitive, or annotated bool or defaulting to True or False in Python; each flagged parameter is a finding",
    skill: "split-boolean-parameter",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "a boolean parameter hides two behaviors; split the function or name the modes with a dedicated type",
    },
};
// DECISION: D088
// DECISION: D101
const TUPLEASRECORD: Descriptor = Descriptor {
    default_id: "tuple-as-record",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust function parameter type or return type that is a tuple, or that nests inside a generic or slice type, at any depth, a tuple of three or more elements with a primitive, String or &str element (closure signatures included for the nested form), outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above the function",
    skill: "name-the-record",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "a tuple in a signature is an unnamed record that evades the parameter-count and boolean-parameter budgets and hides what each position means, whether it stands alone or inside Vec<(..)>, Option<(..)> or &[(..)]; give it a named struct, or put a DECISION: or INVARIANT: reference on the line directly above the function to justify the tuple",
    },
};
// DECISION: D119
const ORDEQMISMATCH: Descriptor = Descriptor {
    default_id: "ord-eq-mismatch",
    exclude: TEST_CODE,
    target: "file",
    metric: "every manual Rust impl of Ord, or of PartialOrd for a type without a manual Ord in the same file, whose cmp or partial_cmp reads a different set of self fields than equality reads, the eq of a manual PartialEq or every named field of a struct deriving PartialEq in the same file, following self method calls into methods of the same type in that file, skipping a comparison with a self use or method call that file does not resolve, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above the impl",
    skill: "align-order-with-equality",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "order and equality that read different fields break the standard library contract that a == b exactly when the comparison is Equal, so BTreeSet, BTreeMap, sort with dedup and binary search disagree with ==; compare the same fields in both, derive both or express both through one key method, or put a DECISION: or INVARIANT: reference on the line directly above the impl to justify a field derived from the compared ones, such as a cache",
    },
};
// DECISION: D130
const SECRETDEBUG: Descriptor = Descriptor {
    default_id: "secret-debug",
    exclude: TEST_CODE,
    target: "file",
    metric: "every named field of a Rust struct deriving Debug, directly or inside cfg_attr, whose name is exactly password, passwd, secret, token, api_key, apikey, private_key, access_token, refresh_token, client_secret, jwt or cvv and whose type is String, &str, Vec<u8> or an Option of one of them, outside test code",
    skill: "redact-secret",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "a derived Debug prints the secret in full wherever the struct reaches {:?}, a log macro or a panic message; wrap the secret in a type whose Debug prints a placeholder, or write Debug by hand without the value",
    },
};
