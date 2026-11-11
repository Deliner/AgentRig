use super::descriptor::{Descriptor, TEST_CODE};
use super::parameters::Parameters;
use crate::shared::diagnostics::lint::Level as Severity;
use serde::{Deserialize, Serialize};

// DECISION: D133
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Signature {
    ParameterCount,
    BooleanParameter,
    TupleAsRecord,
    OrdEqMismatch,
    SecretDebug,
    UncheckedDeserialize,
    ImplicitOrder,
    SameTypeParameters,
    SameTypePositions,
    PrimitiveAlias,
    PartialImpl,
    AmbiguousOption,
    ConstantDefault,
}
impl Signature {
    pub(super) fn descriptor(self) -> &'static Descriptor {
        match self {
            Self::ParameterCount => &PARAMETERCOUNT,
            Self::BooleanParameter => &BOOLEANPARAMETER,
            Self::TupleAsRecord => &TUPLEASRECORD,
            Self::OrdEqMismatch => &ORDEQMISMATCH,
            Self::SecretDebug => &SECRETDEBUG,
            Self::UncheckedDeserialize => &UNCHECKEDDESERIALIZE,
            Self::ImplicitOrder => &IMPLICITORDER,
            Self::SameTypeParameters => &SAMETYPEPARAMETERS,
            Self::SameTypePositions => &SAMETYPEPOSITIONS,
            Self::PrimitiveAlias => &PRIMITIVEALIAS,
            Self::PartialImpl => &PARTIALIMPL,
            Self::AmbiguousOption => &AMBIGUOUSOPTION,
            Self::ConstantDefault => &CONSTANTDEFAULT,
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
// DECISION: D131
// DECISION: D135
const UNCHECKEDDESERIALIZE: Descriptor = Descriptor {
    default_id: "unchecked-deserialize",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust struct deriving Deserialize, directly or inside cfg_attr, with a checking constructor in the same file: a function without self in its inherent impl, impl TryFrom or impl FromStr, returning a Result or Option of Self, that refuses on a parameter tested, directly or through a local bound by let from it, by an if returning Err or None, calling bail! or panic! or ending in Err or None, by ensure! or by then or then_some, and stores that parameter in a named or tuple field without serde deserialize_with or with; one finding per struct, none with serde try_from or from on the struct, a manual Deserialize impl in the file or in test code",
    skill: "check-deserialize",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "the derived Deserialize builds the value without the check its constructor enforces, so data from a file or the network bypasses the invariant; route deserialization through the same check, for example serde(try_from = ...) from a raw shape",
    },
};
// DECISION: D155
const IMPLICITORDER: Descriptor = Descriptor {
    default_id: "implicit-order",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust enum of two or more variants deriving PartialOrd or Ord, directly or inside cfg_attr, with a variant lacking an explicit discriminant, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above the enum or its attributes",
    skill: "pin-the-order",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "a derived order follows declaration order, so adding or sorting variants silently changes every comparison; give each variant an explicit discriminant, compare through a named rank method, or put a DECISION: or INVARIANT: reference on the line directly above the enum when any stable order serves",
    },
};
// DECISION: D156
const SAMETYPEPARAMETERS: Descriptor = Descriptor {
    default_id: "same-type-parameters",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust fn item or fn signature with two non-receiver parameters of token-equal type, skipping fns inside impl Trait for a type, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "name-the-arguments",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "two parameters of one type can be swapped at a call and still compile; give each a type of its own, pass a named record, or put a DECISION: or INVARIANT: reference on the line directly above the fn when the operation is commutative",
    },
};
// DECISION: D156
const SAMETYPEPOSITIONS: Descriptor = Descriptor {
    default_id: "same-type-positions",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust tuple struct or tuple enum variant with two fields of token-equal type, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "name-the-positions",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "two positions of one type are told apart only by .0 and .1, so a swap compiles; name the fields, or put a DECISION: or INVARIANT: reference on the line directly above it when the order is the meaning itself",
    },
};
// DECISION: D156
const PRIMITIVEALIAS: Descriptor = Descriptor {
    default_id: "primitive-alias",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust type alias without generics whose type is a numeric primitive, char or String, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "wrap-the-primitive",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "an alias of a primitive is the primitive, so any other value of that type passes where the alias is expected; declare a newtype, or put a DECISION: or INVARIANT: reference on the line directly above it for an FFI alias",
    },
};
// DECISION: D156
const PARTIALIMPL: Descriptor = Descriptor {
    default_id: "partial-impl",
    exclude: TEST_CODE,
    target: "file",
    metric: "every manual Rust impl of PartialEq, Hash, PartialOrd, Ord or Clone whose method reads a self field by access, without a struct or tuple-struct pattern of the type lacking .. and without a struct literal of the type lacking a base, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above the impl",
    skill: "destructure-the-fields",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "a field added to the type is silently left out of this comparison, hash or copy; destructure Self without .. so the compiler names every field, or put a DECISION: or INVARIANT: reference on the line directly above the impl",
    },
};
// DECISION: D156
const AMBIGUOUSOPTION: Descriptor = Descriptor {
    default_id: "ambiguous-option",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust Option<bool> or Option<Option<..>> type, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "name-the-states",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "Option<bool> and Option<Option<T>> encode three states without names, and unwrap_or merges two of them silently; declare an enum naming each state, or put a DECISION: or INVARIANT: reference on the line directly above it for a DTO of an external format",
    },
};
// DECISION: D156
const CONSTANTDEFAULT: Descriptor = Descriptor {
    default_id: "constant-default",
    exclude: TEST_CODE,
    target: "file",
    metric: "every Rust trait fn with a nonempty default body that does not mention self, outside test code, without a DECISION: or INVARIANT: reference comment on the line directly above it",
    skill: "require-the-method",
    parameters: Parameters::Level {
        default: Severity::Error,
        violation: "a default body that ignores self hands every implementor a constant it never decided; drop the default so each implementor writes it, or put a DECISION: or INVARIANT: reference on the line directly above it for a compatibility promise",
    },
};
